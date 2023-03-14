use std::fmt::Debug;

use super::gen;
use bulb::{
    dto::{BotId, BotSrc, Cell, CellMap, ProgramId},
    hex::{Direction, Hex},
};
use sys::wasm::{self, spec::StoreRef};
use tracing::instrument;

pub struct Bot {
    pub program: ProgramId,
    pub cpu: Result<Cpu, StateOff>,
}
impl Bot {
    pub fn at(&self) -> Hex {
        match &self.cpu {
            Ok(cpu) => cpu.state().at,
            Err(off) => off.at,
        }
    }
    pub fn src(&self, bid: BotId) -> BotSrc {
        match &self.cpu {
            Ok(cpu) => cpu.state().src(),
            Err(off) => BotSrc { bid, at: off.at },
        }
    }
}

pub struct Cpu {
    pub process: wasm::Instance<Store>,
    tick: wasm::Func<(), ()>,
}
impl Cpu {
    #[cold]
    #[inline]
    #[instrument(level = "trace", skip_all)]
    pub fn boot(
        tpl: &mut Template,
        state: State,
        fuel: u64,
    ) -> Result<Self, (String, wasm::Error)> {
        let (mut process, res) = wasm::Instance::started(tpl, state, fuel);
        if let Err(err) = res {
            return Err((process.store_mut().read_log(), err));
        }
        let tick = process.get_func::<(), ()>("tick").unwrap();
        Ok(Self { process, tick })
    }
    #[inline]
    pub fn tick(&mut self) -> Result<(), wasm::Error> {
        self.process.call(&self.tick, ())
    }
    #[inline]
    pub fn store(&self) -> &Store {
        self.process.store()
    }
    #[inline]
    pub fn store_mut(&mut self) -> &mut Store {
        self.process.store_mut()
    }
    #[inline]
    pub fn state(&self) -> &State {
        self.process.state()
    }
    #[inline]
    pub fn state_mut(&mut self) -> &mut State {
        self.process.state_mut()
    }
}

/// Booted bot state
pub struct State {
    /// Self id
    pub id: BotId,
    /// Position
    pub at: Hex,
    /// Front orientation
    pub facing: Direction,
    /// Cell in facing direction
    pub front: Cell,
    /// Next action intent
    pub action: Action,
}
pub type Store = wasm::WasiStore<State>;
impl State {
    pub fn boot(id: BotId, off: &StateOff) -> Self {
        Self {
            id,
            at: off.at,
            facing: off.facing,
            ..Default::default()
        }
    }
    pub fn shutdown(&self, fuel: u64) -> StateOff {
        StateOff {
            at: self.at,
            facing: self.facing,
            fuel,
        }
    }

    pub fn at_front(&self) -> Hex {
        self.at + self.facing.into()
    }
    pub fn src(&self) -> BotSrc {
        BotSrc {
            bid: self.id,
            at: self.at,
        }
    }

    pub fn update(&mut self, map: &impl CellMap) {
        self.action = Self::default().action;
        self.front = map.get(self.at_front());
    }
}
impl Default for State {
    //MAYBE: remove
    fn default() -> Self {
        Self {
            id: u64::MAX.into(),
            at: Hex::default(),
            facing: Direction::Up,
            front: Cell::Ground,
            action: Action::Wait,
        }
    }
}
/// Stopped bot state
pub struct StateOff {
    pub at: Hex,
    pub facing: Direction,
    pub fuel: u64,
}
impl Debug for StateOff {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("StateOff {{ ... }}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Wait,
    //Sleep(u32)
    MotorForward,
    MotorLeft,
    MotorRight,
}

pub type Template = wasm::Template<Store>;

impl From<gen::Id> for BotId {
    fn from(id: gen::Id) -> Self {
        Self::from(id.pack())
    }
}
impl From<BotId> for gen::Id {
    fn from(id: BotId) -> gen::Id {
        gen::Id::new(id.into())
    }
}
