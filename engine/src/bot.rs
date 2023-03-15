use std::fmt::Debug;
use bulb::{
    dto::{BotSrc, Cell, ProgramId, CellGrid, UserId, Bytes, ObjId},
    hex::{Direction, Hex, Angle},
};
use sys::wasm::{self, spec::StoreRef};
use tracing::instrument;

pub struct Bot {
    process: wasm::Instance<Store>,
    tick: wasm::Func<(), ()>,
    program: ProgramId,
}
impl Bot {
    #[instrument(level = "trace", skip_all)]
    pub fn new(
        prg: &Program,
        pid: ProgramId, 
        state: State,
        fuel: u64,
    ) -> Result<Self, (String, wasm::Error)> {
        let (mut process, res) = wasm::Instance::started(&prg.tpl, state, fuel);
        if let Err(err) = res {
            return Err((process.store_mut().read_log(), err));
        }
        let tick = process.get_func::<(), ()>("tick").unwrap();
        Ok(Self { process, tick, program: pid })
    }

    #[inline]
    pub fn process(&mut self) -> &mut wasm::Instance<Store> {
        &mut self.process
    }
    #[inline]
    pub fn tick(&mut self) -> Result<(), wasm::Error> {
        self.process.call(&self.tick, ())
    }
    #[inline]
    pub fn program(&self) -> ProgramId {
        self.program
    }
}

/// Booted bot state
pub struct State {
    /// Self id
    id: ObjId,
    /// Owner UserId
    owner: UserId,
    /// Position
    at: Hex,
    /// Front orientation
    facing: Direction,
    /// Cell in facing direction
    front: Cell,
    /// Next action intent
    pub action: Action,
}
pub type Store = wasm::WasiStore<State>;
impl State {
    pub fn new(id: ObjId, owner: UserId, at: Hex, facing: Direction) -> Self {
        Self { id, owner, at, facing, front: Cell::Ground, action: Action::default() }
    }

    pub fn src(&self) -> BotSrc {
        BotSrc { bid: self.id, at: self.at, uid: self.owner }
    }
    #[inline]
    pub fn at_front(&self) -> Hex {
        self.at + self.facing.into()
    }
    #[inline]
    pub fn set_at(&mut self, at: Hex) {
        self.at = at
    }
    #[inline]
    pub fn facing(&self) -> Direction {
        self.facing
    }
    #[inline]
    pub fn turn(&mut self, dir: Angle) -> Direction {
        self.facing += dir;
        self.facing
    }
    #[inline]
    pub fn front(&self) -> Cell {
        self.front
    }

    pub fn update(&mut self, grid: &impl CellGrid) {
        self.action = Action::default();
        self.front = grid.get(self.at_front());
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Wait,
    MotorForward,
    MotorLeft,
    MotorRight,
}
impl Default for Action {
    fn default() -> Self {
        Self::Wait
    }
}

pub struct Program {
    tpl: wasm::Template<Store>,
    code: Bytes,
}
impl Program {
    #[instrument(level = "trace", skip_all)]
    pub fn new(code: Bytes, vm: &VM) -> sys::Result<Self> {
        let tpl = wasm::Template::new(vm, &code)?;
        Ok(Self { tpl, code })
    }
    #[inline]
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

pub type VM = wasm::Linker<Store>;
