//use actix::prelude::*;
//use actix::{Actor, StreamHandler};
//use actix_web_actors::ws;
use tokio::sync::RwLock;
use viola_common::WsMessage;

#[derive(Clone)]
pub struct MyWs {
    //    pub addr: Option<Addr<Self>>,
}

pub async fn handle_websocket() {}

/*
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
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, _: &mut Self::Context) {
        match msg {
            //Ok(ws::Message::Text(b)) => println!("we found text {}", b),
            _ => {}
        }
        //self.addr.unwrap().do_send(msg.unwrap());
        //println!("We want to handle");
    }
}

pub fn send_my_message(ws: &RwLock<Option<MyWs>>, msg: WsMessage) {
    let addr = ws.read().as_ref().and_then(|t| t.addr.to_owned());
    if let Some(addr) = addr {
        addr.do_send(msg);
    }
}
*/
