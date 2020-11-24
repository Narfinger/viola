extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod websocket;

use seed::{prelude::*, *};
use viola_common::{GStreamerAction, GStreamerMessage, Smartplaylists, Track};

#[derive(Debug)]
struct Model {
    playlist_tabs: Vec<PlaylistTab>,
    playlist_window: PlaylistWindow,
    current_playlist_tab: usize,
    current_time: u64,
    play_status: GStreamerMessage,
    web_socket: WebSocket,
    is_repeat_once: bool,
    sidebar: Sidebar,
    treeviews: Vec<TreeView>,
}

trait ModelImpl {
    fn get_current_playlist_tab_tracks(&self) -> Option<&Vec<Track>>;
}

impl ModelImpl for Model {
    fn get_current_playlist_tab_tracks(&self) -> Option<&Vec<Track>> {
        self.playlist_tabs
            .get(self.current_playlist_tab)
            .map(|tab| &tab.tracks)
    }
}

#[derive(Debug)]
struct Sidebar {
    smartplaylists: Vec<String>,
}

#[derive(Debug)]
struct TreeView {
    tree: indextree::Arena<String>,
    root: indextree::NodeId,
    type_vec: Vec<viola_common::TreeType>,
    current_window: usize,
    stream_handle: Option<StreamHandle>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PlaylistTab {
    tracks: Vec<Track>,
    name: String,
    current_index: usize,
}

/// Struct for having a window into our playlist and slowly fill it
#[derive(Debug, Default)]
struct PlaylistWindow {
    current_window: usize,
    stream_handle: Option<StreamHandle>,
}

const WINDOW_INCREMENT: usize = 50;
const WINDOW_INCREMENT_INTERVALL: u32 = 1000;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::InitPlaylistTabs);
    let sidebar = Sidebar {
        smartplaylists: vec![],
    };
    Model {
        playlist_tabs: vec![],
        playlist_window: PlaylistWindow::default(),
        current_playlist_tab: 0,
        current_time: 0,
        play_status: GStreamerMessage::Nop,
        web_socket: crate::websocket::create_websocket(orders),
        is_repeat_once: false,
        sidebar,
        treeviews: vec![],
    }
}

fn format_time_string(time_in_seconds: u64) -> String {
    let mut res = String::new();
    let seconds: u64 = time_in_seconds % 60;
    let minutes: u64 = (time_in_seconds / 60) % 60;
    let hours: u64 = (time_in_seconds / 60 / 60) % 24;
    let days: u64 = time_in_seconds / 60 / 60 / 24;
    if days != 0 {
        res.push_str(&format!("{} Days, ", days));
    }
    if days != 0 || hours != 0 {
        res.push_str(&format!("{}:", hours));
    }
    if days != 0 || hours != 0 || minutes != 0 {
        res.push_str(&format!("{:02}:", minutes));
    }
    res.push_str(&format!("{:02}", seconds));
    res
}

#[derive(Debug)]
enum Msg {
    InitPlaylistTabs,
    InitPlaylistTabRecv((usize, Vec<PlaylistTab>)),
    PlaylistTabChange(usize),
    /// Increments the playlist window
    PlaylistWindowIncrement,
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
        type_vec: Vec<viola_common::TreeType>,
        search: String,
    },
    CurrentTimeChanged(u64),
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::InitPlaylistTabs => {
            orders.perform_cmd(async {
                #[derive(serde::Deserialize)]
                struct PlaylistTabsJSON {
                    current: usize,
                    current_playing_in: usize,
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
        Msg::PlaylistWindowIncrement => {
            model.playlist_window.current_window += WINDOW_INCREMENT;
            // stop the timer
            if (model.get_current_playlist_tab_tracks().is_some())
                && (model.get_current_playlist_tab_tracks().unwrap().len()
                    <= model.playlist_window.current_window)
            {
                model.playlist_window.stream_handle = None;
            } else if model.playlist_window.stream_handle.is_none() {
                model.playlist_window.stream_handle = Some(orders.stream_with_handle(
                    streams::interval(WINDOW_INCREMENT_INTERVALL, || Msg::PlaylistWindowIncrement),
                ));
            }
        }
        Msg::PlaylistTabChange(index) => {
            model.current_playlist_tab = index;
            orders.perform_cmd(async move {
                #[derive(serde::Serialize)]
                struct ChangePlaylistTabJson {
                    pub index: usize,
                }
                let req = Request::new("/playlisttab/")
                    .method(Method::Post)
                    .json(&ChangePlaylistTabJson { index })
                    .expect("Error in setting stuff");
                fetch(req).await.expect("Could not send message");
            });
            model.playlist_window.current_window = WINDOW_INCREMENT;
            orders.send_msg(Msg::PlaylistWindowIncrement);
        }
        Msg::Transport(t) => {
            orders.perform_cmd(async move {
                let req = Request::new("/transport/")
                    .method(Method::Post)
                    .json(&t)
                    .expect("Could not build result");
                fetch(req).await.expect("Could not send message");
                Msg::RefreshPlayStatus
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
            orders.perform_cmd(async {
                let req = Request::new("/clean/").method(Method::Post);
                fetch(req).await.expect("Could not send request");
                Msg::InitPlaylistTabs
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
                };
                model.treeviews.push(view);
            }
            orders.perform_cmd(async move {
                let data = viola_common::TreeViewQuery {
                    types: type_vec,
                    indices: tree_index.clone(),
                    search: None,
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
                seed::log(tree_index);
                seed::log!(nodeid);
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
            type_vec,
            search,
        } => {
            orders.perform_cmd(async move {
                let data = viola_common::TreeViewQuery {
                    types: type_vec,
                    indices: tree_index,
                    search: None,
                };
                let req = Request::new("/libraryview/full/")
                    .method(Method::Post)
                    .json(&data)
                    .expect("Could not construct query");
                let result = fetch(req)
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
    }
}

fn view_buttons(model: &Model) -> Node<Msg> {
    let play_button: seed::virtual_dom::node::Node<Msg> =
        if model.play_status == GStreamerMessage::Playing {
            button![
                C!["btn", "btn-success"],
                "Pause",
                ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Pausing))
            ]
        } else {
            button![
                C!["btn", "btn-success"],
                "Play",
                ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Playing))
            ]
        };
    div![
        C!["container"],
        div![
            C!["row"],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-info"],
                    attrs!(At::from("data-toggle") => "collapse", At::from("data-target") => "#sidebar", At::from("aria-expanded") => "false", At::from("aria-controls") => "sidebar"),
                    "Menu"
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-primary"],
                    "Prev",
                    ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Previous))
                ]
            ],
            div![C!["col-sm"], play_button],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-primary"],
                    "Pause",
                    ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Pausing))
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-primary"],
                    "Next",
                    ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Next))
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-secondary"],
                    "Again",
                    ev(Ev::Click, |_| Msg::Transport(GStreamerAction::RepeatOnce))
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-danger"],
                    "Clean",
                    ev(Ev::Click, |_| Msg::Clean)
                ]
            ],
        ]
    ]
}

fn view_tabs(model: &Model) -> Node<Msg> {
    div![
        C!["container"],
        div![
            C!["row"],
            div![
                C!["col"],
                ul![
                    C!["nav", "nav-tabs"],
                    model.playlist_tabs.iter().enumerate().map(|(pos, tab)| {
                        if pos == model.current_playlist_tab {
                            li![
                                C!["nav-item"],
                                a![
                                    attrs! {At::Href => "#"},
                                    C!["nav-link", "active"],
                                    &tab.name,
                                    ev(Ev::Click, move |_| Msg::PlaylistTabChange(pos))
                                ]
                            ]
                        } else {
                            li![
                                C!["nav-item"],
                                a![
                                    C!["nav-link"],
                                    &tab.name,
                                    ev(Ev::Click, move |_| Msg::PlaylistTabChange(pos))
                                ]
                            ]
                        }
                    })
                ]
            ]
        ]
    ]
}

fn view_track(t: &Track, is_selected: bool, pos: usize) -> Node<Msg> {
    let length = format!("{}:{:02}", t.length / 60, t.length % 60);
    tr![
        IF!(is_selected => style!(St::Color => "Red")),
        td![
            &t.tracknumber,
            ev(Ev::DblClick, move |_| Msg::Transport(
                GStreamerAction::Play(pos)
            ))
        ],
        td![&t.title,],
        td![&t.artist,],
        td![&t.album,],
        td![&t.genre,],
        td![&t.year,],
        td![&length,],
        ev(Ev::DblClick, move |_| Msg::Transport(
            GStreamerAction::Play(pos)
        ))
    ]
}

fn view_status(model: &Model) -> Node<Msg> {
    let track_option = model
        .playlist_tabs
        .get(model.current_playlist_tab)
        .and_then(|tab| tab.tracks.get(tab.current_index));
    let mut track_status_string = if let Some(track) = track_option {
        format!("{} - {} - {}", track.title, track.artist, track.album)
    } else {
        "Nothing Playing".to_string()
    };
    if !(model.play_status == GStreamerMessage::Playing
        || model.play_status == GStreamerMessage::Pausing)
    {
        track_status_string = "Nothing playing".to_string();
    }

    let total_time: u64 = model
        .get_current_playlist_tab_tracks()
        .unwrap_or(&vec![])
        .iter()
        .map(|track| track.length as u64)
        .sum();
    let total_time_string = format_time_string(total_time);

    let tracks_number_option = model
        .get_current_playlist_tab_tracks()
        .map(|tracks| tracks.len());
    let window_number_option = model.playlist_window.current_window;
    let window_string =
        if tracks_number_option.is_some() && tracks_number_option.unwrap() > window_number_option {
            format!(
                "Pl: {} ({})",
                tracks_number_option.map_or("".to_string(), |t| t.to_string()),
                window_number_option
            )
        } else {
            format!(
                "Pl: {}",
                tracks_number_option.map_or("".to_string(), |t| t.to_string())
            )
        };

    div![
        C!["row", "border", "border-dark"],
        style!(St::Padding => unit!(0.1,em)),
        div![
            C!["col-md"],
            img![
                attrs!(At::Src => "/currentimage/", At::Width => unit!(100,px), At::Height => unit!(100,px))
            ]
        ],
        div![C!["col"], window_string],
        div![C!["col"], format!("Status: {}", model.play_status)],
        div![C!["col"], track_status_string],
        div![C!["col"], "Total Time: ", total_time_string],
        div![C!["col"], IF!(model.is_repeat_once => "Repeat")],
        div![
            C!["col"],
            "Time: ",
            format_time_string(model.current_time),
            "--",
            format_time_string(track_option.map(|t| t.length as u64).unwrap_or(0))
        ]
    ]
}

/// puts true where the track is selected and otherwise false
fn tuple_to_selected_true<'a>(
    model: &'a Model,
    id: usize,
    track: &'a Track,
) -> (&'a Track, bool, usize) {
    (
        track,
        if model.play_status == GStreamerMessage::Playing
            || model.play_status == GStreamerMessage::Pausing
        {
            model
                .playlist_tabs
                .get(model.current_playlist_tab)
                .map(|tab| tab.current_index == id)
                .unwrap_or(false)
        } else {
            false
        },
        id,
    )
}

fn view_smartplaylists(model: &Model) -> Node<Msg> {
    div![
        C!["modal", "fade"],
        attrs![At::from("aria-hidden") => "true", At::Id => "sm_modal"],
        div![
            C!["modal-dialog"],
            div![
                C!["modal-content"],
                div![
                    C!["modal-body"],
                    ul![model
                        .sidebar
                        .smartplaylists
                        .iter()
                        .enumerate()
                        .map(|(i, smp)| li![a![
                            attrs!(At::from("data-dismiss") => "modal"),
                            smp,
                            ev(Ev::Click, move |_| Msg::LoadSmartPlaylist(i))
                        ]])]
                ]
            ]
        ]
    ]
}

fn view_tree_lvl1(
    treeview: &TreeView,
    nodeid: indextree::NodeId,
    model_index: usize,
    index: usize,
) -> Node<Msg> {
    let type_vec_clone = treeview.type_vec.clone();
    let type_vec_clone_12 = treeview.type_vec.clone();

    li![div![
        treeview.tree.get(nodeid).unwrap().get(),
        ul![nodeid
            .children(&treeview.tree)
            .enumerate()
            .map(|(index2, el)| {
                let type_vec_clone_2 = treeview.type_vec.clone();
                let type_vec_clone_22 = treeview.type_vec.clone();

                li![
                    span![
                        treeview.tree.get(el).unwrap().get(),
                        ev(Ev::Click, move |_| Msg::FillTreeView {
                            model_index,
                            tree_index: vec![index, index2],
                            type_vec: type_vec_clone_2,
                            search: "".to_string()
                        }),
                        ev(Ev::DblClick, move |_| Msg::LoadFromTreeView {
                            model_index,
                            tree_index: vec![index, index2],
                            type_vec: type_vec_clone_22,
                            search: "".to_string()
                        }),
                    ],
                    ul![el
                        .children(&treeview.tree)
                        .enumerate()
                        .map(|(index3, el2)| {
                            let type_vec_clone_3 = treeview.type_vec.clone();
                            li![span![
                                treeview.tree.get(el2).unwrap().get(),
                                ev(Ev::DblClick, move |_| Msg::LoadFromTreeView {
                                    model_index,
                                    tree_index: vec![index, index2, index3],
                                    type_vec: type_vec_clone_3,
                                    search: "".to_string(),
                                }),
                            ]]
                        })]
                ]
            })],
        ev(Ev::Click, move |_| Msg::FillTreeView {
            model_index,
            tree_index: vec![index],
            type_vec: type_vec_clone,
            search: "".to_string(),
        }),
        ev(Ev::DblClick, move |_| Msg::LoadFromTreeView {
            model_index,
            tree_index: vec![index],
            type_vec: type_vec_clone_12,
            search: "".to_string(),
        })
    ]]
}

fn view_tree(model_index: usize, model: &Model) -> Node<Msg> {
    if let Some(treeview) = model.treeviews.get(model_index) {
        div![
            C!["modal", "fade"],
            attrs![At::from("aria-hidden") => "true", At::Id => "artisttree"],
            div![
                C!["modal-dialog"],
                div![
                    C!["modal-content"],
                    div![
                        C!["modal-body"],
                        ul![treeview
                            .root
                            .children(&treeview.tree)
                            .take(treeview.current_window)
                            .enumerate()
                            .map(|(i, tree)| view_tree_lvl1(treeview, tree, model_index, i)),]
                    ]
                ]
            ]
        ]
    } else {
        div![]
    }
}

/// Makes the sidebar show for SmartPlaylist, Database Access
fn sidebar_navigation(model: &Model) -> Node<Msg> {
    div![
        //sidebar
        C!["col-xs", "collapse"],
        style!(St::Width => unit!(20,%), St::Padding => unit!(10,px)),
        attrs![At::Id => "sidebar"],
        ul![
            C!["navbar-nav"],
            li![
                C!["nav-item"],
                style!(St::Padding => unit!(10,px)),
                button![
                    C!["btn", "btn-primary"],
                    attrs![At::from("data-toggle") => "modal", At::from("data-target") => "#sm_modal", At::from("data-dismiss") => "modal"],
                    "SmartPlaylist",
                    ev(Ev::Click, move |_| Msg::LoadSmartPlaylistList),
                ]
            ],
            li![
                C!["nav-item"],
                style!(St::Padding => unit!(10,px)),
                button![
                    C!["btn", "btn-primary"],
                    attrs![At::from("data-toggle") => "modal", At::from("data-target") => "#artisttree", At::from("data-dismiss") => "modal"],
                    "Artists",
                    ev(Ev::Click, move |_| Msg::FillTreeView {
                        model_index: 0,
                        tree_index: vec![],
                        search: "".to_string(),
                        type_vec: vec![
                            viola_common::TreeType::Artist,
                            viola_common::TreeType::Album,
                            viola_common::TreeType::Track
                        ]
                    }),
                ]
            ],
        ],
    ]
}

/// Main view
fn view(model: &Model) -> Node<Msg> {
    div![
        C!["container"],
        view_smartplaylists(model),
        view_tree(0, model),
        div![
            C!["row"],
            style!(St::Width => unit!(95,%), St::PaddingTop => unit!(0.1,em)),
            sidebar_navigation(model),
            div![
                C!["col"],
                style!(St::Height => unit!(80,vh)),
                view_buttons(model),
                view_tabs(model),
                div![
                    C!["row"],
                    style!(St::Height => unit!(75,vh),  St::OverflowX => "auto"),
                    div![
                        C!["col-xs", "table-responsive"],
                        style!(St::Overflow => "auto"),
                        table![
                            C!["table", "table-sm", "table-bordered"],
                            thead![
                                style!(St::Position => "sticky"),
                                th!["TrackNumber"],
                                th!["Title"],
                                th!["Artist"],
                                th!["Album"],
                                th!["Genre"],
                                th!["Year"],
                                th!["Length"],
                            ],
                            tbody![model
                                .get_current_playlist_tab_tracks()
                                .unwrap_or(&vec![])
                                .iter()
                                .take(model.playlist_window.current_window)
                                .enumerate()
                                .map(|(id, t)| tuple_to_selected_true(model, id, t))
                                .map(|(t, is_selected, pos)| view_track(t, is_selected, pos)),]
                        ]
                    ],
                ],
                view_status(model),
            ]
        ]
    ]
}

fn main() {
    App::start("app", init, update, view);
}
