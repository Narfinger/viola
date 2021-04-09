use futures::{stream::SplitSink, SinkExt};
//use actix::prelude::*;
//use actix::{Actor, StreamHandler};
//use actix_web_actors::ws;
use std::{ops::DerefMut, sync::Arc};
use tokio::sync::RwLock;
use viola_common::WsMessage;
use warp::ws::Message;

pub type MyWs = Arc<RwLock<Option<SplitSink<warp::ws::WebSocket, warp::ws::Message>>>>;

pub async fn send_my_message(socket: &MyWs, msg: WsMessage) {
    let mut socket = socket.write().await;
    if let Some(sink) = socket.deref_mut() {
        let st = serde_json::to_string(&msg).expect("Error serializing");
        let msg = Message::text(&st);
        sink.send(msg).await.expect("Error in sending");
    }
}

/*/
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
