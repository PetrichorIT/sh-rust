export interface GameStateView {
  board: BoardStateView;
  state: State;
  me: PlayerView;
}

export interface BoardStateView {
  players: PlayerView[];

  draw_pile: number;
  discard_pile: number;

  executive_actions: [
    null | ExecutiveAction,
    null | ExecutiveAction,
    null | ExecutiveAction,
    null | ExecutiveAction,
    null | ExecutiveAction,
    null | ExecutiveAction,
  ];
  voting_result: null | { [key: string]: boolean };

  passed_fasho_laws: number;
  passed_liberal_laws: number;

  no_goverment_counter: number;

  previous_president: null | PlayerId;
  previous_chancellor: null | PlayerId;

  current_president: PlayerId;
  next_president_by_rules: null | PlayerId;

  history: Event[];
}

export interface State {
  type: string;
  value: any;
}

export type Law = Faction;
export type Faction = "Fasho" | "Liberal";
export type Role = "FashoHitler" | "Fasho" | "Liberal";
export type Position = "Persident" | "Chancellor";

export interface PlayerView {
  id: PlayerId;
  user: User;
  alive: boolean;

  role: null | Role;
  faction: null | Faction;

  hasTask: boolean;
  access_key: string;
}

export type PlayerId = string;

export interface User {
  name: string;
  image: string;
  color: string;
}

export interface Task {
  type: string;
  value: ChooseChancellorTask | VoteTask | PickLawsTask | ExecutiveActionTask;
}

export type ChooseChancellorTask = PlayerId[];

export interface VoteTask {
  president: PlayerId;
  chancellor: PlayerId;
}

export type PickLawsTask = [Law[], boolean];

export type ExecutiveActionTask = ExecutiveAction;

type ExecutiveAction = any;

export interface TaskAction {
  type: string;
  value: any;
}

export interface AuthenticateMessage {
  user: null | User;
  access_key: null | string;
}

export interface Event {
  ChooseChancellor?: {
    president: PlayerId;
    chancellor: PlayerId;
  };
  Vote?: {
    president: PlayerId;
    chancellor: PlayerId;
    votes: { [key: PlayerId]: boolean };
    success: boolean;
  };
  PlayedLaw?: {
    president: PlayerId;
    chancellor: null | PlayerId;
    law: Law;
  };
  Veto?: {
    president: PlayerId;
    chancellor: PlayerId;
  };
}
