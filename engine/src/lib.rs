mod api;
mod bot;
mod gen;
mod helper;
mod noise;
mod user;
mod grid;
use api::*;
use bot::Bot;
pub use bulb::dto::{Event::*, *};
use bulb::hex::{Angle, Hex};
use chrono::Utc;
pub use helper::*;
use std::collections::{BTreeMap, HashMap};
use tracing::instrument;
use user::Users;
use grid::Grid;

pub const DEFAULT_TICK_DURATION_MS: u64 = 1000;

pub struct Game<S> {
    events: EventSender<S>,

    counter: u32,
    in_tick: bool,

    vm: VM,
    users: Users,
    objects: Objects,
    grid: Grid,
    cache: Cache,
}
impl<S: FnMut(Event)> Game<S> {
    pub fn new(events: S) -> Self {
        Self {
            events: EventSender(events),
            counter: 0,
            in_tick: false,
            vm: new_vm().unwrap(),
            users: Users::new(),
            objects: Objects::new(),
            grid: Grid::new(42),
            cache: Cache::new(),
        }
    }
    //TODO: serialize, deserialize

    #[instrument(skip_all, fields(id = self.counter))]
    pub fn tick(&mut self) {
        self.with_tick();

        for (id, bot) in self.bots.iter_mut() {
            //Process
            Self::tick_bot(
                id,
                bot,
                &mut self.programs,
                &self.vm,
                &self.map,
                &mut self.events,
            )
        }
        self.tick_act();
        self.tick_death();
        self.tick_move();

        self.events.send(TickEnd);
        tracing::debug!("done");
        self.in_tick = false;
        self.counter += 1;
    }
    #[inline]
    #[instrument(level = "debug", name = "bot", skip_all, fields(id = gen::I::from(id)))]
    fn tick_bot(
        id: BotId,
        bot: &mut Bot,
        programs: &mut Programs,
        vm: &VM,
        map: &GameMap,
        events: &mut EventSender<S>,
    ) {
        match bot.cpu.as_mut() {
            Ok(cpu) => cpu.state_mut().update(map),
            Err(off) => {
                off.fuel -= 1;
                //TODO: if sleep return
                if off.fuel < MIN_BOOT_FUEL {
                    return;
                }

                let mut state = bot::State::boot(id, off);
                let src = state.src();
                state.update(map);
                let tpl = programs[bot.program].compiled(vm);
                match bot::Cpu::boot(tpl.unwrap(), state, off.fuel) {
                    Ok(cpu) => bot.cpu = Ok(cpu),
                    Err((log, err)) => {
                        events.log(src, log);
                        events.send(BotError {
                            src,
                            err: err_wrap("During start", err),
                        });
                        return;
                    }
                }
            }
        }
        let cpu = bot.cpu.as_mut().ok().unwrap();

        // Tick
        let src = cpu.state().src();
        tracing::trace!("fuel {}", cpu.process.fuel());
        let res = cpu.tick();
        events.log(src, cpu.store_mut().read_log());
        if let Err(err) = res {
            events.send(BotError {
                src,
                err: err_wrap("During tick", err),
            });
        }
    }
    /// Edit bots and maps
    /// Fill self.cache.moves
    #[instrument(level = "trace", skip_all)]
    fn tick_act(&mut self) {
        self.cache.moves.clear();
        self.cache.deaths.clear();
        for index in self.bots.iter_index() {
            if let Some((id, bot, others)) = self.bots.split_at_mut(index) {
                let alive = match &mut bot.cpu {
                    Ok(cpu) => {
                        let src = cpu.state().src();
                        let action = cpu.state().action;
                        tracing::debug!(?src, ?action);
                        use bot::Action::*;
                        let mut _alive = cpu.process.fuel() > 0;
                        let alive = &mut _alive;
                        fn consume_fuel(cpu: &mut bot::Cpu, v: u64, alive: &mut bool) -> bool {
                            match cpu.process.consume_fuel(v) {
                                Ok(_) => true,
                                Err(_) => {
                                    *alive = false;
                                    false
                                }
                            }
                        }
                        match action {
                            //TODO: mine, attack, etc...
                            MotorLeft => {
                                if consume_fuel(cpu, TURN_FUEL, alive) {
                                    let state = cpu.state_mut();
                                    state.facing += Angle::Left;
                                    self.events.send(BotRotate {
                                        src,
                                        dir: state.facing,
                                    });
                                }
                            }
                            MotorRight => {
                                if consume_fuel(cpu, TURN_FUEL, alive) {
                                    let state = cpu.state_mut();
                                    state.facing += Angle::Right;
                                    self.events.send(BotRotate {
                                        src,
                                        dir: state.facing,
                                    });
                                }
                            }
                            MotorForward => {
                                if consume_fuel(cpu, MOVE_FUEL, alive) {
                                    use TryMoveState::*;
                                    let state = cpu.state();
                                    let to = state.at_front();
                                    let mut mov = match self.map.get(to) {
                                        Cell::Ground => Valid,
                                        Cell::Bot(other_id) => {
                                            // Assume other bot will move successfully
                                            if let Ok(other) = &others.get(other_id).unwrap().cpu {
                                                let other = other.state();
                                                if other.action == MotorForward
                                                // No passthrough swap
                                                && other.facing != -state.facing
                                                {
                                                    After(other.at_front())
                                                } else {
                                                    Cancelled
                                                }
                                            } else {
                                                Cancelled
                                            }
                                        }
                                        Cell::Wall => Cancelled,
                                    };
                                    if mov.is_ok()
                                        && !match self.cache.moves.get_mut(&to) {
                                            Some(v) => {
                                                if v.1.is_ok() {
                                                    let other =
                                                        others.get(v.0).unwrap().cpu.as_ref();
                                                    let other = other.unwrap().state().src();
                                                    self.events.send(BotCollide { src: other, to });
                                                    v.1 = Cancelled;
                                                }
                                                false
                                            }
                                            None => true,
                                        }
                                    {
                                        mov = Cancelled;
                                    }
                                    if mov.is_ok() {
                                        self.cache.moves.insert(to, (id, mov));
                                        //NOTE: Postponed
                                    } else {
                                        self.events.send(BotCollide { src, to });
                                    }
                                }
                            }
                            Wait => {}
                        }
                        *alive
                    }
                    Err(off) => off.fuel > 0,
                };
                if !alive {
                    self.cache.deaths.push(id);
                }
            }
        }
    }
    /// Move bot chains
    /// Consume self.cache.deaths
    #[instrument(level = "trace", skip_all)]
    fn tick_death(&mut self) {
        //FIXME: remove deaths
        for id in self.cache.deaths.iter() {
            if let Ok(bot) = self.bots.remove(*id) {
                let src = bot.src(*id);
                self.map.set(src.at, Cell::Ground);
                self.events.send(BotDie { src });
            }
        }
        self.cache.deaths.clear();
    }
    /// Move bot chains
    /// Consume self.cache.moves
    #[instrument(level = "trace", skip_all)]
    fn tick_move(&mut self) {
        let ms = &mut self.cache.moves;
        while let Some((to, id, state)) = next_ok(ms) {
            let tail = if let Ok(bot) = self.bots.get(id) {
                bot.at()
            } else {
                ms.remove(&to);
                continue;
            };

            // Check chain head
            let mut it = It {
                to: Some(to),
                tail,
                state,
            };
            while let TryMoveState::After(_) = it.state {
                it.next(ms);
            }
            let valid = it.state.is_ok();
            let head = it.to;
            it = It {
                to: Some(to),
                tail,
                state,
            };

            if valid && head.map_or(true, |at| self.map.get(at).is_empty()) {
                // Move chain
                while let Some(to) = it.next(ms) {
                    let id = ms.remove(&to).unwrap().0;
                    if let Ok(bot) = self.bots.get_mut(id) {
                        if let Ok(cpu) = &mut bot.cpu {
                            let state = cpu.state_mut();
                            debug_assert!(state.at_front() == to);
                            let src = state.src();
                            self.map.set(to, Cell::Bot(id));
                            state.at = to;
                            self.events.send(BotMove { src, to });
                        } else {
                            panic!("Bot off moved {:?} ???", id)
                        }
                    }
                }
                if head.is_some() {
                    self.map.set(tail, Cell::Ground);
                }
            } else {
                // Cancel chain
                while let Some(to) = it.next(ms) {
                    let (id, state) = ms.get_mut(&to).unwrap();
                    *state = TryMoveState::Cancelled;
                    let id = *id;
                    if let Ok(bot) = self.bots.get(id) {
                        self.events.send(BotCollide {
                            src: bot.src(id),
                            to,
                        });
                    }
                }
            }
        }
        if !ms.is_empty() {
            tracing::debug!(count = ms.len(), "cancelled moves");
        }
        ms.clear();

        #[inline(always)]
        fn next_ok(ms: &HashMap<Hex, (BotId, TryMoveState)>) -> Option<(Hex, BotId, TryMoveState)> {
            ms.iter()
                .find(|(_, (_, state))| state.is_ok())
                .map(|(at, (id, state))| (*at, *id, *state))
        }
        struct It {
            to: Option<Hex>,
            tail: Hex,
            state: TryMoveState,
        }
        impl It {
            fn next(&mut self, ms: &HashMap<Hex, (BotId, TryMoveState)>) -> Option<Hex> {
                let res = self.to;
                self.to = if let TryMoveState::After(at) = self.state {
                    self.state = TryMoveState::Valid;
                    if let Some((_, next)) = ms.get(&at) {
                        if at != self.tail {
                            // No Loop
                            self.state = *next;
                        }
                        Some(at)
                    } else {
                        None // Last moved
                    }
                } else {
                    None
                };
                res
            }
        }
    }

    #[instrument(level = "trace", name = "command", skip_all)]
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Compile(code, cb) => {
                let res = Program::new(code, &self.vm)
                    .map(|program| self.programs.push_and_get_key(program))
                    .map_err(|err| Error::new("Failed to compile", err.root_cause().to_string()));
                cb.resolve(res)
            }
            Command::Map(r, cb) => {
                let range = r.center.range(r.rad as bulb::hex::I);
                cb.resolve(CellRange::new(r, &self.map));
            }
            Command::BotSpawn(q) => {
                let user = match self.users.get_mut(q.uid) {
                    Some(v) => v,
                    None => {
                        tracing::warn!("bad {:?}", q.uid);
                        return; //FIXME: Bad user
                    }
                };
                let prg = match user.get_program(q.pid) {
                    Some(v) => v,
                    None => {
                        tracing::warn!("bad {:?}", q.pid);
                        return; //FIXME: Bad program
                    }
                };

                if !self.grid.get(q.to).is_empty() {
                    tracing::warn!("bad {:?}", at);
                    return; //FIXME: Bad pos
                }
                let bid = self.bots.insert(Bot {
                    program: q.pid,
                    cpu: Err(bot::StateOff {
                        at,
                        facing: bulb::hex::Direction::Up,
                        fuel: 10_000,
                    }),
                });
                self.map.set(at, Cell::Bot(bid));
                self.with_tick();
                self.events.send(Event::BotSpawn {
                    src: BotSrc { bid, at },
                });
            }
            Command::ChangeState(_) => unreachable!("Server command"),
        }
    }

    #[inline]
    pub fn send(&mut self, e: Event) {
        self.events.send(e)
    }

    fn with_tick(&mut self) {
        if !self.in_tick {
            self.events.send(TickStart {
                tid: self.counter.into(),
                ts: Utc::now().timestamp_millis().into(),
            });
            self.in_tick = true;
        }
    }
}

type Bots = gen::Array<BotId, Bot>;

struct GameMap {
    pub grid: BTreeMap<Hex, Cell>,
    pub gen: MapGenerator,
}
impl GameMap {
    fn new(seed: u32) -> Self {
        Self {
            grid: BTreeMap::new(),
            gen: MapGenerator::new(seed),
        }
    }
    fn set(&mut self, h: Hex, v: Cell) {
        self.grid.insert(h, v);
    }
    fn drain_unchanged(&mut self) {
        self.grid.retain(|h, v| *v != self.gen.get(*h));
    }
}
impl CellMap for GameMap {
    fn get(&self, h: Hex) -> Cell {
        if let Some(v) = self.grid.get(&h) {
            *v
        } else {
            self.gen.get(h)
        }
    }
}
struct MapGenerator(noise::Fbm<noise::OpenSimplex>);
impl MapGenerator {
    fn new(seed: u32) -> Self {
        use noise::Seedable;
        let mut noise = noise::Fbm::new_seed(seed);
        noise.frequency = 1. / 256.;
        Self(noise)
    }
    fn get(&self, h: Hex) -> Cell {
        use noise::NoiseFn;
        let p = bulb::hex::Point::from(h);
        let height = self.0.get([p.x, p.y]);
        if height < 0. {
            Cell::Ground
        } else {
            Cell::Wall
        }
    }
}

struct GameCache {
    moves: HashMap<Hex, (BotId, TryMoveState)>,
    deaths: Vec<BotId>,
}
impl GameCache {
    fn new() -> Self {
        Self {
            moves: HashMap::new(),
            deaths: Vec::new(),
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
enum TryMoveState {
    Valid,
    After(Hex),
    Cancelled,
}
impl TryMoveState {
    #[inline]
    fn is_ok(&self) -> bool {
        *self != Self::Cancelled
    }
}

struct EventSender<S>(S);
impl<S: FnMut(Event)> EventSender<S> {
    fn send(&mut self, event: Event) {
        tracing::trace!(?event);
        self.0(event);
    }
    #[inline]
    fn log(&mut self, src: BotSrc, log: String) {
        if !log.is_empty() {
            self.send(BotLog {
                src,
                msg: log.into(),
            })
        }
    }
}

type Programs = TiVec<ProgramId, Program>;
struct Program {
    inner: Option<bot::Template>,
    code: Bytes,
}
impl Program {
    fn new(code: Bytes, vm: &VM) -> Result<Self> {
        let mut s = Self { inner: None, code };
        s.compile(vm)?;
        Ok(s)
    }

    fn compiled(&mut self, vm: &VM) -> Result<&mut bot::Template> {
        if self.inner.is_none() {
            self.compile(vm)?;
        }
        Ok(self.inner.as_mut().unwrap())
    }
    #[cold]
    #[inline]
    #[instrument(level = "trace", skip_all)]
    fn compile(&mut self, vm: &VM) -> Result<()> {
        debug_assert!(self.inner.is_none());
        let tpl = bot::Template::new(vm, &self.code)?;
        self.inner = Some(tpl);
        Ok(())
    }
}
