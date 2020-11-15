pub mod websocket;

use seed::{prelude::*, *};
use viola_common::{GStreamerAction, GStreamerMessage, GeneralTreeViewJson, Smartplaylists, Track};

#[derive(Debug)]
struct Model {
    playlist_tabs: Vec<PlaylistTab>,
    playlist_window: PlaylistWindow,
    current_playlist_tab: usize,
    play_status: GStreamerMessage,
    web_socket: WebSocket,
    is_repeat_once: bool,
    sidebar: Sidebar,
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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PlaylistTab {
    tracks: Vec<Track>,
    name: String,
    current_index: usize,
}

/// Struct for having a window into our playlist and slowly fill it
#[derive(Debug, Default)]
struct PlaylistWindow {
    current_window: Option<usize>,
    stream_handle: Option<StreamHandle>,
}

const WINDOW_INCREMENT: usize = 50;
const WINDOW_INCREMENT_INTERVALL: u32 = 1000;

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::InitPlaylist);
    orders.send_msg(Msg::InitPlaylistTabs);
    let sidebar = Sidebar {
        smartplaylists: vec![],
    };
    Model {
        playlist_tabs: vec![],
        playlist_window: PlaylistWindow::default(),
        current_playlist_tab: 0,
        play_status: GStreamerMessage::Nop,
        web_socket: crate::websocket::create_websocket(orders),
        is_repeat_once: false,
        sidebar,
    }
}

fn format_time_string(time_in_seconds: i32) -> String {
    let mut res = String::new();
    let seconds = time_in_seconds % 60;
    let minutes = (time_in_seconds / 60) % 60;
    let hours = (time_in_seconds / 60 / 60) % 24;
    let days = time_in_seconds / 60 / 60 / 24;
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
    InitPlaylist,
    InitPlaylistRecv(Vec<Track>),
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
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::InitPlaylist => {
            orders.perform_cmd(async {
                let response = fetch("/playlist/").await.expect("HTTP request failed");
                let tracks = response
                    .check_status() // ensure we've got 2xx status
                    .expect("status check failed")
                    .json::<Vec<Track>>()
                    .await
                    .expect("deserialization failed");
                Msg::InitPlaylistRecv(tracks)
            });
        }
        Msg::InitPlaylistRecv(t) => {}

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
        }
        Msg::InitPlaylistTabRecv((current, tabs)) => {
            model.playlist_tabs = tabs;
            model.current_playlist_tab = current;
            orders.send_msg(Msg::PlaylistWindowIncrement);
        }
        Msg::PlaylistWindowIncrement => {
            model.playlist_window.current_window = Some(
                model
                    .playlist_window
                    .current_window
                    .unwrap_or(WINDOW_INCREMENT)
                    + WINDOW_INCREMENT,
            );
            // stop the timer
            if (model.get_current_playlist_tab_tracks().is_some())
                && (model.playlist_window.current_window.is_some())
                && (model.get_current_playlist_tab_tracks().unwrap().len()
                    <= model.playlist_window.current_window.unwrap())
            {
                model.playlist_window.stream_handle = None;
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
            model.playlist_window.stream_handle = Some(orders.stream_with_handle(
                streams::interval(WINDOW_INCREMENT_INTERVALL, || Msg::PlaylistWindowIncrement),
            ));
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
        }
        Msg::PlaylistIndexChange(index) => {
            model.is_repeat_once = false;
            model.play_status = GStreamerMessage::Playing;
            if let Some(tab) = model.playlist_tabs.get_mut(model.current_playlist_tab) {
                tab.current_index = index;
            }
        }
        Msg::Clean => {
            model.is_repeat_once = true;
            orders.perform_cmd(async {
                let req = Request::new("/clean/").method(Method::Post);
                fetch(req).await.expect("Could not send request");
                Msg::InitPlaylist
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
    }
}

fn view_buttons(model: &Model) -> Node<Msg> {
    let play_button: seed::virtual_dom::node::Node<Msg> =
        if model.play_status == GStreamerMessage::Playing {
            button![
                C!["btn", "btn-primary"],
                "Pause",
                ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Pausing))
            ]
        } else {
            button![
                C!["btn", "btn-primary"],
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

    let total_time: i32 = model
        .get_current_playlist_tab_tracks()
        .unwrap_or(&vec![])
        .iter()
        .map(|track| track.length)
        .sum();
    let total_time_string = format_time_string(total_time);

    let tracks_number_option = model
        .get_current_playlist_tab_tracks()
        .map(|tracks| tracks.len());
    let window_number_option = model.playlist_window.current_window;
    let window_string = if tracks_number_option.is_some()
        && window_number_option.is_some()
        && tracks_number_option.unwrap() > window_number_option.unwrap()
    {
        format!(
            "Pl: {} ({})",
            tracks_number_option.map_or("".to_string(), |t| t.to_string()),
            window_number_option.map_or("".to_string(), |t| t.to_string())
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
        div![C!["col"], IF!(model.is_repeat_once => "Repeat")]
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
                button![
                    C!["btn", "btn-primary"],
                    attrs![At::from("data-toggle") => "modal", At::from("data-target") => "#sm_modal", At::from("data-dismiss") => "modal"],
                    "SmartPlaylist",
                    ev(Ev::Click, move |_| Msg::LoadSmartPlaylistList),
                ]
            ],
            li![C!["navbar-nav"], "Test2"],
        ],
    ]
}

/// Main view
fn view(model: &Model) -> Node<Msg> {
    div![
        C!["container"],
        view_smartplaylists(model),
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
                                .take(model.playlist_window.current_window.unwrap_or(100))
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
