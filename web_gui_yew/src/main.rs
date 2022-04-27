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
use button::Buttons;
use status::Status;
use tracks::{TracksComponent, TracksComponentProps};

struct App {
    current_playing: usize,
    current_status: GStreamerMessage,
    current_tracks: Rc<Vec<viola_common::Track>>,
}

enum AppMessage {
    WsMessage(viola_common::WsMessage),
    RefreshList,
    RefreshListDone(Vec<viola_common::Track>),
}

impl App {
    fn handle_wsmessage(&mut self, msg: viola_common::WsMessage) -> bool {
        match msg {
            WsMessage::PlayChanged(i) => {
                self.current_playing = i;
                self.current_status = GStreamerMessage::Playing;
                true
            }
            WsMessage::CurrentTimeChanged(_) => false,
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
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="container-fluid" style="padding-left: 5vw; padding-bottom: 1vh; height: 75vh">
                <div class="col" style="height: 80vh">
                <Buttons status={self.current_status} />
                <div class="row" style="height: 75vh; overflow-x: auto">
                    <TracksComponent tracks={&self.current_tracks} current_playing={self.current_playing} status = {self.current_status} />
                </div>
                <Status current_status = {self.current_status} current_track = {self.current_tracks.get(self.current_playing).cloned()} />
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
