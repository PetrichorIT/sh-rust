import { getStoredValue, setStoredValue } from "./storage";
import { GameStateView, Task, TaskAction, User } from "./types";

export type RenderFn = (
  state: GameStateView,
  task: null | Task,
  auth: boolean,
) => void;

export class Connection {
  ws: WebSocket;
  rerender: RenderFn;
  authenticated: boolean;

  state: null | GameStateView;
  task: null | Task;

  constructor(ws: WebSocket, rerender: RenderFn) {
    this.ws = ws;
    this.rerender = rerender;
    this.authenticated = true;

    this.state = null;
    this.task = null;

    this.ws.onopen = () => this.onOpen();
    this.ws.onmessage = (msg) => this.onMessage(msg);
  }

  onOpen() {
    const key = getStoredValue("sh.access-key");
    if (key !== null) {
      this.ws.send(
        JSON.stringify({
          Authenticate: {
            access_key: key,
          },
        }),
      );
    }
  }

  onMessage(msg: MessageEvent<string>) {
    const data = JSON.parse(msg.data);
    switch (data.type) {
      case "State":
        if (this.authenticated) {
          this.state = data.value.game_state;
          this.task = data.value.task;
          this.rerender(this.state!, this.task, this.authenticated);
        }
        break;
      case "Authenticated":
        this.authenticated = true;
        console.log("ac: " + data.value.access_key);
        setStoredValue("sh.access-key", data.value.access_key);
        this.rerender(this.state!, this.task, this.authenticated);
        break;
    }
  }

  login(user: User) {
    this.ws.send(
      JSON.stringify({
        Authenticate: {
          user,
        },
      }),
    );
  }

  send(action: TaskAction) {
    console.log(action);
    this.ws.send(
      JSON.stringify({
        Task: action,
      }),
    );
  }
}

export function connect(rerender: RenderFn, port?: string) {
  const usePort = port ?? window.location.port;
  const socketProtocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  const socketUrl =
    socketProtocol + "//" + window.location.hostname + ":" + usePort + "/ws";

  const socket = new WebSocket(socketUrl);

  return new Connection(socket, rerender);
}
