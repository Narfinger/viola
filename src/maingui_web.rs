use diesel::Connection;
use futures::StreamExt;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use std::{io::Read, sync::Arc};
use tokio::sync::RwLock;
use viola_common::*;
use warp::hyper::Uri;
use warp::Filter;

use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::GStreamerExt;
use crate::libraryviewstore;
use crate::loaded_playlist::{LoadedPlaylistExt, PlaylistControls, SavePlaylistExt};
use crate::my_websocket;
use crate::playlist_tabs::PlaylistTabsExt;
use crate::smartplaylist_parser;
use crate::types::*;

async fn playlist(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let state = state.read().await;
    let items_json = state.playlist_tabs.items_json();
    Ok(items_json)
}

async fn playlist_for(index: usize, state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let state = state.read().await;
    let items_json = state.playlist_tabs.items_for_json(index);
    Ok(items_json)
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
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.read().await.ws, WsMessage::ReloadPlaylist).await;
    });
    Ok(warp::reply())
}

async fn delete_from_playlist(
    deleterange: std::ops::Range<usize>,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    println!("Doing delete");
    state.read().await.playlist_tabs.delete_range(deleterange);
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.read().await.ws, WsMessage::ReloadPlaylist).await;
    });
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
    //tokio::spawn(async move {
    //    my_websocket::send_my_message(&state.read().await.ws, WsMessage::GStreamerAction(msg))
    //        .await;
    //});
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
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.read().await.ws, WsMessage::ReloadTabs).await;
    });
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
        tokio::spawn(async move {
            my_websocket::send_my_message(&state.read().await.ws, WsMessage::ReloadTabs).await;
        });
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

async fn current_image(
    state: WebGuiData,
    _query: ImageQuery,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("into stuff");
    if let Ok(p) = state
        .read()
        .await
        .playlist_tabs
        .get_current_track()
        .albumpath
        .ok_or_else(warp::reject::not_found)
    {
        let path = std::path::PathBuf::from(p);
        let mut f = std::fs::File::open(&path).map_err(|_| warp::reject::not_found())?;
        let mut v = Vec::new();
        f.read_to_end(&mut v)
            .map_err(|_| warp::reject::not_found())?;
        let content_type = match path.extension().and_then(|s| s.to_str()) {
            Some("jpg") => "image/jpeg",
            Some("png") => "image/png",
            _ => "application/octet-stream",
        };
        let resp = warp::hyper::Response::builder()
            .status(warp::hyper::StatusCode::OK)
            .header(warp::hyper::header::CONTENT_TYPE, content_type)
            .body(v)
            .unwrap();
        Ok(resp)
    } else {
        info!("Nothng playing so we don't have a query");
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
        tabs,
    };
    Ok(warp::reply::json(&resp))
}

async fn change_playlist_tab(
    index: usize,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    state.read().await.playlist_tabs.set_tab(index);
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.read().await.ws, WsMessage::ReloadPlaylist).await;
    });
    Ok(warp::reply())
}

async fn delete_playlist_tab(
    index: usize,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    println!("deleting {}", &index);
    let statelock = state.read().await;
    statelock.playlist_tabs.delete(&statelock.pool, index);
    let state = state.clone();
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.read().await.ws, WsMessage::ReloadTabs).await;
        my_websocket::send_my_message(&state.read().await.ws, WsMessage::ReloadPlaylist).await;
    });
    Ok(warp::reply())
}

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<parking_lot::RwLock<gstreamer_wrapper::GStreamer>>,
    playlist_tabs: PlaylistTabsPtr,
    ws: my_websocket::MyWs,
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

/// blocking function that handles messages on the GStreamer Bus
async fn handle_gstreamer_messages(
    state: WebGuiData,
    rx: tokio::sync::watch::Receiver<viola_common::GStreamerMessage>,
) {
    let mut rx = rx;
    while rx.changed().await.is_ok() {
        let state = state.clone();
        let val = *rx.borrow();
        match val {
            viola_common::GStreamerMessage::Playing => {
                let state = state.clone();
                let pos = state.read().await.playlist_tabs.current_position();
                tokio::spawn(async move {
                    //let state = state.clone();
                    my_websocket::send_my_message(
                        &state.read().await.ws,
                        WsMessage::PlayChanged(pos),
                    )
                    .await;
                });
            }
            _ => (),
        }
        if vec![
            viola_common::GStreamerMessage::Pausing,
            viola_common::GStreamerMessage::Stopped,
        ]
        .contains(&val)
        {
            tokio::spawn(async move {
                //let state = state.clone();
                my_websocket::send_my_message(
                    &state.read().await.ws,
                    WsMessage::GStreamerMessage(val),
                )
                .await;
            });
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
    let (rx, tx) = tokio::sync::watch::channel(GStreamerMessage::Nop);
    //let mut bus = bus::Bus::new(50);
    let dbus_recv = tx.clone();
    let websocket_recv = tx.clone();
    let gst = gstreamer_wrapper::new(plt.clone(), pool.clone(), rx)
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
        ws: Arc::new(RwLock::new(None)),
    };

    println!("Doing data");
    let state = Arc::new(RwLock::new(state));

    {
        let datac = state.clone();
        tokio::spawn(async move { handle_gstreamer_messages(datac, websocket_recv).await });
    }
    {
        let datac = state.clone();
        tokio::spawn(async move { auto_save(datac).await });
    }
    {
        let datac = state.clone();
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
                    my_websocket::send_my_message(
                        &datac.read().await.ws,
                        WsMessage::CurrentTimeChanged(data),
                    )
                    .await;
                }
            }
        });
    }

    println!("Starting web gui on {}", crate::types::URL);

    let statec = state.clone();
    let data = warp::any().map(move || Arc::clone(&state));

    let gets = {
        let pl = warp::path!("playlist").and(data.clone()).and_then(playlist);
        let pl_for = warp::path!("playlist" / usize)
            .and(data.clone())
            .and_then(playlist_for);
        let tr = warp::path!("transport")
            .and(data.clone())
            .and_then(get_transport);
        let curid = warp::path!("currentid")
            .and(data.clone())
            .and_then(current_id);
        let pltab = warp::path!("playlisttab")
            .and(data.clone())
            .and_then(playlist_tab);
        let cover = warp::path("currentimage")
            .and(data.clone())
            .and(warp::query::<ImageQuery>())
            .and_then(current_image);
        let smartpl = warp::path!("smartplaylist")
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
        let rep = warp::path!("repeat").and(data.clone()).and_then(repeat);
        let clean = warp::path!("clean").and(data.clone()).and_then(clean);
        let save = warp::path!("save").and(data.clone()).and_then(save);
        let transp = warp::path!("transport")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(transport);
        let playlist_tab = warp::path!("playlisttab")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(change_playlist_tab);
        let sm_load = warp::path!("smartplaylist" / "load")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(smartplaylist_load);
        let lib_load = warp::path!("libraryview" / "full")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(library_load);

        let lib_part = warp::path!("libraryview" / "partial")
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
        let deletepl = warp::path!("deletefromplaylist")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(delete_from_playlist);
        let deletetab = warp::path!("playlisttab" / usize)
            .and(data.clone())
            .and_then(delete_playlist_tab);
        warp::delete().and(deletepl.or(deletetab))
    };

    let websocket = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let statec = statec.clone();
            ws.on_upgrade(|websocket| async move {
                let (tx, _) = websocket.split();
                statec.write().await.ws = Arc::new(RwLock::new(Some(tx)));
            })
        });
    //let web_gui_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/dist/index.html");
    let web_gui_dist_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/dist/");
    //let static_files = warp::get().and(warp::path("static").and(warp::fs::dir(web_gui_dist_path)));
    //let statics = warp::get()
    //    .and(warp::path("static"))
    //    .and(warp::fs::dir(web_gui_dist_path));
    let index = warp::get().and(warp::fs::dir(web_gui_dist_path));

    let all = gets
        .or(posts)
        .or(deletes)
        //.or(static_files)
        .or(websocket)
        .or(index);
    let s: SocketAddr = crate::types::SOCKETADDR.parse().unwrap();
    warp::serve(all).run(s).await;
}
