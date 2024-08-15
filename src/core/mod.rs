mod types;

use std::collections::HashSet;

use rand::prelude::SliceRandom;
use rand::random;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use types::ExecutiveActionResponse;
use types::Win;

use crate::core::types::VotingProposal;

use types::{ExecutiveAction, Law, Player, PlayerId, Role, Task, TaskAction, User};

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

    current_president: PlayerId,
    next_president_by_rules: Option<PlayerId>,

    state: Option<State>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum State {
    Uninit,
    ChooseChancellor {
        options: Vec<PlayerId>,
    },
    VoteChancellor {
        canidate: PlayerId,
        votes: Vec<Option<bool>>,
    },
    PresidentChooseLaws {
        laws: Vec<Law>,
        chancellor: PlayerId,
    },
    ChancellorChooseLaws {
        can_ask_veto: bool,
        laws: Vec<Law>,
        chancellor: PlayerId,
    },
    ExecutiveAction {
        chancellor: PlayerId,
        action: ExecutiveAction,
    },
    AskVeto {
        chancellor: PlayerId,
        laws: Vec<Law>,
    },
}

impl State {
    pub fn is_player_elect(&self, player_id: &PlayerId) -> bool {
        use State::*;
        match self {
            PresidentChooseLaws { chancellor, .. }
            | ChancellorChooseLaws { chancellor, .. }
            | AskVeto { chancellor, .. } => player_id == chancellor,
            _ => false,
        }
    }
}

impl GameState {
    pub fn index(&self, id: &PlayerId) -> usize {
        self.players.iter().position(|p| &p.id == id).unwrap()
    }

    pub fn players_alive(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().filter(|p| p.alive)
    }

    pub fn player_mut(&mut self, id: &PlayerId) -> &mut Player {
        self.players.iter_mut().find(|p| &p.id == id).unwrap()
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

            current_president: String::new(),
            next_president_by_rules: None,

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
        self.current_president = self.players[idx].id.clone();
        self.next_president_by_rules = None;

        // TODO: initiate first action
        let ids = self
            .players
            .iter()
            .map(|p| p.id.clone())
            .filter(|id| *id != self.current_president)
            .collect::<Vec<_>>();
        self.players[idx].pending_task = Some(Task::ChooseChancellor(ids.clone()));
        self.state = Some(State::ChooseChancellor { options: ids })
    }

    pub fn draw_laws(&mut self, n: usize) -> Vec<Law> {
        if self.draw_pile.len() < n {
            self.draw_pile.append(&mut self.discard_pile);
            self.draw_pile.shuffle(&mut thread_rng());
        }
        self.draw_pile.drain(..n).collect()
    }

    pub fn check_win_conditions(&self) -> Option<Win> {
        if self.passed_fasho_laws == 6 {
            return Some(Win::Fasho);
        }
        if self.passed_liberal_laws == 5 {
            return Some(Win::Liberal);
        }

        if let Some(hitler) = self.players_alive().find(|p| p.role == Role::FashoHitler) {
            if self.passed_fasho_laws >= 3
                && self.state.as_ref().unwrap().is_player_elect(&hitler.id)
            {
                Some(Win::Fasho)
            } else {
                None
            }
        } else {
            Some(Win::Liberal)
        }
    }

    pub fn play_law(&mut self, law: Law, chancellor: Option<PlayerId>) -> State {
        match law {
            Law::Liberal => {
                self.passed_liberal_laws += 1;
                self.select_next_president()
            }
            Law::Fasho => {
                self.passed_fasho_laws += 1;
                if let Some(chancellor) = chancellor {
                    let executive_action = self.executive_actions[self.passed_fasho_laws];
                    if let Some(executive_action) = executive_action {
                        State::ExecutiveAction {
                            action: executive_action,
                            chancellor,
                        }
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

    pub fn on_action(&mut self, player: PlayerId, action: TaskAction) {
        use TaskAction::*;
        self.state = Some(match (self.state.take().unwrap(), action) {
            (State::ChooseChancellor { options }, ChooseChancellor(choosen)) => {
                assert!(options.contains(&choosen));

                State::VoteChancellor {
                    canidate: choosen,
                    votes: vec![None; self.players.len()],
                }
            }
            (
                State::VoteChancellor {
                    canidate,
                    mut votes,
                },
                Vote(agree),
            ) => {
                let idx = self.index(&player);
                votes[idx] = Some(agree);

                if votes.iter().filter(|v| v.is_some()).count() == self.players_alive().count() {
                    let pros = votes.iter().filter(|v| **v == Some(true)).count();
                    let cons = votes.iter().filter(|v| **v == Some(false)).count();
                    let presidents_vote = votes[self.index(&self.current_president)].unwrap();

                    if pros > cons || (pros == cons && presidents_vote) {
                        self.no_goverment_counter = 0;

                        let laws = self.draw_laws(3);

                        State::PresidentChooseLaws {
                            laws,
                            chancellor: canidate.clone(),
                        }
                    } else {
                        // Vote failed
                        self.no_goverment_counter += 1;
                        if self.no_goverment_counter == 3 {
                            // Goverment failed: autoplay card
                            let law = self.draw_laws(1)[0];
                            self.play_law(law, None)
                        } else {
                            self.select_next_president()
                        }
                    }
                } else {
                    State::VoteChancellor { canidate, votes }
                }
            }

            (State::PresidentChooseLaws { chancellor, .. }, PickedLaws(keep, discard)) => {
                // Assert true
                self.discard_pile.push(discard);
                State::ChancellorChooseLaws {
                    laws: keep,
                    can_ask_veto: self.passed_fasho_laws == 5,
                    chancellor,
                }
            }

            (State::ChancellorChooseLaws { chancellor, .. }, PickedLaws(keep, discard)) => {
                // Assert true
                assert_eq!(keep.len(), 1);
                self.discard_pile.push(discard);

                self.previous_president = Some(self.current_president.clone());
                self.previous_chancellor = Some(chancellor.clone());

                self.play_law(keep[0], Some(chancellor))
            }

            (
                State::ChancellorChooseLaws {
                    chancellor,
                    laws,
                    can_ask_veto,
                },
                Veto(true),
            ) if can_ask_veto => State::AskVeto {
                chancellor: chancellor.clone(),
                laws,
            },

            (
                State::AskVeto {
                    chancellor,
                    mut laws,
                },
                Veto(accept),
            ) => {
                if accept {
                    self.discard_pile.append(&mut laws);

                    self.previous_president = Some(self.current_president.clone());
                    self.previous_chancellor = Some(chancellor.clone());

                    self.select_next_president()
                } else {
                    State::ChancellorChooseLaws {
                        can_ask_veto: false,
                        laws,
                        chancellor,
                    }
                }
            }

            (
                State::ExecutiveAction {
                    action: ExecutiveAction::Kill,
                    ..
                },
                ExecuteAction(ExecutiveActionResponse::Kill(target)),
            ) => {
                let idx = self.index(&target);
                self.players[idx].alive = false;
                self.select_next_president()
            }

            (
                State::ExecutiveAction {
                    action: ExecutiveAction::RevealFaction,
                    ..
                },
                ExecuteAction(ExecutiveActionResponse::RevealFaction(target)),
            ) => {
                let idx = self.index(&target);
                let faction = self.players[idx].role.faction();
                // TODO: propagate information
                self.select_next_president()
            }

            (
                State::ExecutiveAction {
                    action: ExecutiveAction::DeterminePresident,
                    ..
                },
                ExecuteAction(ExecutiveActionResponse::DeterminePresident(target)),
            ) => {
                self.next_president_by_rules = Some(self.determine_next_president_in_line());
                self.select_president(target)
            }

            (
                State::ExecutiveAction {
                    action: ExecutiveAction::RevealNextCards,
                    ..
                },
                ExecuteAction(ExecutiveActionResponse::RevealNextCards()),
            ) => self.select_next_president(),

            _ => todo!(),
        });
    }

    pub fn select_next_president(&mut self) -> State {
        let president = self.determine_next_president_in_line();
        self.select_president(president)
    }

    pub fn select_president(&mut self, president: PlayerId) -> State {
        let options = self
            .players_alive()
            .map(|p| p.id.clone())
            .filter(|id| *id == self.current_president)
            .filter(|id| {
                Some(id) == self.previous_president.as_ref()
                    || Some(id) == self.previous_chancellor.as_ref()
            })
            .collect::<Vec<_>>();

        let idx = self.index(&president);
        self.current_president = president;
        self.players[idx].pending_task = Some(Task::ChooseChancellor(options.clone()));
        State::ChooseChancellor { options }
    }

    pub fn determine_next_president_in_line(&mut self) -> PlayerId {
        if let Some(next) = self.next_president_by_rules.take() {
            return next;
        }

        let mut idx = self.index(&self.current_president) + 1;
        while !self.players[idx].alive {
            idx += 1;
        }
        self.players[idx].id.clone()
    }
}
