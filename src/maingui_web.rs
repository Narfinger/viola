use futures::StreamExt;
use log::info;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use std::{io::Read, sync::Arc};
use tokio::sync::RwLock;
use viola_common::*;
use warp::Filter;

use crate::gstreamer_wrapper::{self};
use crate::libraryviewstore;
use crate::loaded_playlist::SavePlaylistExt;
use crate::my_websocket;
use crate::playlist_tabs::{LoadedPlaylistExtImut, PlaylistControlsImut, PlaylistTabsExt};
use crate::smartplaylist_parser;
use crate::types::*;

/// Handler: returns the current playlist tab items in json
async fn playlist(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let items_json = state.playlist_tabs.items_json();
    Ok(items_json)
}

/// Handler: return the items in a playlist by `index` in json
async fn playlist_for(index: usize, state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let items_json = state.playlist_tabs.items_for_json(index);
    Ok(items_json)
}

/// Handler: set that we want to repeat
async fn repeat(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    state
        .gstreamer
        .do_gstreamer_action(viola_common::GStreamerAction::RepeatOnce);
    Ok(warp::reply())
}

/// Handler: removes all already played data
async fn clean(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    info!("doing cleaning");
    state.playlist_tabs.clean();
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist).await;
    });
    Ok(warp::reply())
}

/// Handler: deletes a range from current playlist
async fn delete_from_playlist(
    deleterange: std::ops::Range<usize>,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    info!("Doing delete");
    state.playlist_tabs.delete_range(deleterange);
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.ws, WsMessage::ReloadPlaylist).await;
    });
    Ok(warp::reply())
}

/// Handler: save into database
async fn save(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    println!("Saving");
    let mut db = state.pool.lock();
    state
        .playlist_tabs
        .save(&mut db)
        .expect("Error in saving");
    Ok(warp::reply())
}

/// Handler: returns current transport state
async fn get_transport(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(
        &state.gstreamer.get_state(),
    ))
}

/// Handler: sets current transport state
async fn transport(
    msg: viola_common::GStreamerAction,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    info!("state json data: {:?}", &msg);
    state
        .gstreamer
        .do_gstreamer_action(msg);
    //tokio::spawn(async move {
    //    my_websocket::send_my_message(&state.read().await.ws, WsMessage::GStreamerAction(msg))
    //        .await;
    //});
    Ok(warp::reply())
}

/// Handler: play an artist string (fuzzy matched)
async fn play(artist: String, state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let item_number = {
        let cur = state.playlist_tabs.current_tab();
        state
            .playlist_tabs
            .read()
            .pls
            .get(cur)
            .unwrap()
            .items
            .iter()
            .position(|t| t.artist.contains(&artist))
    };
    if let Some(item_number) = item_number {
        state
            .gstreamer
            .do_gstreamer_action(GStreamerAction::Play(item_number));
    }
    Ok(warp::reply())
}

/// Handler: Returns the partial tree for the query
async fn library_partial_tree(
    query: viola_common::TreeViewQuery,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    let mut q = query;
    if q.search.is_some() && q.search.as_ref().unwrap().is_empty() {
        q.search = None;
    }
    let items = libraryviewstore::partial_query(&state.pool, &q);

    Ok(warp::reply::json(&items))
}

/// Handler: loads the `query` into a new playlist
async fn library_load(
    query: viola_common::TreeViewQuery,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    let mut q = query;
    q.search = q.search.filter(|t| !t.is_empty());
    let pl = libraryviewstore::load_query(&state.pool, &q);
    info!("Loading new playlist {}", pl.name);
    state.playlist_tabs.add(pl);
    let size = state.playlist_tabs.read().pls.len();
    state.playlist_tabs.set_tab(size - 1);
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs).await;
    });
    Ok(warp::reply())
}

/// Handler: returns all smartplaylist names
async fn smartplaylist(_: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let spl = smartplaylist_parser::construct_smartplaylists_from_config()
        .into_iter()
        .map(|pl| pl.name)
        .collect::<Vec<String>>();
    Ok(warp::reply::json(&spl))
}

/// Handler: Loada a smartplaylist into a new tab
async fn smartplaylist_load(
    index: viola_common::LoadSmartPlaylistJson,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    let spl = smartplaylist_parser::construct_smartplaylists_from_config();
    let pl = spl.get(index.index);

    if let Some(p) = pl {
        let rp = { p.load(&state.pool) };
        state.playlist_tabs.add(rp);
        tokio::spawn(async move {
            my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs).await;
        });
    }

    Ok(warp::reply())
}

/// Handler: returns the current playlist position, meaning the track that is playing or would play next
async fn current_id(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(
        &state.playlist_tabs.current_position(),
    ))
}

/// Nice Json for current playing
async fn current_fancy(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let track = state.playlist_tabs.get_current_track();
    Ok(warp::reply::json(&track))
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

/// Handler: returns the current cover album (ignores query because of caching)
async fn current_image(
    state: WebGuiData,
    _query: ImageQuery,
) -> Result<impl warp::Reply, warp::Rejection> {
    info!("into stuff");
    if let Ok(p) = state
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
            Some("webp") => "image/webp",
            _ => "application/octet-stream",
        };
        let resp = warp::hyper::Response::builder()
            .status(warp::hyper::StatusCode::OK)
            .header(warp::hyper::header::CONTENT_TYPE, content_type)
            .header(
                warp::hyper::header::CACHE_CONTROL,
                warp::hyper::header::HeaderValue::from_static("no-cache"),
            )
            .body(v)
            .unwrap();
        Ok(resp)
    } else {
        info!("Nothng playing so we don't have a query");
        Err(warp::reject::not_found())
    }
}

/// Handler: returns all playlist tabs
async fn playlist_tab(state: WebGuiData) -> Result<impl warp::Reply, Infallible> {
    let tabs = state
        .playlist_tabs
        .read()
        .pls
        .iter()
        .map(|pl| PlaylistTabJSON {
            id: pl.id,
            name: pl.name.clone(),
            current_position: pl.current_position,
        })
        .collect::<Vec<PlaylistTabJSON>>();
    // we should not sort this as the ids are different and the vector position should be good
    //tabs.sort_by_key(|a| a.id);

    let current_playing_in = if [GStreamerMessage::Nop, GStreamerMessage::Stopped]
        .contains(&state.gstreamer.get_state())
    {
        None
    } else {
        Some(state.playlist_tabs.current_playing_in())
    };

    let resp = PlaylistTabsJSON {
        current: state.playlist_tabs.current_tab(),
        current_playing_in,
        tabs,
    };
    Ok(warp::reply::json(&resp))
}

/// Handler: sets the current playlist tab to a certain index
async fn change_playlist_tab(
    index: usize,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    state.playlist_tabs.set_tab(index);
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs).await;
    });
    Ok(warp::reply())
}

/// Handler: modifies the current playlist tab, i.e., change name
async fn modify_playlist_tab(
    tab: PlaylistTabJSON,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    {
        let tabs = &state.playlist_tabs;
        let id = tab.id;
        tabs.write()
            .pls
            .iter_mut()
            .find(|t| t.id == id)
            .unwrap()
            .name = tab.name;
    }
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs).await;
    });
    Ok(warp::reply())
}

/// Handler: deletes the playlist tab at the index
async fn delete_playlist_tab(
    index: usize,
    state: WebGuiData,
) -> Result<impl warp::Reply, Infallible> {
    info!("deleting {}", &index);
    state.playlist_tabs.delete(&state.pool, index);
    let state = state.clone();
    tokio::spawn(async move {
        my_websocket::send_my_message(&state.ws, WsMessage::ReloadTabs).await;
    });
    Ok(warp::reply())
}

struct WebGui {
    pool: DBPool,
    gstreamer: Arc<gstreamer_wrapper::GStreamer>,
    playlist_tabs: PlaylistTabsPtr,
    ws: my_websocket::MyWs,
}

impl WebGui {
    fn save(&self) {
        let mut db = self.pool.lock();
        //db.transaction::<_, diesel::result::Error, _>(|_| {
        self.playlist_tabs.save(&mut db).expect("Error in saving");
        //Ok(())
        //})
        //.expect("Error in saving");
    }
}

type WebGuiData = Arc<WebGui>;

/// blocking function that handles messages on the GStreamer Bus
async fn handle_gstreamer_messages(
    state: WebGuiData,
    rx: &mut tokio::sync::broadcast::Receiver<viola_common::GStreamerMessage>,
) {
    while let Ok(val) = rx.recv().await {
        let state = state.clone();
        //let val = *rx.borrow();

        match val {
            viola_common::GStreamerMessage::Playing => {
                let state = state.clone();
                let pos = state.playlist_tabs.current_position();
                tokio::spawn(async move {
                    //let state = state.clone();
                    my_websocket::send_my_message(
                        &state.ws,
                        WsMessage::PlayChanged(pos),
                    )
                    .await;
                });
            }
            GStreamerMessage::Pausing
            | GStreamerMessage::Stopped
            | GStreamerMessage::IncreasePlayCount(_)
            | GStreamerMessage::FileNotFound => {
                tokio::spawn(async move {
                    //let state = state.clone();
                    my_websocket::send_my_message(
                        &state.ws,
                        WsMessage::GStreamerMessage(val),
                    )
                    .await;
                });
            }
            GStreamerMessage::Nop | GStreamerMessage::ChangedDuration(_) => {}
        }
    }
}

/// Timer for auto saving
async fn auto_save(state: WebGuiData) {
    loop {
        tokio::time::sleep(Duration::new(10 * 60, 0)).await;
        state.save();
    }
}

pub async fn run(pool: DBPool) {
    info!("Loading playlist");
    let plt = crate::playlist_tabs::load(&pool).expect("Failure to load old playlists");

    info!("Starting gstreamer");
    let (tx, rx) = tokio::sync::broadcast::channel(10);
    let mut websocket_recv = tx.subscribe();

    let gst = gstreamer_wrapper::new(plt.clone(), pool.clone(), tx)
        .expect("Error Initializing gstreamer");
    {
        info!("Starting dbus");
        let plt = plt.clone();
        let gst = gst.clone();
        tokio::spawn(async move { crate::dbus_interface::main(gst, plt, rx).await });
    }

    info!("Setting up gui");
    let state = WebGui {
        pool: pool.clone(),
        gstreamer: gst,
        playlist_tabs: plt,
        ws: Arc::new(RwLock::new(None)),
    };

    info!("Doing data");
    let state = Arc::new(state);

    {
        let datac = state.clone();
        tokio::spawn(async move { handle_gstreamer_messages(datac, &mut websocket_recv).await });
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
                if datac.gstreamer.get_state()
                    == viola_common::GStreamerMessage::Playing
                {
                    let data = datac
                        .gstreamer
                        .get_elapsed()
                        .unwrap_or(0);
                    my_websocket::send_my_message(
                        &datac.ws,
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
            .and_then(get_transport)
            .with(warp::compression::brotli());
        let curid = warp::path!("currentid")
            .and(data.clone())
            .and_then(current_id)
            .with(warp::compression::brotli());
        let curfancy = warp::path!("currentfancy")
            .and(data.clone())
            .and_then(current_fancy)
            .with(warp::compression::brotli());
        let pltab = warp::path!("playlisttab")
            .and(data.clone())
            .and_then(playlist_tab)
            .with(warp::compression::brotli());
        let cover = warp::path("currentimage")
            .and(data.clone())
            .and(warp::query::<ImageQuery>())
            .and_then(current_image)
            .with(warp::compression::brotli());
        let smartpl = warp::path!("smartplaylist")
            .and(data.clone())
            .and_then(smartplaylist)
            .with(warp::compression::brotli());
        warp::get().and(
            pl.or(pl_for)
                .or(tr)
                .or(curid)
                .or(curfancy)
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
        let play = warp::path!("play")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(play);
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
                .or(play)
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

    let puts = {
        let mod_playlist = warp::path!("playlisttab")
            .and(warp::body::json())
            .and(data.clone())
            .and_then(modify_playlist_tab);
        warp::put().and(mod_playlist)
    };

    let websocket = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let statec = statec.clone();
            ws.on_upgrade(|websocket| async move {
                info!("doing new websocket connection");
                let (tx, _) = websocket.split();
                *statec.ws.write().await = Some(tx);
            })
        });
    //let web_gui_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_seed/dist/index.html");
    let web_gui_dist_path = concat!(env!("CARGO_MANIFEST_DIR"), "/web_gui_yew/dist/");
    //let static_files = warp::get().and(warp::path("static").and(warp::fs::dir(web_gui_dist_path)));
    //let statics = warp::get()
    //    .and(warp::path("static"))
    //    .and(warp::fs::dir(web_gui_dist_path));
    let index = warp::get().and(warp::fs::dir(web_gui_dist_path));

    let all = gets
        .or(posts)
        .or(deletes)
        .or(puts)
        //.or(static_files)
        .or(websocket)
        .or(index);
    let s: SocketAddr = crate::types::SOCKETADDR.parse().unwrap();
    warp::serve(all).run(s).await;
}
