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
use std::{env, io};

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::libraryviewstore;
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

async fn ws_start(
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

#[get("/playlist/")]
async fn playlist(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    let items = &*state.playlist.items();
    HttpResponse::Ok().json(items)
}

// removes all already played data
#[get("/clean/")]
async fn clean(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    state.playlist.clean();

    //reload playlist
    HttpResponse::Ok().finish()
}

#[post("/transport/")]
async fn transport(
    state: web::Data<WebGui>,
    msg: web::Json<gstreamer_wrapper::GStreamerAction>,
) -> HttpResponse {
    println!("stuff: {:?}", &msg);
    state.gstreamer.do_gstreamer_action(&msg);

    HttpResponse::Ok().finish()
}

#[get("/libraryview/artist/")]
async fn library_artist(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    let items = libraryviewstore::get_artist_trees(&state.pool);
    //println!("items: {:?}", items);
    HttpResponse::Ok().json(items)
}

#[derive(Deserialize, Serialize)]
struct Q {
    artist: Option<String>,
    album: Option<String>,
    track: Option<String>,
}

#[get("/libraryview/querytree/")]
async fn library_tree(
    state: web::Data<WebGui>,
    q: web::Query<Q>,
    req: HttpRequest,
) -> HttpResponse {
    let items = libraryviewstore::query_tree(&state.pool, &q.artist, &q.album, &q.track);
    HttpResponse::Ok().json(items)
}

#[get("/libraryview/albums/{name}")]
fn library_albums(
    state: web::Data<WebGui>,
    req: HttpRequest,
    path: web::Path<String>,
) -> HttpResponse {
    let name = path.into_inner();
    let db = state.pool.lock().expect("Error in db locking");
    println!("getting album with: {:?}", name);
    let items = libraryviewstore::get_album_subtree(&db, Some(&name));
    //println!("{:?}", items);
    HttpResponse::Ok().json(items)
}

/*fn library_tracks(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    let items = libraryviewstore::get_tracks(&state.pool);
    //println!("{:?}", items);
    HttpResponse::Ok().json(items)
}*/

#[get("/currentid/")]
async fn current_id(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
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

pub async fn run(pool: DBPool) -> io::Result<()> {
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
            .app_data(data.clone())
            .service(playlist)
            .service(current_id)
            .service(clean)
            .service(transport)
            .service(library_artist)
            .service(library_albums)
            //.service(web::resource("/libraryview/albums/").route(web::get().to(library_albums)))
            //.service(web::resource("/libraryview/tracks/").route(web::get().to(library_tracks)))
            .service(library_tree)
            .service(web::resource("/ws/").route(web::get().to(ws_start)))
            .service(fs::Files::new("/static/", "web_gui/dist/").show_files_listing())
            .service(fs::Files::new("/", "./web_gui/").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")
    .expect("Cannot bind address")
    .run()
    .await
}
