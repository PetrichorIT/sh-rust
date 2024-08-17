mod cfg;
mod types;
mod view;

use std::collections::HashMap;
use std::sync::Arc;

use cfg::Config;
use rand::prelude::SliceRandom;
use rand::random;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

use tokio::sync::Notify;
use tracing::instrument;
pub use types::*;
pub use view::*;

#[derive(Debug, Clone)]
pub struct GameState {
    cfg: Config,
    board: BoardState,
    state: DynamicState,
    notify: Arc<Notify>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BoardState {
    players: Vec<Player>,

    draw_pile: Vec<Law>,
    discard_pile: Vec<Law>,

    executive_actions: [Option<ExecutiveAction>; 6],
    voting_result: Option<HashMap<PlayerId, bool>>,

    passed_fasho_laws: usize,
    passed_liberal_laws: usize,

    no_goverment_counter: usize,

    previous_president: Option<PlayerId>,
    previous_chancellor: Option<PlayerId>,

    current_president: PlayerId,
    next_president_by_rules: Option<PlayerId>,

    history: Vec<Event>,
    revealed_factions: Option<(PlayerId, PlayerId, Faction)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicState {
    Uninit,
    ChooseChancellor {
        options: Vec<PlayerId>,
    },
    VoteChancellor {
        chancellor: PlayerId,
        votes: HashMap<PlayerId, Option<bool>>,
    },
    PresidentChooseLaws {
        laws: [Law; 3],
        chancellor: PlayerId,
    },
    ChancellorChooseLaws {
        can_ask_veto: bool,
        laws: [Law; 2],
        chancellor: PlayerId,
    },
    ExecutiveAction {
        chancellor: PlayerId,
        action: ExecutiveActionTask,
    },
    AskVeto {
        chancellor: PlayerId,
        laws: [Law; 2],
    },
}

impl GameState {
    pub fn new(users: Vec<User>) -> GameState {
        GameState {
            cfg: Config {
                id: "001".to_string(),
                no_votes: false,
            },
            board: BoardState::new(users),
            state: DynamicState::Uninit,
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }

    #[instrument(name = "game", fields(id = %self.cfg.id), skip(self, user))]
    pub fn add_new_user(&mut self, user: User) -> Result<Player, String> {
        let player = Player::new(user);
        if self.board.players.iter().any(|p| p.id == player.id) {
            return Err("Player already exists".to_string());
        }

        tracing::info!("new player {:?} joined", player.user);
        self.board.players.push(player.clone());
        self.notify.notify_waiters();
        Ok(player)
    }

    pub fn join_existing_user(&mut self, auth_token: String) -> Result<Player, String> {
        let Some(player) = self
            .board
            .players
            .iter()
            .find(|p| p.access_key == auth_token)
        else {
            return Err("No such auth token".to_string());
        };

        if player.connected {
            return Err("Already connected".to_string());
        }

        tracing::info!("reconnected player {:?}", player.user);
        self.notify.notify_waiters();
        Ok(player.clone())
    }

    #[instrument(name = "game", fields(id = %self.cfg.id), skip(self, user))]
    pub fn remove_player(&mut self, user: &PlayerId) {
        if let Some(index) = self.board.players.iter().position(|p| &p.id == user) {
            tracing::info!("player {:?} removed", user);

            if self.state == DynamicState::Uninit {
                self.board.players.remove(index);
                self.notify.notify_waiters();
            } else {
                self.board.players[index].connected = false;
                self.notify.notify_waiters()
            }
        }
    }
}

impl DynamicState {
    pub fn is_player_elect(&self, player_id: &PlayerId) -> bool {
        use DynamicState::*;
        match self {
            PresidentChooseLaws { chancellor, .. }
            | ChancellorChooseLaws { chancellor, .. }
            | AskVeto { chancellor, .. } => player_id == chancellor,
            _ => false,
        }
    }
}

impl BoardState {
    pub fn index(&self, id: &PlayerId) -> usize {
        self.players.iter().position(|p| &p.id == id).unwrap()
    }

    pub fn players_alive(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().filter(|p| p.alive)
    }

    pub fn new(users: Vec<User>) -> Self {
        Self {
            players: users.into_iter().map(|u| Player::new(u)).collect(),

            draw_pile: Vec::new(),
            discard_pile: Vec::new(),

            executive_actions: [None; 6],
            voting_result: None,

            passed_fasho_laws: 0,
            passed_liberal_laws: 0,

            no_goverment_counter: 0,

            previous_president: None,
            previous_chancellor: None,

            current_president: String::new(),
            next_president_by_rules: None,
            revealed_factions: None,
            history: Vec::new(),
        }
    }

    pub fn draw_laws<const N: usize>(&mut self) -> [Law; N] {
        if self.draw_pile.len() < N {
            self.draw_pile.append(&mut self.discard_pile);
            self.draw_pile.shuffle(&mut thread_rng());
        }
        let mut result = [Law::Fasho; N];
        result.copy_from_slice(&self.draw_pile[..3]);
        for i in 0..N {
            self.draw_pile.remove(0);
        }
        result
    }

    pub fn play_law(&mut self, law: Law, chancellor: Option<PlayerId>) -> DynamicState {
        self.history.push(Event::PlayedLaw {
            president: self.current_president.clone(),
            chancellor: chancellor.clone(),
            law,
        });

        match law {
            Law::Liberal => {
                self.passed_liberal_laws += 1;
                self.select_next_president()
            }
            Law::Fasho => {
                self.passed_fasho_laws += 1;
                if let Some(chancellor) = chancellor {
                    let executive_action = self.executive_actions[self.passed_fasho_laws - 1];
                    if let Some(executive_action) = executive_action {
                        let action = match executive_action {
                            ExecutiveAction::RevealFaction => ExecutiveActionTask::RevealFaction,
                            ExecutiveAction::RevealNextCards => {
                                ExecutiveActionTask::RevealNextCards(
                                    self.draw_pile[..3.min(self.draw_pile.len())].to_vec(),
                                )
                            }
                            ExecutiveAction::DeterminePresident => {
                                ExecutiveActionTask::DeterminePresident
                            }
                            ExecutiveAction::Kill => ExecutiveActionTask::Kill,
                        };

                        DynamicState::ExecutiveAction { action, chancellor }
                    } else {
                        self.select_next_president()
                    }
                } else {
                    self.no_goverment_counter = 0;
                    self.select_next_president()
                }
            }
        }
    }

    pub fn select_next_president(&mut self) -> DynamicState {
        let president = self.determine_next_president_in_line();
        self.select_president(president)
    }

    pub fn select_president(&mut self, president: PlayerId) -> DynamicState {
        self.current_president = president;
        let options = self
            .players_alive()
            .map(|p| p.id.clone())
            .filter(|id| *id != self.current_president)
            .filter(|id| {
                Some(id) != self.previous_president.as_ref()
                    && Some(id) != self.previous_chancellor.as_ref()
            })
            .collect::<Vec<_>>();

        DynamicState::ChooseChancellor { options }
    }

    pub fn determine_next_president_in_line(&mut self) -> PlayerId {
        if let Some(next) = self.next_president_by_rules.take() {
            return next;
        }

        let mut idx = self.index(&self.current_president) + 1;
        while !self.players[idx % self.players.len()].alive {
            idx += 1;
        }
        self.players[idx % self.players.len()].id.clone()
    }
}

impl GameState {
    pub fn start(&mut self) {
        let player_count = self.board.players.len();
        let roles = Role::roles_for(player_count);
        for (player, role) in self.board.players.iter_mut().zip(roles.into_iter()) {
            player.role = role;
        }

        self.board.executive_actions = ExecutiveAction::for_player_count(player_count);

        self.board.draw_pile = Law::full_draw_pile();
        self.board.discard_pile = Vec::new();

        self.board.passed_fasho_laws = 0;
        self.board.passed_liberal_laws = 0;

        self.board.no_goverment_counter = 0;

        self.board.previous_president = None;
        self.board.previous_president = None;

        self.board.history = Vec::new();

        let idx = random::<usize>() % self.board.players.len();
        self.board.current_president = self.board.players[idx].id.clone();
        self.board.next_president_by_rules = None;

        // TODO: initiate first action
        let ids = self
            .board
            .players
            .iter()
            .map(|p| p.id.clone())
            .filter(|id| *id != self.board.current_president)
            .collect::<Vec<_>>();
        self.state = DynamicState::ChooseChancellor { options: ids };
    }

    pub fn check_win_conditions(&self) -> Option<Win> {
        if self.board.passed_fasho_laws == 6 {
            return Some(Win::Fasho);
        }
        if self.board.passed_liberal_laws == 5 {
            return Some(Win::Liberal);
        }

        if let Some(hitler) = self
            .board
            .players_alive()
            .find(|p| p.role == Role::FashoHitler)
        {
            // TODO
            if self.board.passed_fasho_laws >= 3 && self.state.is_player_elect(&hitler.id) {
                Some(Win::Fasho)
            } else {
                None
            }
        } else {
            Some(Win::Liberal)
        }
    }

    pub fn tasks(&self, player_id: &PlayerId) -> Option<Task> {
        match &self.state {
            DynamicState::Uninit => None,
            DynamicState::ChooseChancellor { options } => (*player_id
                == self.board.current_president)
                .then(|| Task::ChooseChancellor(options.clone())),
            DynamicState::VoteChancellor {
                chancellor: canidate,
                votes,
            } => votes.get(player_id).unwrap().is_none().then(|| {
                Task::Vote(VotingProposal {
                    president: self.board.current_president.clone(),
                    chancellor: canidate.clone(),
                })
            }),
            DynamicState::PresidentChooseLaws { laws, .. } => (*player_id
                == self.board.current_president)
                .then(|| Task::PickLaws(laws.to_vec(), false)),
            DynamicState::ChancellorChooseLaws {
                laws,
                chancellor,
                can_ask_veto,
            } => (player_id == chancellor).then(|| Task::PickLaws(laws.to_vec(), *can_ask_veto)),

            DynamicState::AskVeto { .. } => {
                (*player_id == self.board.current_president).then(|| Task::ConfirmVeto)
            }

            DynamicState::ExecutiveAction { action, .. } => (*player_id
                == self.board.current_president)
                .then(|| Task::ExecutiveAction(action.clone())),
        }
    }

    #[instrument(name = "game", fields(id = %self.cfg.id), skip(self, action))]
    pub fn on_action(&mut self, player: PlayerId, action: TaskAction) {
        use TaskAction::*;
        let new_state = match (self.state.clone(), action) {
            (_, Start) => {
                self.start();
                tracing::info!("Staring game");
                self.notify.notify_waiters();
                return;
            }

            (DynamicState::ChooseChancellor { options }, ChooseChancellor(choosen)) => {
                assert!(options.contains(&choosen));

                // FIXME: Quick Hack to make debugging easier

                self.board.history.push(Event::ChooseChancellor {
                    president: self.board.current_president.clone(),
                    chancellor: choosen.clone(),
                });
                self.board.voting_result = None;

                if self.cfg.no_votes {
                    self.board.no_goverment_counter = 0;
                    let laws = self.board.draw_laws::<3>();
                    DynamicState::PresidentChooseLaws {
                        laws,
                        chancellor: choosen.clone(),
                    }
                } else {
                    let votes = self
                        .board
                        .players
                        .iter()
                        .map(|p| (p.id.clone(), None))
                        .collect();
                    DynamicState::VoteChancellor {
                        chancellor: choosen,
                        votes,
                    }
                }
            }
            (
                DynamicState::VoteChancellor {
                    chancellor,
                    mut votes,
                },
                Vote(agree),
            ) => {
                // let idx = self.board.index(&player);
                // votes[idx] = Some(agree);
                votes.insert(player, Some(agree));

                if votes.values().filter(|v| v.is_some()).count()
                    == self.board.players_alive().count()
                {
                    let pros = votes.values().filter(|v| **v == Some(true)).count();
                    let cons = votes.values().filter(|v| **v == Some(false)).count();
                    let presidents_vote =
                        votes.get(&self.board.current_president).unwrap().unwrap();

                    let vote_success = pros > cons || (pros == cons && presidents_vote);

                    self.board.history.push(Event::Vote {
                        president: self.board.current_president.clone(),
                        chancellor: chancellor.clone(),
                        votes: votes
                            .iter()
                            .flat_map(|(k, v)| Some((k.clone(), (*v)?)))
                            .collect(),
                        success: vote_success,
                    });

                    self.board.voting_result =
                        Some(votes.into_iter().flat_map(|(k, v)| Some((k, v?))).collect());

                    if vote_success {
                        self.board.no_goverment_counter = 0;

                        let laws = self.board.draw_laws::<3>();

                        DynamicState::PresidentChooseLaws {
                            laws,
                            chancellor: chancellor.clone(),
                        }
                    } else {
                        // Vote failed
                        self.board.no_goverment_counter += 1;
                        if self.board.no_goverment_counter == 3 {
                            // Goverment failed: autoplay card
                            let law = self.board.draw_laws::<1>()[0];
                            self.board.play_law(law, None)
                        } else {
                            self.board.select_next_president()
                        }
                    }
                } else {
                    DynamicState::VoteChancellor { chancellor, votes }
                }
            }

            (DynamicState::PresidentChooseLaws { chancellor, .. }, PickedLaws(keep, discard)) => {
                // Assert true
                self.board.discard_pile.push(discard);
                self.board.voting_result = None;
                DynamicState::ChancellorChooseLaws {
                    laws: keep.try_into().unwrap(),
                    can_ask_veto: self.board.passed_fasho_laws == 5,
                    chancellor,
                }
            }

            (DynamicState::ChancellorChooseLaws { chancellor, .. }, PickedLaws(keep, discard)) => {
                // Assert true
                assert_eq!(keep.len(), 1);
                self.board.discard_pile.push(discard);

                self.board.previous_president = Some(self.board.current_president.clone());
                self.board.previous_chancellor = Some(chancellor.clone());

                self.board.play_law(keep[0], Some(chancellor))
            }

            (
                DynamicState::ChancellorChooseLaws {
                    chancellor,
                    laws,
                    can_ask_veto,
                },
                Veto(true),
            ) if can_ask_veto => DynamicState::AskVeto {
                chancellor: chancellor.clone(),
                laws,
            },

            (DynamicState::AskVeto { chancellor, laws }, Veto(accept)) => {
                if accept {
                    self.board.history.push(Event::Veto {
                        president: self.board.current_president.clone(),
                        chancellor: chancellor.clone(),
                    });

                    self.board.discard_pile.extend_from_slice(&laws);

                    self.board.previous_president = Some(self.board.current_president.clone());
                    self.board.previous_chancellor = Some(chancellor.clone());

                    self.board.select_next_president()
                } else {
                    DynamicState::ChancellorChooseLaws {
                        can_ask_veto: false,
                        laws,
                        chancellor,
                    }
                }
            }

            (
                DynamicState::ExecutiveAction {
                    action: ExecutiveActionTask::Kill,
                    ..
                },
                ExecuteAction(ExecutiveActionResponse::Kill(target)),
            ) => {
                let idx = self.board.index(&target);
                self.board.players[idx].alive = false;
                self.board.select_next_president()
            }

            (
                DynamicState::ExecutiveAction {
                    action: ExecutiveActionTask::RevealFaction,
                    ..
                },
                ExecuteAction(ExecutiveActionResponse::RevealFaction(target)),
            ) => {
                let idx = self.board.index(&target);
                let faction = self.board.players[idx].role.faction();
                self.board.revealed_factions =
                    Some((self.board.current_president.clone(), target, faction));
                self.board.select_next_president()
            }

            (
                DynamicState::ExecutiveAction {
                    action: ExecutiveActionTask::DeterminePresident,
                    ..
                },
                ExecuteAction(ExecutiveActionResponse::DeterminePresident(target)),
            ) => {
                self.board.next_president_by_rules =
                    Some(self.board.determine_next_president_in_line());
                self.board.select_president(target)
            }

            (
                DynamicState::ExecutiveAction {
                    action: ExecutiveActionTask::RevealNextCards(_),
                    ..
                },
                ExecuteAction(ExecutiveActionResponse::RevealNextCards),
            ) => self.board.select_next_president(),

            (_, action) => todo!("unknown state and action combo: {:?} {:?}", self, action),
        };

        let has_changed = new_state != self.state;
        self.state = new_state;
        if has_changed {
            tracing::info!("State: {:?}", self.state);
            self.notify.notify_waiters();
        }
    }
}
