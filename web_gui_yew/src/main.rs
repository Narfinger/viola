#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use futures::{stream::SplitStream, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use viola_common::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

mod button;
mod status;
mod tracks;
use button::Buttons;
use status::Status;
use tracks::TracksComponent;

struct App {
    current_playing: usize,
    current_status: GStreamerMessage,
}

enum AppMessages {
    WsMessage(viola_common::WsMessage),
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
    type Message = AppMessages;
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
                            link.send_message(AppMessages::WsMessage(val));
                        } else {
                            log::info!("Some problem with ws message decode");
                        }
                    }
                } else {
                    log::info!("Something went wrong with the websocket");
                }
            }
        });
        App {
            current_playing: 0,
            current_status: GStreamerMessage::Stopped,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessages::WsMessage(msg) => self.handle_wsmessage(msg),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="container-fluid" style="padding-left: 5vw; padding-bottom: 1vh; height: 75vh">
                <div class="col" style="height: 80vh">
                <Buttons status={self.current_status} />
                <div class="row" style="height: 75vh; overflow-x: auto">
                    <TracksComponent />
                </div>
                <Status current_status = {self.current_status} current_track = {None} />
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
