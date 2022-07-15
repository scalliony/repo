#[cfg(feature = "local")]
mod local;
mod remote;
pub use bulb::dto::*;

trait Client {
    fn update(&mut self) {}
    fn try_recv(&mut self) -> Option<Event>;
    fn send(&mut self, c: Command);
}

pub enum Any {
    #[cfg(feature = "local")]
    Local(local::Client),
    Remote(remote::Client),
}
impl Any {
    #[cfg(feature = "local")]
    pub fn new_local() -> Self {
        Self::Local(local::Client::new())
    }
    pub fn new_remote(url: String) -> Self {
        Self::Remote(remote::Client::new(url))
    }

    pub fn update(&mut self) {
        match self {
            #[cfg(feature = "local")]
            Self::Local(c) => c.update(),
            Self::Remote(c) => c.update(),
        }
    }
    pub fn try_recv(&mut self) -> Option<Event> {
        match self {
            #[cfg(feature = "local")]
            Self::Local(c) => c.try_recv(),
            Self::Remote(c) => c.try_recv(),
        }
    }
    pub fn send(&mut self, v: Command) {
        match self {
            #[cfg(feature = "local")]
            Self::Local(c) => c.send(v),
            Self::Remote(c) => c.send(v),
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
