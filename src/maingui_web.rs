use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use std::fmt;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::loaded_playlist::LoadedPlaylistExt;
use crate::playlist::restore_playlists;
use crate::playlist_tabs;
use crate::types::*;

#[derive(Clone, Message, Serialize)]
#[serde(tag = "type")]
#[rtype(result = "()")]
enum WsMessage {
    PlayChanged { index: usize },
    Ping,
}

impl From<WsMessage> for String {
    fn from(msg: WsMessage) -> Self {
        serde_json::to_string(&msg).unwrap()
    }
}

#[derive(Clone)]
struct MyWs {
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

impl StreamHandler<ws::Message, ws::ProtocolError> for MyWs {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Text(b) => println!("we found text {}", b),
            _ => {}
        }
        //self.addr.unwrap().do_send(msg.unwrap());
        //println!("We want to handle");
    }
}

fn ws_start(
    state: web::Data<WebGui>,
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

fn playlist(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    let items = &*state.playlist.items();
    HttpResponse::Ok().json(items)
}

// removes all already played data
fn clean(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    state.playlist.clean();

    //reload playlist
    HttpResponse::Ok().finish()
}

fn transport(
    state: web::Data<WebGui>,
    msg: web::Json<gstreamer_wrapper::GStreamerAction>,
) -> HttpResponse {
    println!("stuff: {:?}", &msg);
    state.gstreamer.do_gstreamer_action(&msg);

    HttpResponse::Ok().finish()
}

fn current_id(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(state.playlist.current_position())
}

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<gstreamer_wrapper::GStreamer>,
    playlist: LoadedPlaylistPtr,
    ws: RwLock<Option<MyWs>>,
}

fn handle_gstreamer_messages(
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

pub fn run(pool: DBPool) {
    println!("Loading playlist");
    let lp = Arc::new(RwLock::new(
        restore_playlists(&pool)
            .expect("Error restoring playlisttabs")
            .swap_remove(0),
    ));

    println!("Starting gstreamer");
    let (gst, rx) =
        gstreamer_wrapper::new(lp.clone(), pool.clone()).expect("Error Initializing gstreamer");

    println!("Setting up gui");
    let state = WebGui {
        pool: pool.clone(),
        gstreamer: gst,
        playlist: lp,
        ws: RwLock::new(None),
    };

    println!("Doing data");
    let data = web::Data::new(state);

    {
        let datac = data.clone();
        thread::spawn(move || handle_gstreamer_messages(datac, rx));
    }

    println!("Starting web gui on 127.0.0.1:8088");
    HttpServer::new(move || {
        App::new()
            .register_data(data.clone())
            .service(web::resource("/playlist/").route(web::get().to(playlist)))
            .service(web::resource("/currentid/").route(web::get().to(current_id)))
            .service(web::resource("/clean/").route(web::post().to(clean)))
            .service(web::resource("/transport/").route(web::post().to(transport)))
            .service(web::resource("/ws/").route(web::get().to(ws_start)))
            .service(fs::Files::new("/static/", "web_gui/dist/").show_files_listing())
            .service(fs::Files::new("/", "./web_gui/").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
