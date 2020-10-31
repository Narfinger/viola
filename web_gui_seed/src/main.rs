pub mod websocket;

use seed::{prelude::*, *};
use serde;
use viola_common::{GStreamerAction, GStreamerMessage, Track};

struct Model {
    tracks: Vec<Track>,
    playlist_tabs: Vec<PlaylistTab>,
    current_playlist_tab: usize,
    play_status: GStreamerMessage,
    web_socket: WebSocket,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PlaylistTab {
    tracks: Vec<Track>,
    name: String,
    current_index: usize,
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::InitPlaylist);
    orders.send_msg(Msg::InitPlaylistTabs);
    Model {
        tracks: vec![],
        playlist_tabs: vec![],
        current_playlist_tab: 0,
        play_status: GStreamerMessage::Nop,
        web_socket: crate::websocket::create_websocket(orders),
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
    RefreshPlayStatusRecv(GStreamerMessage),
    PlaylistIndexChange(usize),
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
            if let Some(tab) = model.playlist_tabs.get_mut(model.current_playlist_tab) {
                tab.current_index = index;
            }
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
        C!["row"],
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
    ]
}

fn view_tabs(model: &Model) -> Node<Msg> {
    div![
        div![C!["col-sm"], "Tab: ", model.current_playlist_tab],
        div![
            C!["col-sm"],
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

fn view_track(t: &Track, is_selected: bool) -> Node<Msg> {
    let length = format!("{}:{:02}", t.length / 60, t.length % 60);
    tr![
        IF!(is_selected => style!(St::Color => "Red")),
        td![&t.tracknumber],
        td![&t.title],
        td![&t.artist],
        td![&t.album],
        td![&t.genre],
        td![&t.year],
        td![&length],
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
        format!("Nothing Playing")
    };
    if !(model.play_status == GStreamerMessage::Playing
        || model.play_status == GStreamerMessage::Pausing)
    {
        track_status_string = format!("Nothing playing")
    }

    div![
        C!["row"],
        style!(St::Padding => unit!(10,px)),
        div![C!["col-sm"], format!("Status: {}", model.play_status)],
        div![C!["col-sm"], track_status_string]
    ]
}

/// puts true where the track is selected and otherwise false
fn tuple_to_selected_true<'a>(model: &'a Model, id: usize, track: &'a Track) -> (&'a Track, bool) {
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
    )
}

fn view(model: &Model) -> Node<Msg> {
    div![
        style!(St::Padding => "10px"),
        view_buttons(model),
        view_tabs(model),
        div![
            C!["table-responsive"],
            style!(St::Overflow => "auto", St::Height => unit!(80, %)),
            table![
                C!["table"],
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
                model
                    .playlist_tabs
                    .get(model.current_playlist_tab)
                    .map(|t| &t.tracks)
                    .unwrap_or(&vec![])
                    .iter()
                    .enumerate()
                    .map(|(id, t)| tuple_to_selected_true(model, id, t))
                    .map(|(t, is_selected)| view_track(t, is_selected))
            ]
        ],
        view_status(model),
    ]
}

fn main() {
    App::start("app", init, update, view);
    println!("Hello, world!");
}
