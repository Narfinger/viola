#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use std::rc::Rc;

use futures::{stream::SplitStream, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use reqwasm::http::Request;
use viola_common::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

mod button;
mod status;
mod tracks;
mod utils;
use button::Buttons;
use status::Status;
use tracks::{TracksComponent, TracksComponentProps};

struct App {
    current_playing: usize,
    current_status: GStreamerMessage,
    current_tracks: Rc<Vec<viola_common::Track>>,
    current_track_time: u64,
    repeat_once: bool,
}

enum AppMessage {
    WsMessage(viola_common::WsMessage),
    RefreshList,
    RefreshListDone(Vec<viola_common::Track>),
    RepeatOnce,
}

impl App {
    fn handle_wsmessage(&mut self, msg: viola_common::WsMessage) -> bool {
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
            WsMessage::ReloadTabs => false,
            WsMessage::ReloadPlaylist => false,
            WsMessage::Ping => false,
            WsMessage::GStreamerMessage(msg) => match msg {
                GStreamerMessage::Pausing
                | GStreamerMessage::Stopped
                | GStreamerMessage::Playing => {
                    self.current_status = msg;
                    true
                }
                GStreamerMessage::IncreasePlayCount(_) => false,
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
                    if let gloo_net::websocket::Message::Text(msg) = msg {
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
        ctx.link().send_message(AppMessage::RefreshList);
        App {
            current_playing: 0,
            current_status: GStreamerMessage::Stopped,
            current_tracks: Rc::new(vec![]),
            current_track_time: 0,
            repeat_once: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::WsMessage(msg) => self.handle_wsmessage(msg),
            AppMessage::RefreshList => {
                ctx.link().send_future(async move {
                    let new_tracks: Vec<viola_common::Track> = Request::get("/playlist/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap_or_default();
                    AppMessage::RefreshListDone(new_tracks)
                });
                false
            }
            AppMessage::RefreshListDone(tracks) => {
                self.current_tracks = Rc::new(tracks);
                true
            }
            AppMessage::RepeatOnce => {
                self.repeat_once = true;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let full_time_playing: u64 = self.current_tracks.iter().map(|t| t.length as u64).sum();
        let remaining_time_playing: u64 = self
            .current_tracks
            .iter()
            .skip(self.current_playing)
            .map(|t| t.length as u64)
            .sum::<u64>()
            - self.current_track_time;
        let repeat_once_callback = ctx.link().callback(|_| AppMessage::RepeatOnce);
        html! {
            <div class="container-fluid" style="padding-left: 5vw; padding-bottom: 1vh; height: 75vh">
                <div class="col" style="height: 80vh">
                <Buttons status={self.current_status} repeat_callback = {repeat_once_callback }/>
                <div class="row" style="height: 75vh; width: 95vw; overflow-x: auto">
                    <TracksComponent tracks={&self.current_tracks} current_playing={self.current_playing} status = {self.current_status} />
                </div>
                <Status current_status = {self.current_status} current_track = {self.current_tracks.get(self.current_playing).cloned()} total_track_time = {full_time_playing} remaining_time_playing = {remaining_time_playing} current_track_time={self.current_track_time} repeat_once = {self.repeat_once} />
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
