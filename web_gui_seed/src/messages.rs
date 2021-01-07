use crate::models::*;
use viola_common::{GStreamerAction, GStreamerMessage, Smartplaylists, Track};

use seed::prelude::*;

const WINDOW_INCREMENT: usize = 100;
const WINDOW_INCREMENT_INTERVALL: u32 = 1000;
const WINDOW_MAX: usize = 500;
#[derive(Debug)]
pub(crate) enum Msg {
    Nop,
    InitPlaylistTabs,
    InitPlaylistTabRecv((usize, Vec<PlaylistTab>)),
    PlaylistTabChange(usize),
    PlaylistTabDelete(usize),
    /// Increments the playlist window
    PlaylistWindowIncrement,
    FullPlaylistWindow,
    Transport(GStreamerAction),
    RefreshPlayStatus,
    RefreshPlayStatusRecv(GStreamerMessage),
    PlaylistIndexChange(usize),
    Clean,
    /// Loads the names of all smart playlists
    LoadSmartPlaylistList,
    LoadSmartPlaylistListRecv(Vec<String>),
    /// Load a smartplaylist
    LoadSmartPlaylist(usize),
    ///we need to check which click it is and then either do the `FillTreeView` or `LoadFromTreeView` action
    TreeViewClickAction {
        model_index: usize,
        tree_index: Vec<usize>,
        type_vec: Vec<viola_common::TreeType>,
        search: String,
    },
    /// Fill the treeview of `model_index`, with at position `tree_index` with `type_vec`
    FillTreeView {
        model_index: usize,
        tree_index: Vec<usize>,
        type_vec: Vec<viola_common::TreeType>,
        search: String,
    },
    FillTreeViewRecv {
        model_index: usize,
        tree_index: Vec<usize>,
        result: Vec<String>,
    },
    TreeWindowIncrement {
        tree_index: usize,
    },
    LoadFromTreeView {
        model_index: usize,
        tree_index: Vec<usize>,
    },
    CurrentTimeChanged(u64),
    DeleteRangeInputChanged(String),
    DeleteRange,
}

pub(crate) fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Nop => {}
        Msg::InitPlaylistTabs => {
            orders.perform_cmd(async {
                #[derive(serde::Deserialize)]
                struct PlaylistTabsJSON {
                    current: usize,
                    tabs: Vec<String>,
                }
                let response = fetch("/playlisttab/").await.expect("HTTP Request failed");
                let playlisttabs: PlaylistTabsJSON = response
                    .check_status()
                    .expect("status check failed")
                    .json()
                    .await
                    .expect("Deserilization failed");
                let mut tabs = Vec::new();
                for (i, val) in playlisttabs.tabs.into_iter().enumerate() {
                    let res = fetch(format! {"/playlist/{}/", i})
                        .await
                        .expect("Error in request");
                    let items: Vec<Track> = res
                        .check_status()
                        .expect("status check failed")
                        .json()
                        .await
                        .expect("Deserilization failed");
                    let new_tab = PlaylistTab {
                        name: val,
                        tracks: items,
                        current_index: 0,
                    };
                    tabs.push(new_tab);
                }
                Msg::InitPlaylistTabRecv((playlisttabs.current, tabs))
                //Msg::InitPlaylistTabRecv((playlisttabs.current, playlisttabs.iter().map(|tab_name| {PlaylistTab {name: tab_name, tracks: vec![]}}.collect()))
            });
            orders.send_msg(Msg::RefreshPlayStatus);
        }
        Msg::InitPlaylistTabRecv((current, tabs)) => {
            model.playlist_tabs = tabs;
            model.current_playlist_tab = current;
            orders.send_msg(Msg::PlaylistWindowIncrement);
        }
        Msg::PlaylistTabDelete(index) => {
            model.playlist_tabs.swap_remove(index);
            orders.perform_cmd(async move {
                let req = Request::new("/playlisttab/")
                    .method(Method::Delete)
                    .json(&index)
                    .expect("Could not build query");
                fetch(req).await.expect("Error in sending request");
                Msg::PlaylistTabChange(0)
            });
        }
        Msg::PlaylistWindowIncrement => {
            model.playlist_window.current_window += WINDOW_INCREMENT;
            // stop the timer
            if (model.get_current_playlist_tab_tracks().is_some())
                && (model.get_current_playlist_tab_tracks().unwrap().len()
                    <= model.playlist_window.current_window
                    || model.playlist_window.current_window >= WINDOW_MAX)
            {
                model.playlist_window.stream_handle = None;
            } else if model.playlist_window.stream_handle.is_none() {
                model.playlist_window.stream_handle = Some(orders.stream_with_handle(
                    streams::interval(WINDOW_INCREMENT_INTERVALL, || Msg::PlaylistWindowIncrement),
                ));
            }
        }
        Msg::FullPlaylistWindow => {
            model.playlist_window.current_window =
                model.get_current_playlist_tab_tracks().unwrap().len();
        }
        Msg::PlaylistTabChange(index) => {
            model.current_playlist_tab = index;
            orders.perform_cmd(async move {
                let req = Request::new("/playlisttab/")
                    .method(Method::Post)
                    .json(&index)
                    .expect("Error in setting stuff");
                fetch(req).await.expect("Could not send message");
            });
            model.playlist_window.current_window = WINDOW_INCREMENT;
            orders.send_msg(Msg::PlaylistWindowIncrement);
        }
        Msg::Transport(t) => {
            if t == GStreamerAction::RepeatOnce {
                model.is_repeat_once = true;
            }
            orders.perform_cmd(async move {
                let req = Request::new("/transport/")
                    .method(Method::Post)
                    .json(&t)
                    .expect("Could not build result");
                fetch(req).await.expect("Could not send message");
                if t != GStreamerAction::RepeatOnce {
                    Msg::RefreshPlayStatus
                } else {
                    Msg::Nop
                }
            });
        }
        Msg::RefreshPlayStatus => {
            orders.perform_cmd(async {
                let req = fetch("/transport/").await.expect("Could not send req");
                let action = req
                    .json::<GStreamerMessage>()
                    .await
                    .expect("Could not parse transport");
                Msg::RefreshPlayStatusRecv(action)
            });
        }
        Msg::RefreshPlayStatusRecv(a) => {
            model.play_status = a;
            if model.play_status == GStreamerMessage::Playing {
                orders.perform_cmd(async {
                    let result = fetch("/currentid/")
                        .await
                        .expect("Could not send req")
                        .json::<usize>()
                        .await
                        .expect("Could not parse message");
                    Msg::PlaylistIndexChange(result)
                });
            }
        }
        Msg::PlaylistIndexChange(index) => {
            model.is_repeat_once = false;
            model.play_status = GStreamerMessage::Playing;
            if let Some(tab) = model.playlist_tabs.get_mut(model.current_playlist_tab) {
                tab.current_index = index;
            }
        }
        Msg::Clean => {
            let index = model
                .get_current_playlist_tab()
                .map(|tab| tab.current_index)
                .unwrap();
            model.get_current_playlist_tab_mut().unwrap().current_index = 0;
            let mut_tracks = model.get_current_playlist_tab_tracks_mut().unwrap();
            *mut_tracks = mut_tracks.split_off(index);
            orders.perform_cmd(async {
                let req = Request::new("/clean/").method(Method::Post);
                fetch(req).await.expect("Could not send request");
            });
        }
        Msg::LoadSmartPlaylistList => {
            orders.perform_cmd(async {
                let fill = fetch("/smartplaylist/")
                    .await
                    .expect("Error in request")
                    .json::<Smartplaylists>()
                    .await
                    .expect("Could not fetch smartplaylists");
                Msg::LoadSmartPlaylistListRecv(fill)
            });
        }
        Msg::LoadSmartPlaylistListRecv(list) => {
            model.sidebar.smartplaylists = list;
        }
        Msg::LoadSmartPlaylist(index) => {
            orders.perform_cmd(async move {
                let data = viola_common::LoadSmartPlaylistJson { index };
                let req = Request::new("/smartplaylist/load/")
                    .method(Method::Post)
                    .json(&data)
                    .expect("could not construct query");
                fetch(req).await.expect("Could not send request");
                Msg::InitPlaylistTabs
            });
        }
        Msg::TreeViewClickAction {
            model_index,
            tree_index,
            type_vec,
            search,
        } => {
            orders.send_msg(Msg::FillTreeView {
                model_index,
                tree_index,
                type_vec,
                search,
            });
        }
        Msg::FillTreeView {
            model_index,
            tree_index,
            type_vec,
            search,
        } => {
            if model.treeviews.get(model_index).is_none() {
                let mut arena = indextree::Arena::new();
                let root = arena.new_node("".to_string());
                let view = TreeView {
                    tree: arena,
                    root,
                    type_vec: type_vec.clone(),
                    current_window: 2,
                    stream_handle: None,
                    search: search.clone(),
                };
                model.treeviews.push(view);
            } else {
                model
                    .treeviews
                    .get_mut(model_index)
                    .map(|s| s.search = search.clone());
            }
            orders.perform_cmd(async move {
                let data = viola_common::TreeViewQuery {
                    types: type_vec,
                    indices: tree_index.clone(),
                    search: Some(search),
                };
                let req = Request::new("/libraryview/partial/")
                    .method(Method::Post)
                    .json(&data)
                    .expect("Could not construct query");
                let result = fetch(req)
                    .await
                    .expect("Could not send request")
                    .json::<Vec<String>>()
                    .await
                    .expect("Could not fetch treeview");
                Msg::FillTreeViewRecv {
                    model_index,
                    tree_index,
                    result,
                }
            });
        }
        Msg::FillTreeViewRecv {
            model_index,
            tree_index,
            result,
        } => {
            if let Some(treeview) = model.treeviews.get_mut(model_index) {
                let nodeid = &match tree_index.len() {
                    0 => {
                        // this means we are the second message, hence we need to clear our arena (and make a new root node)
                        let mut arena = indextree::Arena::new();
                        let root = arena.new_node("".to_string());
                        treeview.tree = arena;
                        treeview.root = root;
                        Some(treeview.root)
                    }
                    1 => treeview.root.children(&treeview.tree).nth(tree_index[0]),
                    2 => treeview
                        .root
                        .children(&treeview.tree)
                        .nth(tree_index[0])
                        .map(|t| t.children(&treeview.tree))
                        .and_then(|mut t| t.nth(tree_index[1])),
                    _ => None,
                };
                if let Some(nodeid) = nodeid {
                    if nodeid.children(&treeview.tree).next().is_none() {
                        for i in result.into_iter() {
                            let new_node = treeview.tree.new_node(i);
                            nodeid.append(new_node, &mut treeview.tree);
                        }
                    }

                    treeview.stream_handle = Some(orders.stream_with_handle(streams::interval(
                        WINDOW_INCREMENT_INTERVALL,
                        move || Msg::TreeWindowIncrement {
                            tree_index: model_index,
                        },
                    )));
                }
            }
        }
        Msg::TreeWindowIncrement { tree_index } => {
            let mut tree = model.treeviews.get_mut(tree_index).unwrap();
            tree.current_window += WINDOW_INCREMENT;
            if tree.current_window >= tree.tree.count() {
                tree.stream_handle = None
            };
        }
        Msg::LoadFromTreeView {
            model_index,
            tree_index,
        } => {
            let search = model.treeviews.get(model_index).map(|t| t.search.clone());
            let type_vec = model.treeviews.get(model_index).unwrap().type_vec.clone();
            orders.perform_cmd(async move {
                let data = viola_common::TreeViewQuery {
                    types: type_vec,
                    indices: tree_index,
                    search,
                };
                let req = Request::new("/libraryview/full/")
                    .method(Method::Post)
                    .json(&data)
                    .expect("Could not construct query");
                fetch(req)
                    .await
                    .expect("Could not send request")
                    .json::<Vec<String>>()
                    .await
                    .expect("Could not fetch treeview");
                Msg::InitPlaylistTabs
            });
        }
        Msg::CurrentTimeChanged(time) => {
            model.current_time = time;
        }
        Msg::DeleteRangeInputChanged(text) => {
            model.delete_range_input = Some(text);
        }
        Msg::DeleteRange => {
            let range = model.delete_range_input.as_ref().unwrap();
            let size = model.get_current_playlist_tab_tracks().unwrap().len();
            let strings: Vec<&str> = range.split('-').collect();
            let start: usize = std::str::FromStr::from_str(strings.get(0).unwrap()).unwrap();
            let end: usize = strings
                .get(1)
                .and_then(|t| std::str::FromStr::from_str(t).ok())
                .unwrap_or(size - 1);
            let range = std::ops::Range { start, end };
            //let rangec = range.clone();
            //remove in our model
            let new_playlist = model
                .get_current_playlist_tab_tracks()
                .unwrap()
                .iter()
                .enumerate()
                .skip_while(|(index, _)| start <= *index && *index <= end)
                .map(|(_, val)| val)
                .cloned()
                .collect();
            model
                .playlist_tabs
                .get_mut(model.current_playlist_tab)
                .unwrap()
                .tracks = new_playlist;

            orders.perform_cmd(async move {
                let req = Request::new("/deletefromplaylist/")
                    .method(Method::Delete)
                    .json(&range)
                    .expect("Could not construct request");
                fetch(req).await.expect("Could not send request");
                Msg::RefreshPlayStatus
            });
            orders.send_msg(Msg::PlaylistTabChange(model.current_playlist_tab));
        }
    }
}
