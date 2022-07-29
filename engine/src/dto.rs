use super::generational as gen;
pub use bytes::Bytes;
use chrono::{DateTime, Utc};
use derive_more::{From, Into};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

#[derive(Clone, Debug)]
pub enum Event {
    State(State),
    TickStart(TickId, DateTime<Utc>),
    TickEnd,
    Log(BotSrc, Str),
    Error(BotSrc, Error),
}

#[derive(Debug)]
pub enum Command {
    State(State),
    Compile(Bytes, Promise<CompileRes>),
    Spawn(ProgramId),
}

/// A cheaply clonable readonly String
#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Deserialize)]
#[serde(from = "String")]
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
impl Serialize for Str {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
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
    pub(crate) fn resolve(self, v: V) {
        self.0(v)
    }
}
impl<V> fmt::Debug for Promise<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}) -> ()", std::any::type_name::<V>())
    }
}

#[derive(Debug, Clone, Copy, Into, From, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TickId(u32);
impl fmt::Display for TickId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tick{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Into, From, Serialize, Deserialize)]
#[into(types(gen::Id))]
#[serde(transparent)]
pub struct BotId(gen::I);
impl From<gen::Id> for BotId {
    fn from(id: gen::Id) -> Self {
        Self(id.into())
    }
}
impl fmt::Display for BotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bot{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, Into, From, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProgramId(u32);
impl From<usize> for ProgramId {
    fn from(v: usize) -> Self {
        Self(v.try_into().unwrap_or_default())
    }
}
impl From<ProgramId> for usize {
    fn from(p: ProgramId) -> Self {
        usize::try_from(p.0).unwrap_or_default()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Error {
    pub ctx: &'static str,
    pub err: Str,
}
impl Error {
    pub(crate) fn new(ctx: &'static str, err: String) -> Self {
        Self { ctx, err: err.into() }
    }
}

#[derive(Clone)]
pub struct BotSrc {
    pub id: BotId,
    /*owner, location*/
}
impl BotSrc {
    pub(crate) fn log(&self, msg: String) -> Option<Event> {
        if msg.is_empty() {
            None
        } else {
            Some(Event::Log(self.clone(), msg.into()))
        }
    }
    pub(crate) fn err(&self, err: Error) -> Event {
        Event::Error(self.clone(), err)
    }
}
impl fmt::Debug for BotSrc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BotSrc").field(&self.id.0).finish()
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum State {
    Running,
    Paused,
    Stopped,
}

pub type CompileRes = Result<ProgramId, Error>;
