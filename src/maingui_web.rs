use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::playlist::restore_playlists;
use crate::playlist_tabs;
use crate::types::*;

#[derive(Clone, Message)]
#[rtype(result = "()")]
enum WsMessage {
    PlayChanged(usize),
    Ping,
}

struct MyWs {
    addr: Option<Addr<Self>>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<WsMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut ws::WebsocketContext<Self>) -> Self::Result {
        match msg {
            WsMessage::PlayChanged(i) => {
                ctx.text(format!("playchanged {}", i));
            }
            WsMessage::Ping => {
                ctx.text("bl");
            }
        }
        println!("We want to handle");
    }
}

fn ws_start(
    state: web::Data<WebGui>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let ws = MyWs { addr: None };
    let resp = ws::start(ws, &req, stream)?;
    println!("websocket {:?}", resp);
    *state.ws.write().unwrap() = Some(ws);
    Ok(resp)
}

fn playlist(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(&state.playlist.items)
}

fn play(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    state
        .gstreamer
        .do_gstreamer_action(&gstreamer_wrapper::GStreamerAction::Playing);
    HttpResponse::Ok().finish()
}

fn pause(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    state
        .gstreamer
        .do_gstreamer_action(&gstreamer_wrapper::GStreamerAction::Pausing);
    HttpResponse::Ok().finish()
}

fn current_id(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(state.playlist.current_position.load(Ordering::Relaxed))
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
        println!("loop is working");
        if let Ok(msg) = rx.try_recv() {
            match msg {
                gstreamer_wrapper::GStreamerMessage::Playing => {
                    let pos = state.playlist.current_position.load(Ordering::Relaxed);
                    state
                        .ws
                        .read()
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .do_send(WsMessage::PlayChanged(pos))
                }
                _ => (),
            }
        }

        if let Some(a) = state.ws.read().unwrap().as_ref() {
            println!("Sending ping");
            a.send(WsMessage::Ping).wait().expect("Error in future");
        }

        let secs = Duration::from_secs(1);
        thread::sleep(secs);
    }
}

pub fn run(pool: DBPool) {
    println!("Loading playlist");
    let lp = Arc::new(
        restore_playlists(&pool)
            .expect("Error restoring playlisttabs")
            .swap_remove(0),
    );

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
            .service(web::resource("/transport/play").route(web::get().to(play)))
            .service(web::resource("/transport/pause").route(web::get().to(pause)))
            .service(web::resource("/ws/").route(web::get().to(ws_start)))
            .service(fs::Files::new("/static/", "web_gui/dist/").show_files_listing())
            .service(fs::Files::new("/", "./web_gui/").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .unwrap();
}
