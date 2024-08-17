use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    pub id: String,
    pub no_votes: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            id: String::new(),
            no_votes: false,
        }
    }
}
