use actix::{Actor, StreamHandler};
use actix::AsyncContext;
use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use core::time::Duration;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::playlist::restore_playlists;
use crate::playlist_tabs;
use crate::types::*;

enum WsMessage {
    PlayChanged,
}

struct MyWs {
    recv: Receiver<WsMessage>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.sendmsg(ctx);
    }
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for MyWs {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl MyWs {
    fn sendmsg(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::new(10, 0), |act, ctx| {
            if let Ok(m) = self.recv.recv() {
                match m {
                    PlayChanged => ctx.send_text("playchanged");
                }
            }
        });
    }
}

fn ws_start(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs {}, &req, stream);
    println!("websocket {:?}", resp);
    resp
}

async fn playlist(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
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
}

pub fn run(pool: DBPool) {
    println!("Loading playlist");
    let lp = Arc::new(
        restore_playlists(&pool)
            .expect("Error restoring playlisttabs")
            .swap_remove(0),
    );

    println!("Starting gstreamer");
    let (gst, recv) =
        gstreamer_wrapper::new(lp.clone(), pool.clone()).expect("Error Initializing gstreamer");
    println!("Setting up gui");
    let state = WebGui {
        pool: pool.clone(),
        gstreamer: gst,
        playlist: lp,
    };

    println!("Doing data");
    let data = web::Data::new(state);

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
