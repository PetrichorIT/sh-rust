mod core;
pub use core::*;
use std::{error::Error, net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use rand::random;
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::{Mutex, RwLock},
};
use tower_http::services::ServeDir;

pub async fn launch() -> Result<(), Box<dyn Error>> {
    let static_file_service = ServeDir::new("frontend/dist").append_index_html_on_directories(true);
    let app = Router::new()
        .fallback_service(static_file_service)
        .route("/ws", get(handle_ws_upgrade))
        .with_state(Arc::new(RwLock::new(GameState::new(Vec::new()))));

    let port = std::env::var("PORT")
        .unwrap_or("8000".to_string())
        .parse()
        .unwrap();

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

async fn handle_ws_upgrade(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(game_state): State<Arc<RwLock<GameState>>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    tracing::info!("Connected {} at {}", user_agent, addr);
    ws.on_upgrade(move |socket| async move {
        match handle_ws(socket, addr, game_state).await {
            Ok(()) => (),
            Err(e) => tracing::error!("> {e} <"),
        }
    })
}

async fn handle_ws(
    mut ws: WebSocket,
    client: SocketAddr,
    game_state: Arc<RwLock<GameState>>,
) -> Result<(), Box<dyn Error>> {
    let cookie = random::<u128>();
    ws.send(Message::Ping(cookie.to_be_bytes().to_vec()))
        .await?;
    tracing::trace!("Send {cookie:?} to {client}");

    if let Some(msg) = ws.recv().await {
        if msg? == Message::Pong(cookie.to_be_bytes().to_vec()) {
            tracing::trace!("Recv {cookie:?} to {client}");
        } else {
            tracing::error!("Ping failed");
            return Ok(());
        }
    }

    let (tx, rx) = ws.split();
    let tx = Arc::new(Mutex::new(tx));

    let user = Arc::new(RwLock::new(String::new()));

    tokio::spawn(handle_ws_rx(
        rx,
        tx.clone(),
        user.clone(),
        game_state.clone(),
    ));
    tokio::spawn(handle_ws_tx(tx, user, game_state));

    Ok(())
}

async fn handle_ws_rx(
    mut rx: SplitStream<WebSocket>,
    tx: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    user: Arc<RwLock<PlayerId>>,
    game_state: Arc<RwLock<GameState>>,
) {
    while let Some(msg) = rx.next().await {
        let Ok(msg) = msg else {
            return;
        };
        match msg {
            Message::Ping(_) | Message::Pong(_) => {}
            Message::Binary(_) => {}
            Message::Text(raw) => {
                tracing::info!("recv {raw:?}");

                let incoming = match serde_json::from_str(&raw) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!("parsing error: {e}");
                        continue;
                    }
                };
                match incoming {
                    IncomingMessage::Authenticate(auth) => {
                        if auth.user.is_some() {
                            // Register new user
                            let player = game_state
                                .write()
                                .await
                                .add_new_user(auth.user.unwrap())
                                .unwrap();
                            *user.write().await = player.id;
                            tx.lock()
                                .await
                                .send(Message::Text(
                                    serde_json::to_string(&OutgoingMessage::Authenticated {
                                        access_key: player.access_key,
                                    })
                                    .unwrap(),
                                ))
                                .await
                                .unwrap();
                        } else {
                            // Reconnect user
                            let Ok(player) = game_state
                                .write()
                                .await
                                .join_existing_user(auth.access_key.unwrap())
                            else {
                                tracing::warn!("invalid access key");
                                continue;
                            };
                            *user.write().await = player.id;
                            tx.lock()
                                .await
                                .send(Message::Text(
                                    serde_json::to_string(&OutgoingMessage::Authenticated {
                                        access_key: player.access_key,
                                    })
                                    .unwrap(),
                                ))
                                .await
                                .unwrap();
                        }
                    }
                    IncomingMessage::GetState => {
                        let game_state = game_state.read().await;
                        let id = &user.read().await;
                        let view = game_state.view(id);
                        let tasks = game_state.tasks(id);
                        tx.lock()
                            .await
                            .send(Message::Text(
                                serde_json::to_string(&(view, tasks)).unwrap(),
                            ))
                            .await
                            .unwrap();
                    }
                    IncomingMessage::Task(task_response) => {
                        let mut game_state = game_state.write().await;
                        let id = &user.read().await;

                        tracing::info!("executing task {task_response:?}");
                        game_state.on_action(id.to_string(), task_response);
                    }
                    _ => todo!(),
                }
            }
            Message::Close(close_frame) => {
                tracing::info!("Closing socket");
                game_state.write().await.remove_player(&*user.read().await);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IncomingMessage {
    Task(TaskAction),
    Authenticate(AuthenticateMessage),
    GetState,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuthenticateMessage {
    pub user: Option<User>,
    pub access_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum OutgoingMessage {
    State {
        game_state: GameView,
        task: Option<Task>,
    },
    Authenticated {
        access_key: String,
    },
}

async fn handle_ws_tx(
    tx: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    user: Arc<RwLock<PlayerId>>,
    game_state: Arc<RwLock<GameState>>,
) {
    let notify = game_state.read().await.notify();
    loop {
        notify.notified().await;
        let id = &user.read().await;
        let game_state = game_state.read().await;

        let Some(view) = game_state.view(id) else {
            continue;
        };

        let msg = OutgoingMessage::State {
            game_state: view,
            task: game_state.tasks(id),
        };

        if let Err(e) = tx
            .lock()
            .await
            .send(Message::Text(serde_json::to_string(&msg).unwrap()))
            .await
        {
            tracing::error!("closing tx socket: {e}");
            break;
        }
    }
}
