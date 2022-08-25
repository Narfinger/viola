#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use std::{cell::RefCell, rc::Rc};

use futures::StreamExt;
use reqwasm::http::Request;
use reqwasm::websocket;
use reqwasm::websocket::futures::WebSocket;
use viola_common::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

mod button;
mod delete_range_dialog;
mod sidebar;
mod status;
mod tabs;
mod tracks;
mod treeview;
mod utils;
use button::Buttons;
use delete_range_dialog::DeleteRangeDialog;
use sidebar::Sidebar;
use status::Status;
use tabs::TabsComponent;
use tracks::TracksComponent;

const TRACK_MAX_NUMBER: usize = 500;
const RIDICULOUS_LARGE_TRACK_NUMBER: usize = 10000;
const PLAYLIST_SIZE: usize = 500;

struct App {
    current_playing: usize,
    current_status: GStreamerMessage,
    current_tracks: Rc<RefCell<Vec<viola_common::Track>>>,
    current_track_time: u64,
    repeat_once: bool,
    sidebar_visible: bool,
    delete_range_visible: bool,
    playlist_tabs: PlaylistTabsJSON,
    show_full_playlist: bool,
}

enum AppMessage {
    WsMessage(viola_common::WsMessage),
    RefreshPlayStatus,
    RefreshPlayStatusDone((usize, GStreamerMessage)),
    RefreshList,
    RefreshListDone(Vec<viola_common::Track>),
    RepeatOnce,
    LoadTabs,
    LoadTabsDone(PlaylistTabsJSON),
    ReloadTabs,
    ToggleSidebar,
    ToggleDeleteRange,
    ShowFullPlaylist,
}

impl App {
    fn handle_wsmessage(&mut self, ctx: &Context<Self>, msg: viola_common::WsMessage) -> bool {
        match msg {
            WsMessage::PlayChanged(i) => {
                self.current_playing = i;
                self.current_status = GStreamerMessage::Playing;
                self.current_track_time = 0;
                self.repeat_once = false;
                true
            }
            WsMessage::CurrentTimeChanged(i) => {
                self.current_track_time = i;
                true
            }
            WsMessage::ReloadTabs => {
                ctx.link()
                    .send_message_batch(vec![AppMessage::LoadTabs, AppMessage::RefreshList]);
                false
            }
            WsMessage::ReloadPlaylist => {
                ctx.link().send_message(AppMessage::RefreshList);
                false
            }
            WsMessage::Ping => false,
            WsMessage::GStreamerMessage(msg) => match msg {
                GStreamerMessage::Pausing
                | GStreamerMessage::Stopped
                | GStreamerMessage::Playing => {
                    self.current_status = msg;
                    true
                }
                GStreamerMessage::IncreasePlayCount(i) => {
                    if let Some(ref mut t) = self.current_tracks.borrow_mut().get_mut(i) {
                        t.playcount = Some(t.playcount.unwrap_or(0) + 1);
                    }
                    true
                }
                GStreamerMessage::Nop => false,
                GStreamerMessage::ChangedDuration(_) => false,
            },
        }
    }
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        let ws = WebSocket::open("ws://127.0.0.1:8080/ws/").unwrap();

        let (_write, mut read) = ws.split();

        let link = ctx.link().clone();
        spawn_local(async move {
            while let Some(msg) = read.next().await {
                if let Ok(msg) = msg {
                    if let websocket::Message::Text(msg) = msg {
                        if let Ok(val) = serde_json::from_str(&msg) {
                            link.send_message(AppMessage::WsMessage(val));
                        } else {
                            log::info!("Some problem with ws message decode");
                        }
                    }
                } else {
                    log::info!("Something went wrong with the websocket");
                }
            }
        });
        let a = App {
            current_playing: 0,
            current_status: GStreamerMessage::Stopped,
            current_tracks: Rc::new(RefCell::new(vec![])),
            current_track_time: 0,
            repeat_once: false,
            sidebar_visible: false,
            delete_range_visible: false,
            playlist_tabs: PlaylistTabsJSON {
                current: 0,
                tabs: vec![],
            },
            show_full_playlist: false,
        };
        ctx.link()
            .send_message_batch(vec![AppMessage::LoadTabs, AppMessage::RefreshList]);
        a
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::WsMessage(msg) => self.handle_wsmessage(ctx, msg),
            AppMessage::RefreshList => {
                let show_full = self.show_full_playlist;
                ctx.link().send_future(async move {
                    let q_string = if show_full {
                        "/playlist/".to_string()
                    } else {
                        format!("/playlist/size={}", PLAYLIST_SIZE)
                    };
                    let new_tracks: Vec<viola_common::Track> = Request::get(&q_string)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap_or_default();
                    AppMessage::RefreshListDone(new_tracks)
                });
                ctx.link().send_message(AppMessage::RefreshPlayStatus);
                false
            }
            AppMessage::RefreshListDone(tracks) => {
                self.current_tracks.replace(tracks);
                true
            }
            AppMessage::RefreshPlayStatus => {
                ctx.link().send_future(async move {
                    let status = Request::get("/transport/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await;
                    let id = Request::get("/currentid/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await;
                    AppMessage::RefreshPlayStatusDone((id.unwrap(), status.unwrap()))
                });
                false
            }
            AppMessage::RefreshPlayStatusDone((id, status)) => {
                self.current_playing = id;
                self.current_status = status;
                true
            }
            AppMessage::RepeatOnce => {
                self.repeat_once = true;
                true
            }
            AppMessage::ToggleSidebar => {
                self.sidebar_visible = !self.sidebar_visible;
                true
            }
            AppMessage::ToggleDeleteRange => {
                self.delete_range_visible = !self.delete_range_visible;
                true
            }
            AppMessage::LoadTabs => {
                ctx.link().send_future(async move {
                    let tabs: PlaylistTabsJSON = Request::get("/playlisttab/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    AppMessage::LoadTabsDone(tabs)
                });
                false
            }
            AppMessage::LoadTabsDone(loaded_tabs) => {
                self.playlist_tabs = loaded_tabs;
                true
            }
            AppMessage::ReloadTabs => {
                self.current_tracks.replace(vec![]);
                ctx.link()
                    .send_message_batch(vec![AppMessage::LoadTabs, AppMessage::RefreshList]);
                true
            }
            AppMessage::ShowFullPlaylist => {
                self.show_full_playlist = true;
                ctx.link().send_message(AppMessage::RefreshList);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let full_time_playing: u64 = self
            .current_tracks
            .borrow()
            .iter()
            .map(|t| t.length as u64)
            .sum();
        let remaining_time_playing: u64 = self
            .current_tracks
            .borrow()
            .iter()
            .skip(self.current_playing)
            .map(|t| t.length as u64)
            .sum::<u64>()
            .wrapping_sub(self.current_track_time);
        html! {
            <div class="container-fluid" style="padding-left: 5vw; padding-bottom: 1vh; height: 75vh">
                    <Sidebar
                        visible = {self.sidebar_visible}
                        close_callback = {ctx.link().callback(|_| AppMessage::ToggleSidebar)}
                        reload_callback = {ctx.link().batch_callback(|_| vec![AppMessage::LoadTabs, AppMessage::RefreshList])}
                        show_all_tracks_callback = {ctx.link().callback(|_| AppMessage::ShowFullPlaylist)}
                        />
                    <DeleteRangeDialog
                        visible = {self.delete_range_visible}
                        refresh_callback = {ctx.link().callback(|_| AppMessage::RefreshList)}
                        toggle_visible_callback = {ctx.link().callback(|_| AppMessage::ToggleDeleteRange)}
                        max = {self.current_tracks.borrow().len()}
                    />
                    <div class="row">
                        <div class="col" style="height: 80vh">
                            <Buttons
                                // the clean tab refresh will happen from the websocket and not here
                                status={self.current_status}
                                repeat_once_callback = {ctx.link().callback(|_| AppMessage::RepeatOnce)}
                                refresh_play_callback = {ctx.link().callback(|_| AppMessage::RefreshPlayStatus)}
                                sidebar_callback = {ctx.link().callback(|_| AppMessage::ToggleSidebar)}
                                delete_range_callback = {ctx.link().callback(|_| AppMessage::ToggleDeleteRange)}
                                />

                            <TabsComponent
                            // the tab refresh and similar thing will come from the websocket as otherwise we would refresh the old status
                                tabs = {self.playlist_tabs.clone()}
                                />

                            <div class="row" style="height: 75vh; width: 95vw; overflow-x: auto">
                                <TracksComponent
                                    tracks={&self.current_tracks}
                                    current_playing={self.current_playing}
                                    max_track_number = {if self.show_full_playlist {
                                        RIDICULOUS_LARGE_TRACK_NUMBER
                                    } else {
                                        TRACK_MAX_NUMBER
                                    }}
                                    status = {self.current_status}
                                    />
                            </div>

                        <Status
                            current_status = {self.current_status}
                            current_track = {self.current_tracks.borrow().get(self.current_playing).cloned()} total_track_time = {full_time_playing} remaining_time_playing = {remaining_time_playing} current_track_time={self.current_track_time} repeat_once = {self.repeat_once} number_of_tracks={self.current_tracks.borrow().len()}
                            window = {TRACK_MAX_NUMBER}
                            />
                        </div>
                    </div>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
