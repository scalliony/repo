#[cfg(feature = "offline")]
mod offline;
#[cfg(feature = "online")]
mod online;
pub use bulb::dto::*;

#[cfg(not(any(feature = "online", feature = "offline")))]
compile_error!("Either feature 'online' or 'offline' must be enabled");

trait Client {
    fn update(&mut self) {}
    fn try_recv(&mut self) -> Option<Event>;
    fn send(&mut self, c: Rpc);
    fn connected(&self) -> bool {
        true
    }

    fn compile(&mut self, code: Bytes) -> CompileReq;
}
pub type CompileReq = Box<dyn Compiling>;
pub trait Compiling {
    fn try_recv(&mut self) -> Option<CompileRes>;
}

pub enum Any {
    #[cfg(feature = "offline")]
    Off(offline::Client),
    #[cfg(feature = "online")]
    On(online::Client),
}
impl Any {
    pub const HAS_ONLINE: bool = cfg!(feature = "online");
    pub const HAS_OFFLINE: bool = cfg!(feature = "offline");

    pub fn new(domain_or_local: Option<(&str, bool)>) -> Result<Self, String> {
        if let Some((domain, https)) = domain_or_local {
            #[cfg(not(feature = "online"))]
            return Err("Online mode unavailable".to_string());
            #[cfg(feature = "online")]
            return Ok(Self::On(online::Client::new(domain, https)?));
        } else {
            #[cfg(not(feature = "offline"))]
            return Err("Offline mode unavailable".to_string());
            #[cfg(feature = "offline")]
            return Ok(Self::Off(offline::Client::new()));
        }
    }

    pub fn update(&mut self) {
        match self {
            #[cfg(feature = "offline")]
            Self::Off(c) => c.update(),
            #[cfg(feature = "online")]
            Self::On(c) => c.update(),
        }
    }
    pub fn try_recv(&mut self) -> Option<Event> {
        match self {
            #[cfg(feature = "offline")]
            Self::Off(c) => c.try_recv(),
            #[cfg(feature = "online")]
            Self::On(c) => c.try_recv(),
        }
    }
    pub fn send(&mut self, v: Rpc) {
        match self {
            #[cfg(feature = "offline")]
            Self::Off(c) => c.send(v),
            #[cfg(feature = "online")]
            Self::On(c) => c.send(v),
        }
    }
    pub fn connected(&self) -> bool {
        match self {
            #[cfg(feature = "offline")]
            Self::Off(c) => c.connected(),
            #[cfg(feature = "online")]
            Self::On(c) => c.connected(),
        }
    }
    pub fn compile(&mut self, code: Bytes) -> CompileReq {
        match self {
            #[cfg(feature = "offline")]
            Self::Off(c) => c.compile(code),
            #[cfg(feature = "online")]
            Self::On(c) => c.compile(code),
        }
    }
}
impl Iterator for Any {
    type Item = Event;

    #[inline]
    fn next(&mut self) -> Option<Event> {
        self.try_recv()
    }
}
