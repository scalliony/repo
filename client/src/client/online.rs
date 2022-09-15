use bulb::dto::*;
use quad_net::http_request::{Method, Request, RequestBuilder};
use quad_net::web_socket::WebSocket;

pub struct Client {
    http: String,
    ws: WebSocket,
}
impl Client {
    pub fn new(domain: &str, https: bool) -> Result<Self, String> {
        let s = if https { "s" } else { "" };
        let ws_url = format!("ws{}://{}/api/ws", s, domain);
        let ws = WebSocket::connect(&ws_url).map_err(|e| format!("{:?}", e))?;
        let http = format!("http{}://{}/api/compile", s, domain);
        Ok(Self { ws, http })
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

    fn compile(&mut self, code: Bytes) -> super::CompileReq {
        //FIXME: accept &[u8]
        let code = std::str::from_utf8(&code).expect("binary upload not supported for now");
        Box::new(RequestBuilder::new(&self.http).method(Method::Post).body(code).send())
    }
}

impl super::Compiling for Request {
    fn try_recv(&mut self) -> Option<CompileRes> {
        match self.try_recv() {
            Some(Ok(res)) => Some(Ok(serde_json::from_str(&res).unwrap())),
            Some(Err(err)) => Some(Err(Error::new("Network error", err.to_string()))),
            None => None,
        }
    }
}
