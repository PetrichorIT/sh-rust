use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use super::{
    types::{ExecutiveAction, Faction, Player, PlayerId, Role, User},
    BoardState, DynamicState, Event, ExecutiveActionTask, GameState,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameView {
    board: BoardStateView,
    state: StateView,
    me: PlayerView,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum StateView {
    Uninit,
    ChooseChancellor {
        options: Vec<PlayerId>,
    },
    VoteChancellor {
        canidate: PlayerId,
    },
    PresidentChooseLaws {
        chancellor: PlayerId,
    },
    ChancellorChooseLaws {
        can_ask_veto: bool,
        chancellor: PlayerId,
    },
    ExecutiveAction {
        chancellor: PlayerId,
        action: ExecutiveActionTask,
    },
    AskVeto {
        chancellor: PlayerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardStateView {
    players: Vec<PlayerView>,

    draw_pile: usize,
    discard_pile: usize,

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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerView {
    id: PlayerId,
    user: User,
    alive: bool,
    role: Option<Role>,
    faction: Option<Faction>,
}

impl GameState {
    pub fn view(&self, of: &PlayerId) -> Option<GameView> {
        let player = self.board.players.iter().find(|p| &p.id == of)?;
        Some(GameView {
            board: self.board.view(player),
            state: self.state.view(player),
            me: player.view(&player, false),
        })
    }
}

impl DynamicState {
    pub fn view(&self, of: &Player) -> StateView {
        match self {
            Self::Uninit => StateView::Uninit,
            Self::ChooseChancellor { options } => StateView::ChooseChancellor {
                options: options.clone(),
            },
            Self::VoteChancellor { canidate, .. } => StateView::VoteChancellor {
                canidate: canidate.clone(),
            },
            Self::PresidentChooseLaws { chancellor, .. } => StateView::PresidentChooseLaws {
                chancellor: chancellor.clone(),
            },
            Self::ChancellorChooseLaws {
                chancellor,
                can_ask_veto,
                ..
            } => StateView::ChancellorChooseLaws {
                chancellor: chancellor.clone(),
                can_ask_veto: *can_ask_veto,
            },
            Self::AskVeto { chancellor, .. } => StateView::AskVeto {
                chancellor: chancellor.clone(),
            },

            Self::ExecutiveAction { chancellor, action } => StateView::ExecutiveAction {
                chancellor: chancellor.clone(),
                action: action.clone(),
            },
        }
    }
}

impl BoardState {
    pub fn view(&self, of: &Player) -> BoardStateView {
        let mut view = BoardStateView {
            players: self
                .players
                .iter()
                .map(|p| p.view(of, self.players.len() < 7))
                .collect(),

            draw_pile: self.draw_pile.len(),
            discard_pile: self.discard_pile.len(),

            executive_actions: self.executive_actions,
            voting_result: self.voting_result.clone(),

            passed_fasho_laws: self.passed_fasho_laws,
            passed_liberal_laws: self.passed_liberal_laws,

            no_goverment_counter: self.no_goverment_counter,

            previous_president: self.previous_president.clone(),
            previous_chancellor: self.previous_chancellor.clone(),

            current_president: self.current_president.clone(),
            next_president_by_rules: self.next_president_by_rules.clone(),
            history: self.history.clone(),
        };
        if let Some((knower, target, target_faction)) = &self.revealed_factions {
            if *knower == of.id {
                let view = view.players.iter_mut().find(|p| &p.id == target).unwrap();
                view.faction = Some(*target_faction);
            }
        }
        view
    }
}

impl Player {
    pub fn view(&self, of: &Player, less_than_seven: bool) -> PlayerView {
        let is_self = of.id == self.id;
        let alt_knowledge = of.role == Role::FashoHitler && less_than_seven;
        PlayerView {
            id: self.id.clone(),
            user: self.user.clone(),
            alive: self.alive,
            role: (is_self || alt_knowledge || of.role == Role::Fasho).then_some(self.role),
            faction: (is_self || alt_knowledge || of.role == Role::Fasho)
                .then_some(self.role.faction()),
        }
    }
}
