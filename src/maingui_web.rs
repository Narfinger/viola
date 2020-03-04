use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use diesel::Connection;
use std::io;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::libraryviewstore;
use crate::loaded_playlist::{LoadedPlaylistExt, SavePlaylistExt};
use crate::my_websocket;
use crate::my_websocket::*;
use crate::playlist_tabs::{load, PlaylistTabs, PlaylistTabsExt};
use crate::smartplaylist_parser;
use crate::types::*;

#[get("/playlist/")]
async fn playlist(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    let items = state.playlist.items();
    HttpResponse::Ok().json(&*items)
}

#[post("/repeat/")]
async fn repeat(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    state
        .gstreamer
        .do_gstreamer_action(gstreamer_wrapper::GStreamerAction::RepeatOnce);
    HttpResponse::Ok().finish()
}

// removes all already played data
#[post("/clean/")]
async fn clean(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    println!("doing cleaning");
    state.playlist.clean();
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

#[post("/save/")]
async fn save(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    println!("Saving");
    let db = state.pool.lock().expect("Error for db");
    state.playlist.save(&db).expect("Error in saving");
    HttpResponse::Ok().finish()
}

#[get("/transport/")]
async fn get_transport(state: web::Data<WebGui>) -> HttpResponse {
    HttpResponse::Ok().json(state.gstreamer.get_state())
}

#[post("/transport/")]
async fn transport(
    state: web::Data<WebGui>,
    msg: web::Json<gstreamer_wrapper::GStreamerAction>,
) -> HttpResponse {
    println!("stuff: {:?}", &msg);
    state.gstreamer.do_gstreamer_action(msg.into_inner());

    HttpResponse::Ok().finish()
}

#[post("/libraryview/partial/")]
async fn library_partial_tree(
    state: web::Data<WebGui>,
    level: web::Json<libraryviewstore::PartialQueryLevel>,
    _: HttpRequest,
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
    _: HttpRequest,
) -> HttpResponse {
    let q = level.into_inner();
    let pl = RwLock::new(libraryviewstore::load_query(&state.pool, &q));
    state.playlist.add(pl);
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

use futures::StreamExt;
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
fn smartplaylist(_: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
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
    _: HttpRequest,
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
        let rp = RwLock::new(p.load(&state.pool));
        state.playlist.add(rp);
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
async fn current_id(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(state.playlist.current_position())
}

#[get("/pltime/")]
async fn pltime(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    let current_position = state.playlist.current_position();
    let total_length = state.playlist.get_remaining_length();
    let dur = Duration::new(total_length, 0);
    let time = humantime::format_duration(dur).to_string();
    HttpResponse::Ok().json(time)
}

#[get("/currentimage/")]
async fn current_image(
    state: web::Data<WebGui>,
    _: HttpRequest,
) -> actix_web::Result<actix_files::NamedFile> {
    let current_track_album = state.playlist.get_current_track().albumpath.expect("error");
    println!("asking for image {:?}", &current_track_album);

    Ok(actix_files::NamedFile::open(current_track_album)?)
}

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<gstreamer_wrapper::GStreamer>,
    playlist: PlaylistTabsPtr,
    ws: RwLock<Option<my_websocket::MyWs>>,
}

trait Web {
    fn save(&self);
}

impl Web for WebGui {
    fn save(&self) {
        let db = self.pool.lock().expect("DB Error");
        db.transaction::<_, diesel::result::Error, _>(|| {
            self.playlist.save(&*db)?;
            Ok(())
        })
        .expect("Error in saving");
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

fn handle_gstreamer_messages(
    state: web::Data<WebGui>,
    rx: Receiver<crate::gstreamer_wrapper::GStreamerMessage>,
) {
    loop {
        //println!("loop is working");
        if let Ok(msg) = rx.try_recv() {
            println!("received message: {:?}", msg);
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
    let lp = crate::playlist_tabs::load(&pool).expect("Failure to load old playlists");

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
    {
        let datac = data.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::new(10 * 60, 0));
            datac.save();
        });
    }

    println!("Starting web gui on 127.0.0.1:8088");
    //let mut sys = actix_rt::System::new("test");
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(playlist)
            .service(repeat)
            .service(clean)
            .service(save)
            .service(transport)
            .service(get_transport)
            //.service(web::resource("/libraryview/albums/").route(web::get().to(library_albums)))
            //.service(web::resource("/libraryview/tracks/").route(web::get().to(library_tracks)))
            .service(library_partial_tree)
            .service(library_full_tree)
            .service(library_load)
            .service(smartplaylist)
            .service(smartplaylist_load)
            .service(pltime)
            .service(current_id)
            .service(current_image)
            .service(web::resource("/ws/").route(web::get().to(ws_start)))
            .service(fs::Files::new("/static/", "web_gui/dist/").show_files_listing())
            .service(fs::Files::new("/", "./web_gui/").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")
    .expect("Cannot bind address")
    .run()
    .await;

    println!("I can probably remove the arc and rwlock for playlists and just use");

    //sys.block_on(server);

    Ok(())
}
