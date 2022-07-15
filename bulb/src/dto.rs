use crate::hex::{Direction, Hex};
pub use bytes::Bytes;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Event {
    State(State),
    TickStart(TickId, Timestamp),
    TickEnd,
    Bot(BotSrc, BotEvent),
    Cells(CellRange),
}
#[derive(Clone, Debug)]
pub enum BotEvent {
    Spawn,
    Die,
    Log(Str),
    Error(Error),
    Rotate(Direction),
    Move(Hex),
    Collide(Hex),
}

#[derive(Debug)]
pub enum Command {
    State(State),
    Compile(Bytes, Promise<CompileRes>),
    Spawn(ProgramId, Hex),
    Map(HexRange, Promise<CellRange>),
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
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

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct TickId(u32);
impl From<u32> for TickId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}
impl From<TickId> for u32 {
    fn from(t: TickId) -> Self {
        t.0
    }
}
impl fmt::Display for TickId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tick{}", self.0)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct BotId(u128);
impl From<u128> for BotId {
    fn from(v: u128) -> Self {
        Self(v)
    }
}
impl From<BotId> for u128 {
    fn from(b: BotId) -> Self {
        b.0
    }
}
impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bot{}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct ProgramId(u32);
impl From<u32> for ProgramId {
    fn from(v: u32) -> Self {
        Self(v)
    }
}
impl From<usize> for ProgramId {
    fn from(v: usize) -> Self {
        Self(v.try_into().unwrap_or_default())
    }
}
impl From<ProgramId> for u32 {
    fn from(p: ProgramId) -> u32 {
        p.0
    }
}
impl From<ProgramId> for usize {
    fn from(p: ProgramId) -> Self {
        usize::try_from(p.0).unwrap_or_default()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Cell {
    Ground,
    Wall,
    Bot(BotId),
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
    pub ctx: &'static str,
    pub err: Str,
}
impl Error {
    pub fn new(ctx: &'static str, err: String) -> Self {
        Self { ctx, err: err.into() }
    }
}

#[derive(Clone)]
pub struct BotSrc {
    pub id: BotId,
    pub at: Hex,
    /*owner*/
}
impl BotSrc {
    #[inline]
    pub fn ev(&self, ev: BotEvent) -> Event {
        Event::Bot(self.clone(), ev)
    }
    pub fn log(&self, msg: String) -> Option<Event> {
        if msg.is_empty() {
            None
        } else {
            Some(self.ev(BotEvent::Log(msg.into())))
        }
    }
    pub fn err(&self, err: Error) -> Event {
        self.ev(BotEvent::Error(err))
    }
}
impl fmt::Debug for BotSrc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BotSrc").field(&self.id.0).finish()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum State {
    Running,
    Paused,
    Stopped,
}

pub type CompileRes = Result<ProgramId, Error>;

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
pub struct HexRange {
    pub center: Hex,
    pub rad: u8,
}

#[derive(Clone)]
pub struct CellRange {
    pub range: HexRange,
    pub cells: Vec<Cell>,
}
impl CellRange {
    pub fn iter(&self) -> impl Iterator<Item = (Hex, Cell)> + '_ {
        self.range.center.range(self.range.rad as i32).zip(self.cells.iter().cloned())
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
