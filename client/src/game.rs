pub use super::client::Any as Client;
use super::util::*;
use bulb::dto::*;
use std::collections::{BTreeMap, HashMap};

pub struct ViewTracker {
    at: HexRange,
    throttle: usize,
}
impl ViewTracker {
    const THROTTLE: usize = 100;

    pub fn new() -> Self {
        Self {
            at: HexRange {
                center: Hex::default(),
                rad: 0,
            },
            throttle: 0,
        }
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

#[derive(Clone)]
pub enum ObjState {
    PackedBot {

    }
}

#[derive(Default, Clone)]
struct State {
    grid: BTreeMap<Hex, Cell>,
    bots: HashMap<BotId, BotState>,
    programs: Vec<ProgramId>,
    objs: HashMap<ObjId, ObjState>,
}
impl State {
    fn bot_mut(&mut self, src: &BotSrc) -> &mut BotState {
        self.grid.insert(src.at, Cell::Bot(src.bid));
        self.bots
            .entry(src.bid)
            .and_modify(|bot| bot.at = src.at)
            .or_insert(BotState {
                at: src.at,
                ..Default::default()
            })
    }

    #[inline]
    fn at(&self, at: Hex) -> Option<&Cell> {
        self.grid.get(&at)
    }
    #[inline]
    fn bot(&self, id: BotId) -> Option<&BotState> {
        self.bots.get(&id)
    }
    #[inline]
    fn programs(&self) -> &[ProgramId] {
        &self.programs
    }
    #[inline]
    fn obj(&self, id: ObjId) -> Option<&ObjState> {
        self.objs.get(&id)
    }
}

/// Allow transitions between previous and current state
/// Does not handle temporality
#[derive(Default)]
pub struct AnimatedState {
    prev: State,
    cur: State,
    tick: Option<(TickId, Timestamp)>,
}
impl AnimatedState {
    pub fn apply(&mut self, tick: &[Event]) {
        debug_assert!(matches!(tick.last(), Some(Event::TickEnd)) && tick.len() > 1);
        // MAYBE: filter cur view
        self.prev.clone_from(&self.cur);
        for (_, bot) in self.cur.bots.iter_mut() {
            bot.fix();
        }
        for e in tick.split_last().unwrap().1 {
            trace!("{:?}", e);
            use Event::*;
            match e {
                TickStart { tid, ts } => self.tick = Some((*tid, *ts)),
                TickEnd => unreachable!("Double end"),
            }
        }
    }

    pub fn apply_one(&mut self, e: Event) {
        trace!("{:?}", e);
        use Event::*;
        match e {
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
                debug_assert_eq!(_prev, Some(Cell::Obj(src.bid)));
                self.next.map.insert(to, Cell::Obj(src.bid));
            }
            BotCollide { src, to } => self.next.bot_mut(&src).collide = Some(to),
            BotDie { src } => {
                self.next_tombs.push(src.bid);
                if let Some(bot) = self.next.bots.remove(&src.bid) {
                    self.next.map.insert(bot.at, Cell::Ground);
                }
            }
            BotLog { src, msg } => info!("{:?} log {}", src, msg),
            BotError { src, err } => warn!("{:?} err {:?}", src, err),
            ProgramAdd { cid, pid } => {
                self.next.programs.push(pid);
                info!("CompileId({}) ok {:?}", cid, pid)
            }
            CompileError { cid, err } => error!("CompileId({}) err {:?}", cid, err),
        }
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
    #[inline]
    pub fn programs(&self) -> &[ProgramId] {
        self.cur.programs()
    }
}

pub fn compile(client: &mut Client, code: Bytes) {
    client.send(Rpc::Compile { cid: 0, code });
}
pub fn spawn(client: &mut Client, pid: ProgramId, to: Hex) {
    client.send(Rpc::Spawn(SpawnBody { pid, to }))
}
