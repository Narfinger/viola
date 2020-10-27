use seed::{prelude::*, *};
use serde;

struct Model {
    tracks: Vec<Track>,
    playlist_tabs: Vec<PlaylistTab>,
    current_playlist_tab: usize,
    play_status: GStreamerAction,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Track {
    id: i32,
    title: String,
    artist: String,
    album: String,
    genre: String,
    tracknumber: Option<i32>,
    year: Option<i32>,
    path: String,
    length: i32,
    albumpath: Option<String>,
    playcount: Option<i32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PlaylistTab {
    tracks: Vec<Track>,
    name: String,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::InitPlaylist);
    orders.send_msg(Msg::InitPlaylistTabs);
    Model {
        tracks: vec![],
        playlist_tabs: vec![],
        current_playlist_tab: 0,
        play_status: GStreamerAction::Stop,
    }
}
enum Msg {
    InitPlaylist,
    InitPlaylistRecv(Vec<Track>),
    InitPlaylistTabs,
    InitPlaylistTabRecv((usize, Vec<PlaylistTab>)),
    PlaylistTabChange(usize),
    Transport(GStreamerAction),
    RefreshPlayStatus,
    RefreshPlayStatusRecv(GStreamerAction),
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum GStreamerAction {
    Next,
    Playing,
    Pausing,
    Previous,
    Stop,
    // This means we selected one specific track
    //Play(usize),
    //Seek(u64),
    //RepeatOnce, // Repeat the current playing track after it finishes
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
        Msg::InitPlaylistRecv(t) => {
            model.tracks = t;
        }

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
                    .json(&ChangePlaylistTabJson { index: index })
                    .expect("Error in setting stuff");
            });
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
                    .json::<GStreamerAction>()
                    .await
                    .expect("Could not parse transport");
                Msg::RefreshPlayStatusRecv(action)
            });
        }
        Msg::RefreshPlayStatusRecv(a) => {
            model.play_status = a;
        }
    }
}

fn view_button(model: &Model) -> Node<Msg> {
    div![
        C!["row"],
        div![
            C!["col-sm"],
            button![
                C!["btn", "btn-primary"],
                "Prev",
                ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Previous))
            ]
        ],
        div![
            C!["col-sm"],
            button![
                C!["btn", "btn-primary"],
                "Play",
                ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Playing))
            ]
        ],
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
    ]
}

fn view_tabs(model: &Model) -> Node<Msg> {
    div![
        div![C!["row"], model.current_playlist_tab],
        div![
            C!["row"],
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
}

fn view_track(t: &Track) -> Node<Msg> {
    tr![
        td![&t.tracknumber],
        td![&t.title],
        td![&t.artist],
        td![&t.album],
        td![&t.genre],
        td![&t.year],
        td![&t.length],
    ]
}

fn view_status(model: &Model) -> Node<Msg> {
    div![C!["row"], div![C!["col-sm"], "NYI"]]
}

fn view(model: &Model) -> Node<Msg> {
    div![div![
        C!["container"],
        view_button(model),
        view_tabs(model),
        div![
            C!["container"],
            style!(St::Overflow => "auto"),
            table![
                C!["table", "table-fixed"],
                tr![
                    th!["TrackNumber"],
                    th!["Title"],
                    th!["Artist"],
                    th!["Album"],
                    th!["Genre"],
                    th!["Year"],
                    th!["Length"],
                ],
                model
                    .playlist_tabs
                    .get(model.current_playlist_tab)
                    .map(|t| &t.tracks)
                    .unwrap_or(&vec![])
                    .iter()
                    .map(view_track)
            ]
        ],
        view_status(model),
    ]]
}

fn main() {
    App::start("app", init, update, view);
    println!("Hello, world!");
}
