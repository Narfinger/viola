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
use crate::loaded_playlist::{LoadedPlaylistExt, PlaylistControls, SavePlaylistExt};
use crate::my_websocket;
use crate::my_websocket::*;
use crate::playlist_tabs::PlaylistTabsExt;
use crate::smartplaylist_parser;
use crate::types::*;

#[get("/playlist/")]
async fn playlist(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    let items = state.playlist_tabs.items();
    HttpResponse::Ok().body(items)
}

#[get("/playlist/{index}/")]
async fn playlist_for(
    state: web::Data<WebGui>,
    web::Path(index): web::Path<usize>,
    _: HttpRequest,
) -> HttpResponse {
    let items = state.playlist_tabs.items_for(index);
    HttpResponse::Ok().body(items)
}

#[delete("/playlist/")]
async fn playlist_delete_range(
    state: web::Data<WebGui>,
    _: HttpRequest,
    msg: web::Json<Range>,
) -> HttpResponse {
    println!("Deleting range: {:?}", &msg);
    state.playlist_tabs.delete_range(msg.into_inner());
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);

    let db = state.pool.lock().expect("Error from db");
    state.playlist_tabs.save(&db).expect("Error in saving");
    HttpResponse::Ok().finish()
}

#[post("/repeat/")]
async fn repeat(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    state
        .gstreamer
        .do_gstreamer_action(viola_common::GStreamerAction::RepeatOnce);
    HttpResponse::Ok().finish()
}

// removes all already played data
#[post("/clean/")]
async fn clean(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    println!("doing cleaning");
    state.playlist_tabs.clean();
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

#[delete("/deletefromplaylist/")]
async fn delete_from_playlist(
    state: web::Data<WebGui>,
    deleterange: web::Json<Range>,
    _: HttpRequest,
) -> HttpResponse {
    println!("Doing delete");
    state.playlist_tabs.delete_range(deleterange.into_inner());
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

#[post("/save/")]
async fn save(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    println!("Saving");
    let db = state.pool.lock().expect("Error for db");
    state.playlist_tabs.save(&db).expect("Error in saving");
    HttpResponse::Ok().finish()
}

#[get("/transport/")]
async fn get_transport(state: web::Data<WebGui>) -> HttpResponse {
    HttpResponse::Ok().json(state.gstreamer.get_state())
}

#[post("/transport/")]
async fn transport(
    state: web::Data<WebGui>,
    msg: web::Json<viola_common::GStreamerAction>,
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
    let pl = libraryviewstore::load_query(&state.pool, &q);
    println!("Loading new playlist {}", pl.name);
    state.playlist_tabs.add(pl);
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadTabs);
    HttpResponse::Ok().finish()
}

// use futures::StreamExt;
// #[post("/libraryview/full/")]
// async fn library_full_tree(
//     state: web::Data<WebGui>,
//     req: HttpRequest,
//     //level: web::Json<libraryviewstore::PartialQueryLevel>,
//     mut body: web::Payload,
// ) -> HttpResponse {
//     let mut bytes = web::BytesMut::new();
//     while let Some(item) = body.next().await {
//         bytes.extend_from_slice(&item.unwrap());
//     }
//     format!("Body {:?}!", bytes);
//     let q = serde_json::from_slice::<libraryviewstore::PartialQueryLevel>(&bytes);
//     println!("{:?}", q);

//     //println!("{:?}", level);
//     //let q = level.into_inner();
//     //let items = libraryviewstore::query_tree(&state.pool, &q);
//     //Ok(HttpResponse::Ok().json(items))
//     let items: Vec<String> = Vec::new();
//     HttpResponse::Ok().json(items)
// }

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
    index: web::Json<LoadSmartPlaylistJson>,
    _: HttpRequest,
) -> HttpResponse {
    use crate::smartplaylist_parser::LoadSmartPlaylist;
    let spl = smartplaylist_parser::construct_smartplaylists_from_config();
    let pl = spl.get(index.index);

    if let Some(p) = pl {
        let rp = p.load(&state.pool);
        state.playlist_tabs.add(rp);
        my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadTabs);
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
    HttpResponse::Ok().json(state.playlist_tabs.current_position())
}

#[get("/pltime/")]
async fn pltime(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    let total_length = state.playlist_tabs.get_remaining_length();
    let dur = Duration::new(total_length, 0);
    let time = humantime::format_duration(dur).to_string();
    HttpResponse::Ok().json(time)
}

#[get("/currentimage/")]
async fn current_image(state: web::Data<WebGui>, req: HttpRequest) -> HttpResponse {
    state
        .playlist_tabs
        .get_current_track()
        .albumpath
        .and_then(|p| actix_files::NamedFile::open(p).ok())
        .and_then(|f: actix_files::NamedFile| f.into_response(&req).ok())
        .unwrap_or_else(|| HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize)]
struct PlaylistTabsJSON {
    current: usize,
    current_playing_in: usize,
    tabs: Vec<String>,
}

#[get("/playlisttab/")]
async fn playlist_tab(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    let strings = state
        .playlist_tabs
        .read()
        .unwrap()
        .pls
        .iter()
        .map(|pl| pl.read().unwrap().name.to_owned())
        .collect::<Vec<String>>();
    let resp = PlaylistTabsJSON {
        current: state.playlist_tabs.current_tab(),
        current_playing_in: state.playlist_tabs.current_playing_in(),
        tabs: strings,
    };

    //state.save();

    HttpResponse::Ok().json(resp)
}

#[post("/playlisttab/")]
async fn change_playlist_tab(
    state: web::Data<WebGui>,
    level: web::Json<ChangePlaylistTabJson>,
    _: HttpRequest,
) -> HttpResponse {
    let max = state.playlist_tabs.read().unwrap().pls.len();
    info!("setting to: {}, max: {}", level.index, max - 1);
    state.playlist_tabs.write().unwrap().current_pl = std::cmp::min(max - 1, level.index);
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

#[delete("/playlisttab/")]
async fn delete_playlist_tab(
    state: web::Data<WebGui>,
    index: web::Json<ChangePlaylistTabJson>,
    _: HttpRequest,
    //mut body: web::Payload,
) -> HttpResponse {
    //use futures::StreamExt;
    //let mut bytes = web::BytesMut::new();
    //while let Some(item) = body.next().await {
    //    bytes.extend_from_slice(&item.unwrap());
    //}
    //println!("Body {:?}!", bytes);
    //let q = serde_json::from_slice::<ChangePlaylistTabJson>(&bytes);
    //println!("{:?}", q);

    println!("deleting {}", index.index);
    state.playlist_tabs.delete(&state.pool, index.index);
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadTabs);
    my_websocket::send_my_message(&state.ws, my_websocket::WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<gstreamer_wrapper::GStreamer>,
    playlist_tabs: PlaylistTabsPtr,
    ws: RwLock<Option<my_websocket::MyWs>>,
}

trait Web {
    fn save(&self);
}

impl Web for WebGui {
    fn save(&self) {
        let db = self.pool.lock().expect("DB Error");
        db.transaction::<_, diesel::result::Error, _>(|| {
            self.playlist_tabs.save(&*db)?;
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
    //println!("websocket {:?}", resp);
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
                    let pos = state.playlist_tabs.current_position();
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
    let plt = crate::playlist_tabs::load(&pool).expect("Failure to load old playlists");

    println!("Starting gstreamer");
    let (gst, rx) =
        gstreamer_wrapper::new(plt.clone(), pool.clone()).expect("Error Initializing gstreamer");

    println!("Setting up gui");
    let state = WebGui {
        pool: pool.clone(),
        gstreamer: gst,
        playlist_tabs: plt,
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
    {
        let datac = data.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::new(1, 0));
            if datac.gstreamer.get_state() == crate::gstreamer_wrapper::GStreamerMessage::Playing {
                let data = datac.gstreamer.get_elapsed().unwrap_or(0);
                my_websocket::send_my_message(
                    &datac.ws,
                    WsMessage::CurrentTimeChanged { index: data },
                );
            }
        });
    }

    println!("Starting web gui on 127.0.0.1:8088");
    //let mut sys = actix_rt::System::new("test");

    let web_gui_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/");
    let web_gui_dist_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/dist/");
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(playlist)
            .service(playlist_for)
            .service(repeat)
            .service(clean)
            .service(delete_from_playlist)
            .service(save)
            .service(transport)
            .service(get_transport)
            //.service(web::resource("/libraryview/albums/").route(web::get().to(library_albums)))
            //.service(web::resource("/libraryview/tracks/").route(web::get().to(library_tracks)))
            .service(library_partial_tree)
            .service(library_load)
            .service(smartplaylist)
            .service(smartplaylist_load)
            .service(pltime)
            .service(current_id)
            .service(current_image)
            .service(playlist_tab)
            .service(change_playlist_tab)
            .service(delete_playlist_tab)
            .service(web::resource("/ws/").route(web::get().to(ws_start)))
            .service(fs::Files::new("/static/", web_gui_dist_path).show_files_listing())
            .service(fs::Files::new("/", web_gui_dist_path).index_file("index.html"))
    })
    .bind("127.0.0.1:8088")
    .expect("Cannot bind address")
    .run()
    .await
    .expect("Running server");

    println!("I can probably remove the arc and rwlock for playlists and just use");

    //sys.block_on(server);

    Ok(())
}
