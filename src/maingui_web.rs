use diesel::Connection;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{convert::Infallible, io};
use tokio::sync::RwLock;
use viola_common::*;
use warp::{body::json, reply::Json, Filter, Reply};

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::libraryviewstore;
use crate::loaded_playlist::{LoadedPlaylistExt, PlaylistControls, SavePlaylistExt};
use crate::my_websocket;
use crate::my_websocket::*;
use crate::playlist_tabs::PlaylistTabsExt;
use crate::smartplaylist_parser;
use crate::types::*;

async fn playlist(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let items = state.read().await.playlist_tabs.items();
    Ok(warp::reply::json(&items))
}

async fn playlist_for(index: usize, state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let items = state.read().await.playlist_tabs.items_for(index);
    Ok(warp::reply::json(&items))
}

async fn repeat(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    state
        .read()
        .await
        .gstreamer
        .write()
        .do_gstreamer_action(viola_common::GStreamerAction::RepeatOnce);
    Ok(warp::reply())
}

/// removes all already played data
async fn clean(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    println!("doing cleaning");
    state.write().await.playlist_tabs.clean();
    //my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    Ok(warp::reply())
}

/*#[delete("/deletefromplaylist/")]
async fn delete_from_playlist(
    state: WebGuiData,
    deleterange: web::Json<std::ops::Range<usize>>,
    _: HttpRequest,
) -> HttpResponse {
    println!("Doing delete");
    state.playlist_tabs.delete_range(deleterange.into_inner());
    my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}
*/

async fn save(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    println!("Saving");
    let read_lock = state.read().await;
    let db = read_lock.pool.lock();
    read_lock.playlist_tabs.save(&db).expect("Error in saving");
    Ok(warp::reply())
}

async fn get_transport(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(
        &state.read().await.gstreamer.read().get_state(),
    ))
}

async fn transport(
    msg: viola_common::GStreamerAction,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    info!("state json data: {:?}", &msg);
    state
        .read()
        .await
        .gstreamer
        .write()
        .do_gstreamer_action(msg);
    Ok(warp::reply())
}
/*
#[post("/libraryview/partial/")]
async fn library_partial_tree(
    state: WebGuiData,
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
    state: WebGuiData,
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
fn smartplaylist(_: WebGuiData, _: HttpRequest) -> HttpResponse {
    let spl = smartplaylist_parser::construct_smartplaylists_from_config()
        .into_iter()
        .map(|pl| pl.name)
        .collect::<Vec<String>>();
    HttpResponse::Ok().json(spl)
}

#[post("/smartplaylist/load/")]
async fn smartplaylist_load(
    state: WebGuiData,
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
*/

async fn current_id(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(
        &state.read().await.playlist_tabs.current_position(),
    ))
}

/*
#[get("/pltime/")]
async fn pltime(state: WebGuiData, _: HttpRequest) -> HttpResponse {
    let total_length = state.playlist_tabs.get_remaining_length();
    let dur = Duration::new(total_length, 0);
    let time = humantime::format_duration(dur).to_string();
    HttpResponse::Ok().json(time)
}

#[get("/currentimage/")]
async fn current_image(state: WebGuiData, req: HttpRequest) -> HttpResponse {
    state
        .playlist_tabs
        .get_current_track()
        .albumpath
        .and_then(|p| actix_files::NamedFile::open(p).ok())
        .and_then(|f: actix_files::NamedFile| f.into_response(&req).ok())
        .unwrap_or_else(|| HttpResponse::Ok().finish())
}
*/

async fn playlist_tab(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let tabs = state
        .read()
        .await
        .playlist_tabs
        .read()
        .pls
        .iter()
        .map(|pl| PlaylistTabJSON {
            name: pl.read().name.to_owned(),
            current_position: pl.read().current_position,
        })
        .collect::<Vec<PlaylistTabJSON>>();
    let resp = PlaylistTabsJSON {
        current: state.read().await.playlist_tabs.current_tab(),
        tabs: tabs,
    };
    Ok(warp::reply::json(&resp))
}

/*
#[post("/playlisttab/")]
async fn change_playlist_tab(
    state: WebGuiData,
    index: web::Json<usize>,
    _: HttpRequest,
) -> HttpResponse {
    state.playlist_tabs.set_tab(index.into_inner());
    my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    HttpResponse::Ok().finish()
}

#[delete("/playlisttab/")]
async fn delete_playlist_tab(
    state: WebGuiData,
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
*/

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<parking_lot::RwLock<gstreamer_wrapper::GStreamer>>,
    playlist_tabs: PlaylistTabsPtr,
    ws: parking_lot::RwLock<Option<my_websocket::MyWs>>,
}

impl WebGui {
    fn save(&self) {
        let db = self.pool.lock();
        db.transaction::<_, diesel::result::Error, _>(|| {
            self.playlist_tabs.save(&*db)?;
            Ok(())
        })
        .expect("Error in saving");
    }
}

type WebGuiData = Arc<RwLock<WebGui>>;

/*
async fn ws_start(
    state: WebGuiData,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let mut ws = MyWs { addr: None };
    let (addr, resp) = ws::start_with_addr(ws.clone(), &req, stream)?;
    //println!("websocket {:?}", resp);
    ws.addr = Some(addr);
    *state.ws.write() = Some(ws);
    Ok(resp)
}
*/

/// blocking function that handles messages on the GStreamer Bus
async fn handle_gstreamer_messages(
    state: WebGuiData,
    rx: &mut bus::BusReader<viola_common::GStreamerMessage>,
) {
    for msg in rx.iter() {
        println!("received gstreamer message on own bus: {:?}", msg);
        match msg {
            viola_common::GStreamerMessage::Playing => {
                let pos = state.read().await.playlist_tabs.current_position();
                //my_websocket::send_my_message(&state.ws, WsMessage::PlayChanged(pos));
            }
            _ => (),
        }
    }
}

async fn auto_save(state: WebGuiData) {
    loop {
        tokio::time::sleep(Duration::new(10 * 60, 0));
        state.read().await.save();
    }
}

pub async fn run(pool: DBPool) {
    println!("Loading playlist");
    let plt = crate::playlist_tabs::load(&pool).expect("Failure to load old playlists");

    println!("Starting gstreamer");
    let mut bus = bus::Bus::new(50);
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
        ws: parking_lot::RwLock::new(None),
    };

    println!("Doing data");
    let data = Arc::new(RwLock::new(state));

    {
        let datac = data.clone();
        tokio::spawn(handle_gstreamer_messages(datac, &mut websocket_recv));
    }
    {
        let datac = data.clone();
        tokio::spawn(async move { auto_save(datac) });
    }
    {
        let datac = data.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::new(1, 0));
                if datac.read().await.gstreamer.read().get_state()
                    == viola_common::GStreamerMessage::Playing
                {
                    let data = datac
                        .read()
                        .await
                        .gstreamer
                        .read()
                        .get_elapsed()
                        .unwrap_or(0);
                    //my_websocket::send_my_message(&datac.ws, WsMessage::CurrentTimeChanged(data));
                }
            }
        });
    }

    println!("Starting web gui on 127.0.0.1:8088");
    //let mut sys = actix_rt::System::new("test");

    //let web_gui_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/");
    let web_gui_dist_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/dist/");

    let data = warp::any().map(move || Arc::clone(&data));

    let gets = {
        let pl = warp::path!("/playlist/" / usize)
            .and(data.clone())
            .and_then(playlist_for);
        let tr = warp::path!("/transport/")
            .and(data.clone())
            .and_then(get_transport);
        let curid = warp::path!("/currentid/")
            .and(data.clone())
            .and_then(current_id);
        let pltab = warp::path!("/playlisttab/")
            .and(data.clone())
            .and_then(playlist_tab);
        warp::get().and(pl.or(tr).or(curid).or(pltab))
    };

    let posts = {
        let rep = warp::path!("/repeat/").and(data.clone()).and_then(repeat);
        let clean = warp::path!("/clean/").and(data.clone()).and_then(clean);
        let save = warp::path!("/save/").and(data.clone()).and_then(save);
        let transp = warp::path!("/transport/")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(transport);
        warp::post().and(rep.or(clean).or(save).or(transp))
    };

    let static_files = warp::path("/static/").and(warp::fs::dir("/static/"));
    let index = warp::path("/").and(warp::fs::file("index.html"));

    let all = gets.or(posts).or(static_files).or(index);
    warp::serve(all).run(([127, 0, 0, 1], 8088)).await;
}
