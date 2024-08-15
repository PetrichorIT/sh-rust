use rand::random;
use serde::{Deserialize, Serialize};

use crate::core::types::VotingProposal;

use super::types::{ExecutiveAction, Law, Player, PlayerId, Role, Task, TaskAction, User};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameState {
    players: Vec<Player>,

    draw_pile: Vec<Law>,
    discard_pile: Vec<Law>,

    executive_actions: [Option<ExecutiveAction>; 6],

    passed_fasho_laws: usize,
    passed_liberal_laws: usize,

    no_goverment_counter: usize,

    previous_president: Option<PlayerId>,
    previous_chancellor: Option<PlayerId>,

    current_president: Option<PlayerId>,
    current_chancellor: Option<PlayerId>,
    current_chancellor_canidate: Option<PlayerId>,

    state: Option<(PlayerId, Task)>,
}

impl GameState {
    pub fn player(&self, id: &Option<PlayerId>) -> &Player {
        self.players.iter().find(|p| p.id == id.unwrap()).unwrap()
    }

    pub fn player_mut(&mut self, id: &Option<PlayerId>) -> &mut Player {
        self.players
            .iter_mut()
            .find(|p| p.id == id.unwrap())
            .unwrap()
    }

    pub fn new(users: Vec<User>) -> Self {
        Self {
            players: users.into_iter().map(|u| Player::new(u)).collect(),

            draw_pile: Vec::new(),
            discard_pile: Vec::new(),

            executive_actions: [None; 6],

            passed_fasho_laws: 0,
            passed_liberal_laws: 0,

            no_goverment_counter: 0,

            previous_president: None,
            previous_chancellor: None,

            current_president: None,
            current_chancellor: None,
            current_chancellor_canidate: None,

            state: None,
        }
    }

    pub fn start(&mut self) {
        let player_count = self.players.len();
        let roles = Role::roles_for(player_count);
        for (player, role) in self.players.iter_mut().zip(roles.into_iter()) {
            player.role = role;
        }

        self.draw_pile = Law::full_draw_pile();
        self.discard_pile = Vec::new();

        self.passed_fasho_laws = 0;
        self.passed_liberal_laws = 0;

        self.no_goverment_counter = 0;

        self.previous_president = None;
        self.previous_president = None;

        let idx = random::<usize>() % self.players.len();
        self.current_president = Some(self.players[idx].id.clone());
        self.current_chancellor = None;
        self.current_chancellor_canidate;

        // TODO: initiate first action
        let ids = self
            .players
            .iter()
            .map(|p| p.id.clone())
            .filter(|id| *id != self.current_president.unwrap())
            .collect();
        self.players[idx].pending_task = Some(Task::ChooseChancellor(ids));
    }

    pub fn on_action(&mut self, action: TaskAction) {
        let state = self.state.expect("no task set yet");

        use TaskAction::*;
        match (state.1, action) {
            (Task::ChooseChancellor(options), ChooseChancellor(choosen)) => {
                assert!(options.contains(&choosen));
                self.current_chancellor_canidate = Some(choosen);
                self.player_mut(&self.current_president).pending_task.take();
                self.players.iter_mut().for_each(|player| {
                    player.pending_task = Some(Task::Vote(VotingProposal {
                        president: self.current_president.unwrap(),
                        chancellor: choosen,
                    }))
                })
            }
            _ => todo!(),
        }
    }
}
