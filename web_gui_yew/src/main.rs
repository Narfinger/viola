#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use futures::StreamExt;
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

#[function_component(App)]
fn app() -> Html {
    let ws = WebSocket::open("ws://127.0.0.1:8080/ws/").unwrap();

    let (mut write, mut read) = ws.split();

    spawn_local(async move {
        while let Some(msg) = read.next().await {
            log::info!("1. {:?}", msg);
        }
    });

    html! {
        <div class="container-fluid" style="padding-left: 5vw; padding-bottom: 1vh; height: 75vh">
            <div class="col" style="height: 80vh">
            <Buttons status={GStreamerMessage::Pausing} />
            <div class="row" style="height: 75vh; overflow-x: auto">
                <TracksComponent />
            </div>
            <Status current_track = {None} />
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
