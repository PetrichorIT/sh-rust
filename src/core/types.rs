use rand::distributions::Alphanumeric;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::{once, repeat};

pub type PlayerId = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub image: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub user: User,

    pub role: Role,
    pub alive: bool,

    pub connected: bool,
    pub access_key: String,
}

impl Player {
    pub fn new(user: User) -> Self {
        Player {
            id: user.name.clone(),
            user,
            role: Role::Liberal,
            alive: true,

            connected: true,
            access_key: thread_rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect::<String>(),
        }
    }
}

pub type Law = Faction;

impl Law {
    pub fn full_draw_pile() -> Vec<Law> {
        let liberals = repeat(Law::Liberal).take(6);
        let fashos = repeat(Law::Fasho).take(9);
        let mut pile = liberals.chain(fashos).collect::<Vec<_>>();
        pile.shuffle(&mut thread_rng());
        pile
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Liberal,
    Fasho,
    FashoHitler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Faction {
    Fasho,
    Liberal,
}

impl Role {
    pub fn roles_for(n: usize) -> Vec<Role> {
        let fashos = ((n - 5) / 2) + 1;
        let mut pile = once(Role::FashoHitler)
            .chain(repeat(Role::Fasho).take(fashos))
            .chain(repeat(Role::Liberal).take(n - fashos - 1))
            .collect::<Vec<_>>();
        pile.shuffle(&mut thread_rng());
        pile
    }

    pub fn faction(&self) -> Faction {
        match self {
            Self::Liberal => Faction::Liberal,
            _ => Faction::Fasho,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Task {
    ChooseChancellor(Vec<PlayerId>),
    Vote(VotingProposal),
    PickLaws(Vec<Law>, bool),
    ExecutiveAction(ExecutiveActionTask),
    ConfirmVeto,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VotingProposal {
    pub president: PlayerId,
    pub chancellor: PlayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TaskAction {
    Start,
    ChooseChancellor(PlayerId),
    Vote(bool),
    PickedLaws(Vec<Law>, Law),
    Veto(bool),
    ExecuteAction(ExecutiveActionResponse),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutiveAction {
    Kill,
    RevealFaction,
    DeterminePresident,
    RevealNextCards,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ExecutiveActionTask {
    Kill,
    RevealFaction,
    DeterminePresident,
    RevealNextCards(Vec<Law>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ExecutiveActionResponse {
    Kill(PlayerId),
    RevealFaction(PlayerId),
    DeterminePresident(PlayerId),
    RevealNextCards,
}

impl ExecutiveAction {
    pub fn for_player_count(count: usize) -> [Option<ExecutiveAction>; 6] {
        use ExecutiveAction::*;
        match count {
            5 | 6 => [
                None,
                None,
                Some(RevealNextCards),
                Some(Kill),
                Some(Kill),
                None,
            ],
            7 | 8 => [
                None,
                Some(RevealFaction),
                Some(DeterminePresident),
                Some(Kill),
                Some(Kill),
                None,
            ],
            9 | 10 => [
                Some(RevealFaction),
                Some(RevealFaction),
                Some(DeterminePresident),
                Some(Kill),
                Some(Kill),
                None,
            ],
            _ => unreachable!(),
        }
    }
}

pub type Win = Faction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    ChooseChancellor {
        president: PlayerId,
        chancellor: PlayerId,
    },
    Vote {
        president: PlayerId,
        chancellor: PlayerId,
        votes: HashMap<PlayerId, bool>,
        success: bool,
    },
    PlayedLaw {
        president: PlayerId,
        chancellor: Option<PlayerId>,
        law: Law,
    },
    Veto {
        president: PlayerId,
        chancellor: PlayerId,
    },
}
