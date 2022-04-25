#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use reqwasm::http::Request;
use viola_common::GStreamerAction;
use yew::prelude::*;

mod tracks;
use button::Button;
use tracks::TracksComponent;
mod button;

#[function_component(Buttons)]
fn buttons() -> Html {
    html! {
        <> </>
    }
}

#[function_component(Status)]
fn status() -> Html {
    html! {
    <div class="col">
        <Button text="Menu" icon="" btype={button::ButtonType::Primary} on_click={None} />
        <Button text="Prev" icon="" btype={button::ButtonType::Primary} on_click={Some(GStreamerAction::Previous)} />
        <Button text="Play" icon="" btype={button::ButtonType::Primary} on_click={Some(GStreamerAction::Playing)} />
        <Button text="Pause" icon="" btype={button::ButtonType::Primary} on_click={Some(GStreamerAction::Pausing)} />
        <Button text="Next" icon="" btype={button::ButtonType::Primary} on_click={Some(GStreamerAction::Next)} />
        <Button text="Again" icon="" btype={button::ButtonType::Primary} on_click={Some(GStreamerAction::RepeatOnce)} />
        <Button text="Clean" icon="" btype={button::ButtonType::Primary} on_click={None} />
        <Button text="Delete Range" icon="" btype={button::ButtonType::Primary} on_click={None} />
    </div>}
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="container-fluid" style="padding-left: 5vw; padding-bottom: 1vh; height: 75vh">
            <div class="col" style="height: 80vh">
            <Buttons />
            <div class="row" style="height: 75vh; overflowx: auto">
                <TracksComponent />
            </div>
            <Status />
            </div>
        </div>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::start_app::<App>();
}
