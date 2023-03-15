use crate::hex::{Direction, Hex, HexRangeIter};
pub use bytes::Bytes;
use std::fmt;

/// Message from the engine
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "k"))]
pub enum Event {
    StateChange(State),
    TickStart {
        tid: TickId,
        ts: Timestamp,
    },
    TickEnd,
    BotSpawn {
        #[cfg_attr(feature = "serde", serde(flatten))]
        src: BotSrc,
    },
    BotDie {
        #[cfg_attr(feature = "serde", serde(flatten))]
        src: BotSrc,
    },
    BotLog {
        #[cfg_attr(feature = "serde", serde(flatten))]
        src: BotSrc,
        msg: Str,
    },
    BotError {
        #[cfg_attr(feature = "serde", serde(flatten))]
        src: BotSrc,
        #[cfg_attr(feature = "serde", serde(flatten))]
        err: Error,
    },
    BotRotate {
        #[cfg_attr(feature = "serde", serde(flatten))]
        src: BotSrc,
        dir: Direction,
    },
    BotMove {
        #[cfg_attr(feature = "serde", serde(flatten))]
        src: BotSrc,
        to: Hex,
    },
    BotCollide {
        #[cfg_attr(feature = "serde", serde(flatten))]
        src: BotSrc,
        to: Hex,
    },
    ObjAdd {
        oid: ObjId,
        at: Hex,
    },
    ObjRm {
        oid: ObjId,
        at: Hex,
    },
    Cells(CellRange),
    ProgramAdd {
        pid: ProgramId,
        cid: CompileId,
    },
    CompileError {
        cid: CompileId,
        err: Error,
    },
}
impl Event {
    pub fn src(&self) -> Option<&BotSrc> {
        use Event::*;
        match self {
            BotSpawn { src } => Some(src),
            BotDie { src } => Some(src),
            BotLog { src, .. } => Some(src),
            BotError { src, .. } => Some(src),
            BotRotate { src, .. } => Some(src),
            BotMove { src, .. } => Some(src),
            BotCollide { src, .. } => Some(src),
            _ => None,
        }
    }
}

/// Message to the engine
#[derive(Debug)]
pub enum Command {
    ChangeState(State),
    Compile(Bytes, Promise<CompileRes>),
    BotSpawn(SpawnBody),
    Map(HexRange, Promise<CellRange>),
    UserByLogin(Str, Promise<Option<UserInfo>>),
    UserSpawn {
        login: Str,
        name: Str,
        cb: Promise<(UserId, Hex)>,
    },
}

/// Over the network [`Command`]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "k"))]
pub enum Rpc {
    SetView(Area),
    Map(HexRange),
    Spawn(SpawnBody),
    ChangeState(State),
    Compile { cid: CompileId, code: Bytes },
}

/// A cheaply clonable readonly String
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "String"))]
pub struct Str(Bytes);
impl From<String> for Str {
    fn from(s: String) -> Self {
        Self(Bytes::from(s))
    }
}
impl From<&'static str> for Str {
    fn from(s: &'static str) -> Self {
        Self(Bytes::from_static(s.as_bytes()))
    }
}
impl AsRef<str> for Str {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.0.as_ref()) }
    }
}
impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "S{:?}", self.as_ref())
    }
}
impl fmt::Display for Str {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}
#[cfg(feature = "serde")]
impl serde::Serialize for Str {
    #[inline]
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_ref().serialize(serializer)
    }
}

#[repr(transparent)]
pub struct Promise<V>(Box<dyn FnOnce(V) + Send>);
impl<V> Promise<V> {
    #[inline]
    pub fn new(f: impl FnOnce(V) + Send + 'static) -> Self {
        Self(Box::new(f))
    }

    #[inline]
    pub fn resolve(self, v: V) {
        self.0(v)
    }
}
impl<V, F: FnOnce(V) + Send + 'static> From<F> for Promise<V> {
    fn from(f: F) -> Self {
        Self::new(f)
    }
}
impl<V> fmt::Debug for Promise<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}) -> ()", std::any::type_name::<V>())
    }
}

pub type TickId = u32;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct BotId(u64);
impl From<u64> for BotId {
    fn from(v: u64) -> Self {
        Self(v)
    }
}
impl From<BotId> for u64 {
    fn from(b: BotId) -> Self {
        b.0
    }
}
impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bot{}", self.0)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ObjId(u64);
impl From<u64> for ObjId {
    fn from(v: u64) -> Self {
        Self(v)
    }
}
impl From<ObjId> for u64 {
    fn from(b: ObjId) -> Self {
        b.0
    }
}
impl fmt::Display for ObjId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Obj{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct UserId(u32);
impl From<u32> for UserId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}
impl From<usize> for UserId {
    fn from(v: usize) -> Self {
        Self(v.try_into().unwrap_or_default())
    }
}
impl From<UserId> for u32 {
    fn from(p: UserId) -> u32 {
        p.0
    }
}
impl From<UserId> for usize {
    fn from(p: UserId) -> Self {
        usize::try_from(p.0).unwrap_or_default()
    }
}

/// Compilation request identifier
pub type CompileId = u32;

pub type ProgramId = u32;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Cell {
    Ground,
    Wall,
    Bot(BotId),
    Obj(ObjId),
}
impl Cell {
    #[inline]
    pub fn is_empty(&self) -> bool {
        *self == Self::Ground
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Error {
    pub ctx: Str,
    pub err: Str,
}
impl Error {
    pub fn new(ctx: &'static str, err: String) -> Self {
        Self {
            ctx: ctx.into(),
            err: err.into(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BotSrc {
    pub bid: BotId,
    pub at: Hex,
    pub uid: UserId,
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum State {
    Running,
    Paused,
    Stopped,
}

pub type CompileRes = Result<ProgramId, Error>;

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: UserId,
    pub name: Str,
    pub spawn: Hex,
}

/// Number of non-leap-milliseconds since January 1, 1970 UTC
#[derive(Clone, Copy)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Timestamp(i64);
impl From<i64> for Timestamp {
    fn from(ms: i64) -> Self {
        Self(ms)
    }
}
impl From<Timestamp> for i64 {
    fn from(t: Timestamp) -> Self {
        t.0
    }
}
impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HexRange {
    pub center: Hex,
    pub rad: u8,
}
impl HexRange {
    pub fn iter(&self) -> HexRangeIter {
        self.center.range(self.rad as i32)
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CellRange {
    #[cfg_attr(feature = "serde", serde(flatten))]
    range: HexRange,
    pub cells: Str,
}
impl CellRange {
    pub fn new(range: HexRange, grid: &impl CellGrid) -> Self {
        let mut s = String::with_capacity(range.iter().len());
        for c in range.iter().map(move |h| grid.get(h)) {
            use Cell::*;
            match c {
                Ground => s.push(' '),
                Wall => s.push('x'),
                Bot(BotId(v)) => Self::push_id(&mut s, 'b', v),
                Obj(ObjId(v)) => Self::push_id(&mut s, 'o', v),
            }
        }
        Self {
            range,
            cells: s.into(),
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (Hex, Cell)> + '_ {
        let mut chars = self.cells.as_ref().chars();
        self.range.iter().map(move |h| {
            use Cell::*;
            (h, match chars.next().expect("End of cells") {
                ' ' => Ground,
                'x' => Wall,
                'b' => Bot(BotId(Self::read_id(&mut chars))),
                'o' => Obj(ObjId(Self::read_id(&mut chars))),
                _ => panic!("Invalid cells")
            })
        })
    }

    fn push_id(s: &mut String, ch: char, v: u64) {
        s.push(ch);
        for i in (0..4).rev() {
            let p = ((v & ((u16::MAX as u64) << (i*u16::BITS))) >> i*u16::BITS) as u32 + 0xE000;
            s.push(unsafe { char::from_u32_unchecked(p) });
        }
    }
    fn read_id(chars: &mut std::str::Chars) -> u64 {
        let mut v = 0u64;
        for _ in 0..4 {
            v += chars.next().expect("End of cells") as u64 - 0xE000;
            v <<= u16::BITS;
        }
        v
    }
}
impl fmt::Debug for CellRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cells")
            .field("center", &self.range.center)
            .field("rad", &self.range.rad)
            .finish_non_exhaustive()
    }
}

pub trait CellGrid {
    fn get(&self, h: Hex) -> Cell;
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "t"))]
pub enum Area {
    None,
    Range(HexRange),
    All,
}
impl Default for Area {
    fn default() -> Self {
        Self::None
    }
}
impl Area {
    pub fn contains(&self, p: Hex) -> bool {
        use Area::*;
        match self {
            None => false,
            All => true,
            Range(r) => r.center.dist(p) <= r.rad as i32,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpawnBody {
    pub pid: ProgramId,
    pub to: Hex,
    pub uid: UserId,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Viewer {
    #[cfg_attr(feature = "serde", serde(default))]
    pub view: Area,
}
