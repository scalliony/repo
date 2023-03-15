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
            Self::tick_bot(id, bot, &self.grid, &mut self.events)
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
        grid: &Grid,
        events: &mut EventSender<S>,
    ) {
        bot.state_mut().update(grid);
        match bot.cpu.as_mut() {
            Ok(cpu) => cpu.state_mut().update(grid),
            Err(off) => {
                off.fuel -= 1;
                //TODO: if sleep return
                if off.fuel < MIN_BOOT_FUEL {
                    return;
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
                let src = bot.state().src();
                let action = bot.state().action;
                tracing::debug!(?src, ?action);
                use bot::Action::*;
                match action {
                    //TODO: mine, attack, etc...
                    MotorLeft => {
                        if consume_fuel(bot, TURN_FUEL) {
                            let dir = bot.state_mut().turn(Angle::Left);
                            self.events.send(BotRotate { src, dir });
                        }
                    }
                    MotorRight => {
                        if consume_fuel(bot, TURN_FUEL) {
                            let dir = bot.state_mut().turn(Angle::Right);
                            self.events.send(BotRotate { src, dir });
                        }
                    }
                    MotorForward => {
                        if consume_fuel(bot, MOVE_FUEL) {
                            use TryMoveState::*;
                            let state = bot.state();
                            let to = state.at_front();
                            let mut mov = match self.grid.get(to) {
                                Cell::Ground => Valid,
                                Cell::Bot(other_id) => {
                                    // Assume other bot will move successfully
                                    if let Ok(other) = &others.get(other_id).unwrap().cpu {
                                        let other = other.state();
                                        if other.action == MotorForward
                                        // No passthrough swap
                                        && other.facing() != -state.facing()
                                        {
                                            After(other.at_front())
                                        } else {
                                            Cancelled
                                        }
                                    } else {
                                        Cancelled
                                    }
                                }
                                Cell::Wall | Cell::Obj(_) => Cancelled,
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
                fn consume_fuel(bot: &mut Bot, v: u64) -> bool {
                    let p = bot.process();
                    match p.consume_fuel(v) {
                        Ok(_) => true,
                        Err(_) => {
                            _ = p.consume_fuel(p.fuel());
                            false
                        }
                    }
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
            Command::UserByLogin(login, cb) => {
                cb.resolve(self.users.by_login(&login).map(|(id, data)| UserInfo {
                    id,
                    name: data.name,
                    spawn: data.spawn,
                }))
            }
            Command::UserSpawn { login, name, cb } => {
                self.with_tick();
                if let Some((uid, user)) = self.users.by_login(&login) {
                    self.bots.retain(|bid, bot| {
                        let src = bot.src(bid);
                        let retain = src.uid != uid;
                        if !retain {
                            self.map.set(src.at, Cell::Ground);
                            self.events.send(BotDie { src })
                        }
                        retain
                    });
                }

                const BASE_DIST: bulb::hex::I = 512;
                const MIN_AREA: usize = 262144;
                let spawn = (1..).find_map(|ring| Hex::default().ring(ring).map(|h| h * BASE_DIST).find_map(|spawn| {
                    if self.map.get(spawn) != Cell::Ground {
                        return None
                    }
                    let close = std::collections::HashSet::from([spawn]);
                    let queue = std::collections::VecDeque::from([spawn]);
                    let mut popped = 0usize;
                    while let Some(h) = queue.pop_front() {
                        popped += 1;
                        for d in bulb::hex::Direction::all() {
                            let n = h.neighbor(*d);
                            if close.insert(n) {
                                match self.map.get(n) {
                                    Cell::Bot(_) => return None,
                                    Cell::Ground => queue.push_back(n),
                                    _ => {}
                                }
                            }
                        }
                        if queue.len() + popped > MIN_AREA {
                            return Some(spawn)
                        }
                    }
                    None
                })).expect("Map is full");

                let uid = self.users.set(user::User::new(login, name, spawn));
                //FIXME: add factory
                cb.resolve((uid, spawn))
            },
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

struct Cache {
    moves: HashMap<Hex, (BotId, TryMoveState)>,
}
impl Cache {
    fn new() -> Self {
        Self { moves: HashMap::new() }
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
