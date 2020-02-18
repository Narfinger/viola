use actix::prelude::*;
use actix::{Actor, StreamHandler};
use actix_files as fs;
use actix_rt;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::io;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::libraryviewstore;
use crate::loaded_playlist::LoadedPlaylistExt;
use crate::my_websocket;
use crate::my_websocket::*;
use crate::playlist::restore_playlists;
use crate::playlist_tabs;
use crate::smartplaylist_parser;
use crate::types::*;

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

#[post("/libraryview/partial/")]
async fn library_partial_tree(
    state: web::Data<WebGui>,
    level: web::Json<libraryviewstore::PartialQueryLevel>,
    req: HttpRequest,
) -> HttpResponse {
    let q = level.into_inner();
    let items = libraryviewstore::query_partial_tree(&state.pool, &q);
    //println!("items: {:?}", items);
    HttpResponse::Ok().json(items)
}

#[post("/libraryview/load/")]
async fn library_load(
    state: web::Data<WebGui>,
    level: web::Json<libraryviewstore::PartialQueryLevel>,
    req: HttpRequest,
) -> HttpResponse {
    let q = level.into_inner();
    let pl = libraryviewstore::load_query(&state.pool, &q);
    *state.playlist.write().unwrap() = pl;
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

use futures::{Future, Stream, StreamExt};
#[post("/libraryview/full/")]
async fn library_full_tree(
    state: web::Data<WebGui>,
    req: HttpRequest,
    //level: web::Json<libraryviewstore::PartialQueryLevel>,
    mut body: web::Payload,
) -> HttpResponse {
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }
    format!("Body {:?}!", bytes);
    let q = serde_json::from_slice::<libraryviewstore::PartialQueryLevel>(&bytes);
    println!("{:?}", q);

    //println!("{:?}", level);
    //let q = level.into_inner();
    //let items = libraryviewstore::query_tree(&state.pool, &q);
    //Ok(HttpResponse::Ok().json(items))
    let items: Vec<String> = Vec::new();
    HttpResponse::Ok().json(items)
}

#[get("/smartplaylist/")]
fn smartplaylist(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    let spl = smartplaylist_parser::construct_smartplaylists_from_config()
        .into_iter()
        .map(|pl| GeneralTreeViewJson::<String> {
            value: pl.name,
            children: Vec::new(),
            optional: None,
        })
        .collect::<Vec<GeneralTreeViewJson<String>>>();
    HttpResponse::Ok().json(spl)
}

#[post("/smartplaylist/load/")]
async fn smartplaylist_load(
    state: web::Data<WebGui>,
    //index: web::Json<usize>,
    req: HttpRequest,
    mut body: web::Payload,
) -> HttpResponse {
    use crate::smartplaylist_parser::LoadSmartPlaylist;
    let mut bytes = web::BytesMut::new();
    while let Some(item) = body.next().await {
        bytes.extend_from_slice(&item.unwrap());
    }
    let q = serde_json::from_slice::<usize>(&bytes).expect("Error in parsing");

    let spl = smartplaylist_parser::construct_smartplaylists_from_config();
    let pl = spl.get(q);

    if let Some(p) = pl {
        let rp = p.load(&state.pool);
        *state.playlist.write().unwrap() = rp;
        my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);
    }

    HttpResponse::Ok().finish()
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
    ws: RwLock<Option<my_websocket::MyWs>>,
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

fn handle_gstreamer_messages(
    state: web::Data<WebGui>,
    rx: Receiver<crate::gstreamer_wrapper::GStreamerMessage>,
) {
    loop {
        //println!("loop is working");
        if let Ok(msg) = rx.try_recv() {
            match msg {
                crate::gstreamer_wrapper::GStreamerMessage::Playing => {
                    let pos = state.playlist.current_position();
                    my_websocket::send_my_message(&state.ws, WsMessage::PlayChanged { index: pos });
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
    //let mut sys = actix_rt::System::new("test");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(playlist)
            .service(current_id)
            .service(clean)
            .service(transport)
            //.service(web::resource("/libraryview/albums/").route(web::get().to(library_albums)))
            //.service(web::resource("/libraryview/tracks/").route(web::get().to(library_tracks)))
            .service(library_partial_tree)
            .service(library_full_tree)
            .service(library_load)
            .service(smartplaylist)
            .service(smartplaylist_load)
            .service(web::resource("/ws/").route(web::get().to(ws_start)))
            .service(fs::Files::new("/static/", "web_gui/dist/").show_files_listing())
            .service(fs::Files::new("/", "./web_gui/").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")
    .expect("Cannot bind address")
    .run()
    .await;

    //sys.block_on(server);

    Ok(())
}
