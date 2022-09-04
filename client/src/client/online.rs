use bulb::dto::*;
use quad_net::web_socket::WebSocket;

pub struct Client {
    ws: WebSocket,
}
impl Client {
    //FIXME: proper error
    pub fn new(url: &str) -> Result<Self, String> {
        let ws = WebSocket::connect(url).map_err(|e| format!("{:?}", e))?;
        Ok(Self { ws })
    }
}
impl super::Client for Client {
    fn try_recv(&mut self) -> Option<Event> {
        while let Some(packet) = self.ws.try_recv() {
            match serde_json::from_slice::<Event>(&packet) {
                Ok(ev) => return Some(ev),
                Err(err) => crate::util::warn!("Parse err: {}", err),
            }
        }
        None
    }
    fn send(&mut self, rpc: Rpc) {
        let packet = serde_json::to_string(&rpc).unwrap();
        self.ws.send_text(&packet)
    }
    fn connected(&self) -> bool {
        self.ws.connected()
    }
}
