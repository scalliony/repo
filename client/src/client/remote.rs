use bulb::dto::*;

pub struct Client {}
impl Client {
    pub fn new(url: String) -> Self {
        todo!()
    }
}
impl super::Client for Client {
    fn try_recv(&mut self) -> Option<Event> {
        todo!()
    }
    fn send(&mut self, v: Command) {
        todo!()
    }
}
