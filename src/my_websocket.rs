use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_web_actors::ws;
use std::sync::RwLock;

#[derive(Clone, Message, Serialize)]
#[serde(tag = "type")]
#[rtype(result = "()")]
pub enum WsMessage {
    PlayChanged { index: usize },
    ReloadPlaylist,
    Ping,
}

impl From<WsMessage> for String {
    fn from(msg: WsMessage) -> Self {
        serde_json::to_string(&msg).unwrap()
    }
}

#[derive(Clone)]
pub struct MyWs {
    pub addr: Option<Addr<Self>>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<WsMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            //Ok(ws::Message::Text(b)) => println!("we found text {}", b),
            _ => {}
        }
        //self.addr.unwrap().do_send(msg.unwrap());
        //println!("We want to handle");
    }
}

pub fn send_my_message(ws: &RwLock<Option<MyWs>>, msg: WsMessage) {
    let addr = ws.read().unwrap().as_ref().unwrap().addr.clone();
    addr.unwrap().do_send(msg);
}
