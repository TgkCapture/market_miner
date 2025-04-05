use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::{Instant, Duration};

// WebSocket connection struct
struct StockWebSocket {
    hb: Instant,
}

impl Actor for StockWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

// Handle WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for StockWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Text(text)) => ctx.text(text),
            _ => (),
        }
    }
}

// WebSocket endpoint handler
pub async fn stock_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(StockWebSocket { hb: Instant::now() }, &req, stream);
    resp
}