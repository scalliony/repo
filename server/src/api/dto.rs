use crate::game::{self, Event as Ev};
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "t", content = "p")]
pub enum View {
    None,
    All,
}
impl View {
    pub fn contains(&self, event: &Ev) -> bool {
        match event {
            Ev::State { .. } | Ev::TickStart { .. } | Ev::TickEnd => true,
            Ev::Log { .. } | Ev::Error { .. } => match *self {
                View::None => false,
                View::All => true,
            },
        }
    }
}
impl Default for View {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "k")]
pub enum Event {
    State {
        state: game::State,
    },
    TickStart {
        id: game::TickId,
        /// seconds since unix epoch
        ts: game::Timestamp,
    },
    TickEnd,
    Log {
        bot: game::BotId,
        msg: game::Str,
    },
    Error {
        bot: game::BotId,
        #[serde(flatten)]
        err: game::Error,
    },
}
impl From<Ev> for Event {
    fn from(value: Ev) -> Event {
        match value {
            Ev::State(state) => Event::State { state },
            Ev::TickStart(id, ts) => Event::TickStart { id, ts },
            Ev::TickEnd => Event::TickEnd,
            Ev::Log(src, msg) => Event::Log { bot: src.id, msg },
            Ev::Error(src, err) => Event::Error { bot: src.id, err },
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "k")]
pub enum Command {
    View {
        #[serde(flatten)]
        v: View,
    },
    Spawn {
        #[serde(flatten)]
        q: SpawnBody,
    },
    State {
        state: game::State,
    },
}

#[derive(Deserialize, Clone, Debug)]
pub struct SpawnBody {
    pub program: game::ProgramId,
}

#[derive(Deserialize, Debug)]
pub struct SseQuery {
    #[serde(default)]
    pub view: View,
}
