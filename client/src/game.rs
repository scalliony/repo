pub use super::client::Any as Client;
use super::util::*;
use bulb::dto::*;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex},
};

//FIXME: HexRangeIter::new(view.center, view.rad as i32).zip(map.into_iter())

pub struct ViewTracker {
    at: HexRange,
    throttle: usize,
    map: Arc<Mutex<Option<CellRange>>>,
}
impl ViewTracker {
    const THROTTLE: usize = 100;

    pub fn new() -> Self {
        Self { at: HexRange { center: Hex::default(), rad: 0 }, throttle: 0, map: Arc::default() }
    }
    pub fn track(&mut self, client: &mut Client, view: HexRange) -> Option<CellRange> {
        if self.at != view {
            if self.throttle > 0 {
                self.throttle -= 1;
            } else {
                self.throttle = Self::THROTTLE;
                self.at = view;
                let map = self.map.clone();
                client.send(Command::Map(
                    view,
                    Promise::new(move |cr: CellRange| *map.lock().unwrap() = Some(cr)),
                ))
            }
        }
        self.map.lock().unwrap().take()
    }
}

#[derive(Default, Clone)]
pub struct BotState {
    pub at: Hex,
    pub collide: Option<Hex>,
    pub dir: Option<Direction>,
}
impl BotState {
    #[inline]
    fn fix(&mut self) {
        self.collide = None;
    }
}

trait StateView {
    fn at(&self, at: Hex) -> Option<&Cell>;
    fn bot(&self, id: BotId) -> Option<&BotState>;
}

#[derive(Default, Clone)]
struct State {
    map: BTreeMap<Hex, Cell>,
    bots: HashMap<BotId, BotState>,
}
impl State {
    fn merge(&mut self, other: State) {
        self.bots.extend(other.bots.into_iter());
        self.map.extend(other.map.into_iter());
    }
    fn bot_mut(&mut self, src: &BotSrc) -> &mut BotState {
        self.map.insert(src.at, Cell::Bot(src.id));
        self.bots
            .entry(src.id)
            .and_modify(|bot| bot.at = src.at)
            .or_insert(BotState { at: src.at, ..Default::default() })
    }
}

#[derive(Default)]
struct FullState(State);
impl StateView for FullState {
    fn at(&self, at: Hex) -> Option<&Cell> {
        self.0.map.get(&at)
    }
    fn bot(&self, id: BotId) -> Option<&BotState> {
        self.0.bots.get(&id)
    }
}

struct HistoricState<'a>(&'a [State]);
impl StateView for HistoricState<'_> {
    fn at(&self, at: Hex) -> Option<&Cell> {
        self.0.iter().rev().find_map(|state| state.map.get(&at))
    }
    fn bot(&self, id: BotId) -> Option<&BotState> {
        self.0.iter().rev().find_map(|state| state.bots.get(&id))
    }
}

#[derive(Default)]
pub struct AnimatedState {
    prev: FullState,
    cur: FullState,
    next: State,
    next_deaths: Vec<BotId>,
    tick: Option<(TickId, Timestamp)>,
    next_tick: Option<(TickId, Timestamp)>,
    state: Option<bulb::dto::State>,
}
impl AnimatedState {
    pub fn apply_one(&mut self, e: Event) {
        macroquad::prelude::trace!("{:?}", e);
        match e {
            Event::TickEnd => {
                // MAYBE: filter cur view
                self.prev.0.clone_from(&self.cur.0);
                for (_, bot) in self.cur.0.bots.iter_mut() {
                    bot.fix();
                }
                self.cur.0.merge(std::mem::take(&mut self.next));
                for id in self.next_deaths.iter() {
                    self.cur.0.bots.remove(id);
                }
                self.next_deaths.clear();
                self.tick = std::mem::take(&mut self.next_tick);
            }
            Event::TickStart(id, at) => {
                self.next_tick = Some((id, at));
            }
            Event::State(state) => self.state = Some(state),
            Event::Cells(cr) => self.next.map.extend(cr.iter()),
            Event::Bot(src, e) => match e {
                BotEvent::Spawn => _ = self.next.bot_mut(&src),
                BotEvent::Rotate(to) => self.next.bot_mut(&src).dir = Some(to),
                BotEvent::Move(to) => {
                    let from = std::mem::replace(&mut self.next.bot_mut(&src).at, to);
                    let _prev = self.next.map.insert(from, Cell::Ground);
                    debug_assert_eq!(_prev, Some(Cell::Bot(src.id)));
                    self.next.map.insert(to, Cell::Bot(src.id));
                }
                BotEvent::Collide(to) => self.next.bot_mut(&src).collide = Some(to),
                BotEvent::Die => {
                    self.next_deaths.push(src.id);
                    if let Some(bot) = self.next.bots.remove(&src.id) {
                        self.next.map.insert(bot.at, Cell::Ground);
                    }
                }
                BotEvent::Log(msg) => macroquad::prelude::info!("{:?} log {}", src, msg),
                BotEvent::Error(msg) => macroquad::prelude::warn!("{:?} err {:?}", src, msg),
            },
        }
    }
    pub fn apply(&mut self, client: &mut Client) -> bool {
        let mut ticked = false;
        for e in client {
            ticked |= matches!(e, Event::TickEnd);
            self.apply_one(e);
        }
        ticked
    }

    pub fn at(&self, at: Hex) -> (Option<&Cell>, Option<&Cell>) {
        (self.prev.at(at), self.cur.at(at))
    }
    pub fn bot(&self, id: BotId) -> (Option<&BotState>, Option<&BotState>) {
        (self.prev.bot(id), self.cur.bot(id))
    }
}

pub struct Programs {
    actual: Vec<ProgramId>,
    pipe: Arc<Mutex<Vec<ProgramId>>>,
}
impl Programs {
    pub fn new() -> Self {
        Self { actual: Vec::default(), pipe: Arc::default() }
    }
    pub fn compile(&self, client: &mut Client, code: Bytes) {
        let pipe = self.pipe.clone();
        client.send(Command::Compile(
            code,
            Promise::new(move |res: CompileRes| pipe.lock().unwrap().push(res.unwrap())),
        ))
    }
    pub fn update(&mut self) {
        let pipe = &mut self.pipe.lock().unwrap();
        self.actual.extend_from_slice(pipe.as_slice());
        pipe.clear();
    }
}
impl AsRef<[ProgramId]> for Programs {
    fn as_ref(&self) -> &[ProgramId] {
        &self.actual
    }
}
pub fn spawn(client: &mut Client, program: ProgramId, at: Hex) {
    client.send(Command::Spawn(program, at))
}
