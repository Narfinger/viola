extern crate wee_alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod messages;
pub mod models;
pub mod websocket;

use seed::{prelude::*, *};

use messages::*;
use models::*;
use viola_common::{GStreamerAction, GStreamerMessage, Track};

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
        delete_range_input: None,
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

fn icon(path: &str, size: Option<usize>) -> Node<Msg> {
    span![
        style!(St::PaddingRight => unit!(5,px)),
        img![
            attrs!(At::Src => path, At::Height => unit!(size.unwrap_or(24),px), At::Width => unit!(size.unwrap_or(24),px)),
        ],
    ]
}

fn view_buttons(model: &Model) -> Node<Msg> {
    let play_button: seed::virtual_dom::node::Node<Msg> =
        if model.play_status == GStreamerMessage::Playing {
            button![
                C!["btn", "btn-success"],
                icon("/static/pause.svg", Some(22)),
                "Pause",
                ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Pausing))
            ]
        } else {
            button![
                C!["btn", "btn-success"],
                icon("/static/play.svg", None),
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
                    icon("/static/menu-button.svg", Some(20)),
                    "Menu"
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-primary"],
                    icon("/static/skip-backward.svg", None),
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
                    icon("/static/pause.svg", Some(22)),
                    ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Pausing))
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-primary"],
                    icon("/static/skip-forward.svg", None),
                    "Next",
                    ev(Ev::Click, |_| Msg::Transport(GStreamerAction::Next))
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-secondary"],
                    icon("/static/arrow-repeat.svg", Some(20)),
                    "Again",
                    ev(Ev::Click, |_| Msg::Transport(GStreamerAction::RepeatOnce))
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-danger"],
                    icon("/static/trash.svg", Some(18)),
                    "Clean",
                    ev(Ev::Click, |_| Msg::Clean)
                ]
            ],
            div![
                C!["col-sm"],
                button![
                    C!["btn", "btn-danger"],
                    attrs!(At::from("data-toggle") => "modal", At::from("data-target") => "#deleterangemodal"),
                    icon("/static/trash.svg", Some(12)),
                    "Delete Range",
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
                        li![
                            C!["nav-item"],
                            a![
                                attrs! {At::Href => "#"},
                                IF!(pos == model.current_playlist_tab => C!["nav-link", "active"]),
                                IF!(pos != model.current_playlist_tab => C!["nav-link"]),
                                &tab.name,
                                span![
                                    style!(St::PaddingLeft => unit!(5,px)),
                                    img![attrs!(At::Src => "/static/x-square.svg", At::Height => unit!(8,px), At::Width => unit!(8,px)),
                                    ev(Ev::Click, move |_| Msg::PlaylistTabDelete(pos)),
                                    ],
                                ],
                                ev(Ev::Click, move |_| Msg::PlaylistTabChange(pos)),
                            ]
                        ]
                    })
                ]
            ]
        ]
    ]
}

fn view_track(
    playstatus: &GStreamerMessage,
    t: &Track,
    is_selected: bool,
    pos: usize,
) -> Node<Msg> {
    let length = format!("{}:{:02}", t.length / 60, t.length % 60);
    tr![
        IF!(is_selected => style!(St::Color => "Red")),
        td![
            IF!(is_selected && *playstatus==GStreamerMessage::Playing => icon("/static/play.svg", Some(24))),
            IF!(is_selected && *playstatus==GStreamerMessage::Pausing => icon("/static/pause.svg", Some(24))),
            &pos,
            ev(Ev::DblClick, move |_| Msg::Transport(
                GStreamerAction::Play(pos)
            )),
        ],
        td![&t.tracknumber],
        td![&t.title,],
        td![&t.artist,],
        td![&t.album,],
        td![&t.genre,],
        td![&t.year,],
        td![&length,],
        td![&t.playcount.unwrap_or(0)],
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
                attrs!(At::Src => format!("/currentimage/?{}", &track_option.map(|t| &t.title).unwrap_or(&String::from(""))), At::Width => unit!(100,px), At::Height => unit!(100,px))
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
    let type_vec_clone2 = treeview.type_vec.clone();
    li![span![
        treeview.tree.get(nodeid).unwrap().get(),
        button![
            C!["btn", "btn-outline-primary", "btn-sm"],
            style!(St::MarginLeft => unit!(25,px)),
            "Load",
            ev(Ev::Click, move |_| Msg::LoadFromTreeView {
                tree_index: vec![index],
                model_index
            })
        ],
        ul![nodeid
            .children(&treeview.tree)
            .enumerate()
            .map(|(index2, el)| {
                let type_vec_clone_2 = treeview.type_vec.clone();
                li![
                    span![
                        treeview.tree.get(el).unwrap().get(),
                        button![
                            C!["btn", "btn-outline-primary", "btn-sm"],
                            style!(St::MarginLeft => unit!(25,px)),
                            "Load",
                            ev(Ev::Click, move |_| Msg::LoadFromTreeView {
                                tree_index: vec![index, index2],
                                model_index
                            })
                        ],
                        mouse_ev(Ev::Click, move |_| Msg::TreeViewClickAction {
                            model_index,
                            tree_index: vec![index, index2],
                            type_vec: type_vec_clone_2,
                            search: "".to_string(),
                        }),
                    ],
                    ul![el
                        .children(&treeview.tree)
                        .enumerate()
                        .map(|(index3, el2)| {
                            li![span![
                                treeview.tree.get(el2).unwrap().get(),
                                button![
                                    C!["btn", "btn-outline-primary", "btn-sm"],
                                    style!(St::MarginLeft => unit!(25,px)),
                                    "Load",
                                    ev(Ev::Click, move |_| Msg::LoadFromTreeView {
                                        tree_index: vec![index, index2, index3],
                                        model_index
                                    })
                                ],
                            ]]
                        })]
                ]
            })],
        mouse_ev(Ev::Click, move |_| Msg::TreeViewClickAction {
            model_index,
            tree_index: vec![index],
            type_vec: type_vec_clone2,
            search: "".to_string(),
        }),
    ]]
}

fn view_tree(model_index: usize, model: &Model) -> Node<Msg> {
    //if let Some(treeview) = model.treeviews.get(model_index) {
    div![
        C!["modal", "fade"],
        attrs![At::from("aria-hidden") => "true", At::Id => "artisttree"],
        div![
            C!["modal-dialog"],
            div![
                C!["modal-content"],
                div![
                    C!["modal-body"],
                    input![
                        C!["form-control"],
                        attrs!(At::from("placeholder") => "Search"),
                        input_ev(Ev::Input, move |search| Msg::FillTreeView {
                            model_index,
                            tree_index: vec![],
                            type_vec: vec![
                                viola_common::TreeType::Artist,
                                viola_common::TreeType::Album,
                                viola_common::TreeType::Track
                            ],
                            search
                        },)
                    ],
                    if let Some(treeview) = model.treeviews.get(model_index) {
                        ul![treeview
                            .root
                            .children(&treeview.tree)
                            .take(treeview.current_window)
                            .enumerate()
                            .map(|(i, tree)| view_tree_lvl1(treeview, tree, model_index, i)),]
                    } else {
                        li![]
                    }
                ]
            ]
        ]
    ]
    //} else {
    //    div![]
    //}
}

/// Makes the sidebar show for SmartPlaylist, Database Access
fn sidebar_navigation(_model: &Model) -> Node<Msg> {
    div![
        //sidebar
        C!["col-xs", "collapse"],
        style!(St::Width => unit!(20,%), St::Padding => unit!(20,px)),
        attrs![At::Id => "sidebar"],
        ul![
            C!["navbar-nav"],
            li![
                style!(St::Padding => unit!(5, px)),
                C!["nav-item"],
                button![
                    C!["btn", "btn-primary"],
                    attrs![At::from("data-toggle") => "modal", At::from("data-target") => "#sm_modal"],
                    "SmartPlaylist",
                    ev(Ev::Click, move |_| Msg::LoadSmartPlaylistList),
                ]
            ],
            li![
                C!["nav-item"],
                style!(St::Padding => unit!(5, px)),
                button![
                    C!["btn", "btn-primary"],
                    attrs![At::from("data-toggle") => "modal", At::from("data-target") => "#artisttree"],
                    "Artists",
                    //    ev(Ev::Click, move |_| Msg::FillTreeView {
                    //        model_index: 0,
                    //        tree_index: vec![],
                    //        search: "".to_string(),
                    //        type_vec: vec![
                    //            viola_common::TreeType::Artist,
                    //            viola_common::TreeType::Album,
                    //            viola_common::TreeType::Track
                    //        ]
                    //    }),
                ]
            ],
            li![
                C!["nav-item"],
                style!(St::Padding => unit!(5, px)),
                button![
                    C!["btn", "btn-primary"],
                    "Show Full Playlist Window",
                    ev(Ev::Click, |_| Msg::FullPlaylistWindow),
                ],
            ],
        ],
    ]
}

fn view_deleterangemodal(_model: &Model) -> Node<Msg> {
    div![
        C!["modal"],
        attrs!(At::Id => "deleterangemodal", At::from("aria-hidden") => "false", At::from("role") => "dialog"),
        div![
            C!["modal-content"],
            attrs!(At::from("role") => "document"),
            div![C!["modal-header"],],
            div![
                C!["modal-body"],
                form![div![
                    C!["form-group"],
                    label![attrs!(At::from("for") => "rangeinput"), "Range"],
                    input![
                        C!["form-control"],
                        attrs!(At::Id => "rangeinput"),
                        input_ev(Ev::Input, Msg::DeleteRangeInputChanged),
                    ],
                    small![
                        attrs!(At::Id => "rangeinputhelp"),
                        C!["form-text", "text-muted"],
                        "Number-Number or Number- to have it till the ned"
                    ],
                ]],
            ],
            div![
                C!["modal-footer"],
                button![
                    C!["btn" "btn-secondary"],
                    attrs!(At::from("data-dismiss") => "modal", At::from("data-target") => "deleterangemodal"),
                    "Close"
                ],
                button![
                    C!["btn" "btn-secondary"],
                    attrs!(At::from("data-dismiss") => "modal", At::from("data-target") => "deleterangemodal"),
                    "Delete Range",
                    ev(Ev::Click, |_| Msg::DeleteRange),
                ]
            ]
        ]
    ]
}

/// Main view
fn view(model: &Model) -> Node<Msg> {
    div![
        C!["container-fluid"],
        style!(St::PaddingLeft => unit!(5,vw), St::PaddingBottom => unit!(1,vh), St::Height => unit!(75,vh)),
        view_tree(0, model),
        view_smartplaylists(model),
        div![
            C!["row"],
            style!(St::Width => unit!(95,%), St::PaddingTop => unit!(0.1,em)),
            view_deleterangemodal(model),
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
                                th!["#"],
                                th!["TrackNumber"],
                                th!["Title"],
                                th!["Artist"],
                                th!["Album"],
                                th!["Genre"],
                                th!["Year"],
                                th!["Length"],
                                th!["PlyCnt"],
                            ],
                            tbody![model
                                .get_current_playlist_tab_tracks()
                                .unwrap_or(&vec![])
                                .iter()
                                .take(model.playlist_window.current_window)
                                .enumerate()
                                .map(|(id, t)| tuple_to_selected_true(model, id, t))
                                .map(|(t, is_selected, pos)| view_track(
                                    &model.play_status,
                                    t,
                                    is_selected,
                                    pos
                                )),]
                        ]
                    ],
                ],
                view_status(model),
            ]
        ]
    ]
}

fn main() {
    seed::log("We could refactor delete_range and clean to both use just a simple recieved for a playlist with index, the same holds for initplaylisttabs");
    App::start("app", init, update, view);
}
