use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_files as fs;
use actix_rt;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};

#[derive(Clone, Message, Serialize)]
#[serde(tag = "type")]
#[rtype(result = "()")]
enum WsMessage {
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
    addr: Option<Addr<Self>>,
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

pub async fn ws_start(
    state: WebGui,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut ws = MyWs { addr: None };
    let (addr, resp) = ws::start_with_addr(ws.clone(), &req, stream)?;
    println!("websocket {:?}", resp);
    ws.addr = Some(addr);
    *state.ws.write().unwrap() = Some(ws);
    Ok(resp)
}

pub fn handle_gstreamer_messages(
    state: web::Data<WebGui>,
    rx: Receiver<gstreamer_wrapper::GStreamerMessage>,
) {
    loop {
        //println!("loop is working");
        if let Ok(msg) = rx.try_recv() {
            match msg {
                gstreamer_wrapper::GStreamerMessage::Playing => {
                    let addr = state.ws.read().unwrap().as_ref().unwrap().addr.clone();
                    let pos = state.playlist.current_position();
                    addr.clone()
                        .unwrap()
                        .do_send(WsMessage::PlayChanged { index: pos })
                }
                _ => (),
            }
        }

        /*
        if let Some(a) = state.ws.read().unwrap().as_ref() {
            if let Some(a) = a.addr.clone() {
                println!("Sending ping");
                a.do_send(WsMessage::Ping);
            }
        }
        */
        let secs = Duration::from_secs(1);
        thread::sleep(secs);
    }
}

pub fn send_message(state: &WebGui, msg: Ws::Message) {
    panic!("not yet implemented");
}
