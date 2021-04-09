use diesel::Connection;
use futures::{FutureExt, StreamExt};
use std::thread;
use std::time::Duration;
use std::{convert::Infallible, io};
use std::{io::Read, sync::Arc};
use tokio::sync::RwLock;
use viola_common::*;
use warp::{body::json, hyper::StatusCode, reply::Json, Filter, Reply};

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

async fn delete_from_playlist(
    deleterange: std::ops::Range<usize>,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    println!("Doing delete");
    state.read().await.playlist_tabs.delete_range(deleterange);
    //my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    Ok(warp::reply())
}

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

async fn library_partial_tree(
    level: viola_common::TreeViewQuery,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    let mut q = level;
    if q.search.is_some() && q.search.as_ref().unwrap().is_empty() {
        q.search = None;
    }
    let items = libraryviewstore::partial_query(&state.read().await.pool, &q);

    Ok(warp::reply::json(&items))
}

async fn library_load(
    level: viola_common::TreeViewQuery,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    let mut q = level;
    q.search = q.search.filter(|t| !t.is_empty());
    let pl = libraryviewstore::load_query(&state.read().await.pool, &q);
    println!("Loading new playlist {}", pl.name);
    state.write().await.playlist_tabs.add(pl);
    //my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs);
    Ok(warp::reply())
}

async fn smartplaylist(_: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let spl = smartplaylist_parser::construct_smartplaylists_from_config()
        .into_iter()
        .map(|pl| pl.name)
        .collect::<Vec<String>>();
    Ok(warp::reply::json(&spl))
}

async fn smartplaylist_load(
    index: viola_common::LoadSmartPlaylistJson,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    use crate::smartplaylist_parser::LoadSmartPlaylist;
    let spl = smartplaylist_parser::construct_smartplaylists_from_config();
    let pl = spl.get(index.index);

    if let Some(p) = pl {
        let rp = { p.load(&state.read().await.pool) };
        state.write().await.playlist_tabs.add(rp);
        //my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs);
    }

    Ok(warp::reply())
}

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
*/

async fn current_image(state: WebGuiData) -> Result<impl warp::Reply, warp::Rejection> {
    if let Ok(p) = state
        .read()
        .await
        .playlist_tabs
        .get_current_track()
        .albumpath
        .ok_or(warp::reject::not_found())
    {
        let mut f = std::fs::File::open(p).map_err(|_| warp::reject::not_found())?;
        let mut str = String::new();
        f.read_to_string(&mut str)
            .map_err(|_| warp::reject::not_found())?;
        let res = warp::hyper::Response::builder()
            .status(StatusCode::OK)
            .body(str)
            .map_err(|_| warp::reject::not_found())?;
        Ok(res)
    } else {
        Err(warp::reject::not_found())
    }
}

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

async fn change_playlist_tab(
    index: usize,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    state.read().await.playlist_tabs.set_tab(index);
    //my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    Ok(warp::reply())
}

async fn delete_playlist_tab(
    index: usize,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    println!("deleting {}", &index);
    let statelock = state.read().await;
    statelock.playlist_tabs.delete(&statelock.pool, index);
    //my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs);
    //my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist);
    Ok(warp::reply())
}

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
    rx: bus::BusReader<viola_common::GStreamerMessage>,
) {
    let mut rx = rx;
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
        tokio::time::sleep(Duration::new(10 * 60, 0)).await;
        state.read().await.save();
    }
}

pub async fn run(pool: DBPool) {
    println!("Loading playlist");
    let plt = crate::playlist_tabs::load(&pool).expect("Failure to load old playlists");

    println!("Starting gstreamer");
    let mut bus = bus::Bus::new(50);
    let dbus_recv = bus.add_rx();
    let websocket_recv = bus.add_rx();
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
        tokio::spawn(async move { handle_gstreamer_messages(datac, websocket_recv) });
    }
    {
        let datac = data.clone();
        tokio::spawn(async move { auto_save(datac) });
    }
    {
        let datac = data.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::new(1, 0)).await;
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

    //let web_gui_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/");
    let web_gui_dist_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/dist/");

    let data = warp::any().map(move || Arc::clone(&data));

    let gets = {
        let pl = warp::path!("/playlist/")
            .and(data.clone())
            .and_then(playlist);
        let pl_for = warp::path!("/playlist/" / usize)
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
        let cover = warp::path!("/currentimage/")
            .and(data.clone())
            .and_then(current_image);
        let smartpl = warp::path!("/smartplaylist/")
            .and(data.clone())
            .and_then(smartplaylist);
        warp::get().and(
            pl.or(pl_for)
                .or(tr)
                .or(curid)
                .or(pltab)
                .or(cover)
                .or(smartpl),
        )
    };

    let posts = {
        let rep = warp::path!("/repeat/").and(data.clone()).and_then(repeat);
        let clean = warp::path!("/clean/").and(data.clone()).and_then(clean);
        let save = warp::path!("/save/").and(data.clone()).and_then(save);
        let transp = warp::path!("/transport/")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(transport);
        let playlist_tab = warp::path!("/playlisttab/")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(change_playlist_tab);
        let sm_load = warp::path!("/smartplaylist/load/")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(smartplaylist_load);
        let lib_load = warp::path!("/libraryview/full/")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(library_load);

        let lib_part = warp::path!("/libraryview/partial/")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(library_partial_tree);
        warp::post().and(
            rep.or(clean)
                .or(save)
                .or(transp)
                .or(playlist_tab)
                .or(sm_load)
                .or(lib_load)
                .or(lib_part),
        )
    };

    let deletes = {
        let deletepl = warp::path!("/deletefromplaylist/")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(delete_from_playlist);
        let deletetab = warp::path!("/playlisttab/" / usize)
            .and(data.clone())
            .and_then(delete_playlist_tab);
        warp::delete().and(deletepl.or(deletetab))
    };

    let websocket = warp::path("/ws/").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|websocket| {
            let (tx, rx) = websocket.split();
            my_websocket::handle_websocket(tx).await
        })
    });
    let static_files = warp::path("/static/").and(warp::fs::dir("/static/"));
    let index = warp::path("/").and(warp::fs::file("index.html"));

    let all = gets.or(posts).or(deletes).or(static_files).or(index);
    warp::serve(all).run(([127, 0, 0, 1], 8088)).await;
}
