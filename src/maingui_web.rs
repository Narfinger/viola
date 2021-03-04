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
use viola_common::*;

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

#[post("/repeat/")]
async fn repeat(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    state
        .gstreamer
        .write()
        .unwrap()
        .do_gstreamer_action(viola_common::GStreamerAction::RepeatOnce);
    HttpResponse::Ok().finish()
}

// removes all already played data
#[post("/clean/")]
async fn clean(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    println!("doing cleaning");
    state.playlist_tabs.clean();
    my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

#[delete("/deletefromplaylist/")]
async fn delete_from_playlist(
    state: web::Data<WebGui>,
    deleterange: web::Json<std::ops::Range<usize>>,
    _: HttpRequest,
) -> HttpResponse {
    println!("Doing delete");
    state.playlist_tabs.delete_range(deleterange.into_inner());
    my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
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
    HttpResponse::Ok().json(state.gstreamer.read().unwrap().get_state())
}

#[post("/transport/")]
async fn transport(
    state: web::Data<WebGui>,
    msg: web::Json<viola_common::GStreamerAction>,
) -> HttpResponse {
    println!("stuff: {:?}", &msg);
    state
        .gstreamer
        .write()
        .unwrap()
        .do_gstreamer_action(msg.into_inner());

    HttpResponse::Ok().finish()
}

#[post("/libraryview/partial/")]
async fn library_partial_tree(
    state: web::Data<WebGui>,
    level: web::Json<viola_common::TreeViewQuery>,
    _: HttpRequest,
) -> HttpResponse {
    let mut q = level.into_inner();
    if q.search.is_some() && q.search.as_ref().unwrap().is_empty() {
        q.search = None;
    }
    let items = libraryviewstore::partial_query(&state.pool, &q);

    HttpResponse::Ok().json(items)
}

#[post("/libraryview/full/")]
async fn library_load(
    state: web::Data<WebGui>,
    level: web::Json<viola_common::TreeViewQuery>,
    _: HttpRequest,
) -> HttpResponse {
    let mut q = level.into_inner();
    q.search = q.search.filter(|t| !t.is_empty());
    let pl = libraryviewstore::load_query(&state.pool, &q);
    println!("Loading new playlist {}", pl.name);
    state.playlist_tabs.add(pl);
    my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs);
    HttpResponse::Ok().finish()
}

#[get("/smartplaylist/")]
fn smartplaylist(_: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    let spl = smartplaylist_parser::construct_smartplaylists_from_config()
        .into_iter()
        .map(|pl| pl.name)
        .collect::<Vec<String>>();
    HttpResponse::Ok().json(spl)
}

#[post("/smartplaylist/load/")]
async fn smartplaylist_load(
    state: web::Data<WebGui>,
    index: web::Json<viola_common::LoadSmartPlaylistJson>,
    _: HttpRequest,
) -> HttpResponse {
    use crate::smartplaylist_parser::LoadSmartPlaylist;
    let spl = smartplaylist_parser::construct_smartplaylists_from_config();
    let pl = spl.get(index.index);

    if let Some(p) = pl {
        let rp = p.load(&state.pool);
        state.playlist_tabs.add(rp);
        my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs);
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

#[get("/playlisttab/")]
async fn playlist_tab(state: web::Data<WebGui>, _: HttpRequest) -> HttpResponse {
    let tabs = state
        .playlist_tabs
        .read()
        .unwrap()
        .pls
        .iter()
        .map(|pl| PlaylistTabJSON {
            name: pl.read().unwrap().name.to_owned(),
            current_position: pl.read().unwrap().current_position,
        })
        .collect::<Vec<PlaylistTabJSON>>();
    let resp = PlaylistTabsJSON {
        current: state.playlist_tabs.current_tab(),
        tabs: tabs,
    };

    //state.save();

    HttpResponse::Ok().json(resp)
}

#[post("/playlisttab/")]
async fn change_playlist_tab(
    state: web::Data<WebGui>,
    index: web::Json<usize>,
    _: HttpRequest,
) -> HttpResponse {
    let max = state.playlist_tabs.read().unwrap().pls.len();
    info!("setting to: {}, max: {}", index, max - 1);
    state.playlist_tabs.write().unwrap().current_pl = std::cmp::min(max - 1, index.into_inner());
    my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

#[delete("/playlisttab/")]
async fn delete_playlist_tab(
    state: web::Data<WebGui>,
    index: web::Json<usize>,
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

    println!("deleting {}", &index);
    state.playlist_tabs.delete(&state.pool, index.into_inner());
    my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs);
    my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<RwLock<gstreamer_wrapper::GStreamer>>,
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
    rx: &mut bus::BusReader<viola_common::GStreamerMessage>,
) {
    loop {
        //println!("loop is working");
        if let Ok(msg) = rx.try_recv() {
            println!("received message: {:?}", msg);
            match msg {
                viola_common::GStreamerMessage::Playing => {
                    let pos = state.playlist_tabs.current_position();
                    my_websocket::send_my_message(&state.ws, WsMessage::PlayChanged(pos));
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

pub async fn run(
    pool: DBPool,
    tx: std::sync::mpsc::Sender<actix_web::dev::Server>,
) -> io::Result<()> {
    println!("Loading playlist");
    let plt = crate::playlist_tabs::load(&pool).expect("Failure to load old playlists");

    println!("Starting gstreamer");
    let mut bus = bus::Bus::new(10);
    let mut websocket_recv = bus.add_rx();
    let dbus_recv = bus.add_rx();
    let gst = gstreamer_wrapper::new(plt.clone(), pool.clone(), bus)
        .expect("Error Initializing gstreamer");

    {
        println!("Starting dbus");
        crate::dbus_interface::new(gst.clone(), plt.clone(), dbus_recv)
    }

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
        thread::spawn(move || handle_gstreamer_messages(datac, &mut websocket_recv));
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
            if datac.gstreamer.read().unwrap().get_state()
                == viola_common::GStreamerMessage::Playing
            {
                let data = datac.gstreamer.read().unwrap().get_elapsed().unwrap_or(0);
                my_websocket::send_my_message(&datac.ws, WsMessage::CurrentTimeChanged(data));
            }
        });
    }

    println!("Starting web gui on 127.0.0.1:8088");
    //let mut sys = actix_rt::System::new("test");

    //let web_gui_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/");
    let web_gui_dist_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/dist/");
    let mut sys = actix_web::rt::System::new("test");

    let srv = HttpServer::new(move || {
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
    .run();

    tx.send(srv.clone()).expect("Error in send");

    sys.block_on(srv).expect("Error in block on");
    Ok(())
}
