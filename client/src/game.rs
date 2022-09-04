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
}
impl ViewTracker {
    const THROTTLE: usize = 100;

    pub fn new() -> Self {
        Self { at: HexRange { center: Hex::default(), rad: 0 }, throttle: 0 }
    }
    pub fn track(&mut self, client: &mut Client, view: HexRange) {
        if self.at != view {
            if self.throttle > 0 {
                self.throttle -= 1;
            } else {
                self.throttle = Self::THROTTLE;
                self.at = view;
                //FIXME: SetView more ofter
                client.send(Rpc::SetView(Area::Range(view)));
                client.send(Rpc::Map(view));
            }
        }
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
        self.map.insert(src.at, Cell::Bot(src.bid));
        self.bots
            .entry(src.bid)
            .and_modify(|bot| bot.at = src.at)
            .or_insert(BotState { at: src.at, ..Default::default() })
    }

    #[inline]
    fn at(&self, at: Hex) -> Option<&Cell> {
        self.map.get(&at)
    }
    #[inline]
    fn bot(&self, id: BotId) -> Option<&BotState> {
        self.bots.get(&id)
    }
}

/// Allow transitions between previous and current state while building next one
/// Does not handle temporality
#[derive(Default)]
pub struct AnimatedState {
    prev: State,
    cur: State,
    next: State,
    next_deaths: Vec<BotId>,
    tick: Option<(TickId, Timestamp)>,
    next_tick: Option<(TickId, Timestamp)>,
    state: Option<bulb::dto::State>,
}
impl AnimatedState {
    pub fn apply_one(&mut self, e: Event) {
        trace!("{:?}", e);
        use Event::*;
        match e {
            TickEnd => {
                // MAYBE: filter cur view
                self.prev.clone_from(&self.cur);
                for (_, bot) in self.cur.bots.iter_mut() {
                    bot.fix();
                }
                self.cur.merge(std::mem::take(&mut self.next));
                for id in self.next_deaths.iter() {
                    self.cur.bots.remove(id);
                }
                self.next_deaths.clear();
                self.tick = std::mem::take(&mut self.next_tick);
            }
            TickStart { tid, ts } => {
                self.next_tick = Some((tid, ts));
            }
            StateChange(state) => self.state = Some(state),
            Cells(cr) => self.next.map.extend(cr.iter()),
            BotSpawn { src } => _ = self.next.bot_mut(&src),
            BotRotate { src, dir } => self.next.bot_mut(&src).dir = Some(dir),
            BotMove { src, to } => {
                let from = std::mem::replace(&mut self.next.bot_mut(&src).at, to);
                let _prev = self.next.map.insert(from, Cell::Ground);
                debug_assert_eq!(_prev, Some(Cell::Bot(src.bid)));
                self.next.map.insert(to, Cell::Bot(src.bid));
            }
            BotCollide { src, to } => self.next.bot_mut(&src).collide = Some(to),
            BotDie { src } => {
                self.next_deaths.push(src.bid);
                if let Some(bot) = self.next.bots.remove(&src.bid) {
                    self.next.map.insert(bot.at, Cell::Ground);
                }
            }
            BotLog { src, msg } => info!("{:?} log {}", src, msg),
            BotError { src, err } => warn!("{:?} err {:?}", src, err),
        }
    }
    pub fn apply(&mut self, it: impl Iterator<Item = Event>) -> bool {
        let mut ticked = false;
        for e in it {
            ticked |= matches!(e, Event::TickEnd);
            self.apply_one(e);
        }
        ticked
    }

    #[inline]
    pub fn tick(&self) -> Option<(TickId, Timestamp)> {
        self.tick
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
        todo!();
        // client.send(Rpc::Compile(code));
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
pub fn spawn(client: &mut Client, pid: ProgramId, to: Hex) {
    client.send(Rpc::Spawn(SpawnBody { pid, to }))
}
