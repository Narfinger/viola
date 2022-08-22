use std::ops::Range;

use crate::button::*;
use reqwasm::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct DeleteRangeDialogProps {
    pub(crate) visible: bool,
    pub(crate) refresh_callback: Callback<()>,
    pub(crate) toggle_visible_callback: Callback<()>,
    pub(crate) max: usize,
}

fn send_delete_range(max: usize, input: UseStateHandle<String>, refresh_callback: Callback<()>) {
    let split = input.split('-').collect::<Vec<_>>();
    let start: usize = split.first().unwrap().parse().unwrap();
    let end: usize = split.get(1).and_then(|s| s.parse().ok()).unwrap_or(max);
    let range = Range { start, end };
    wasm_bindgen_futures::spawn_local(async move {
        Request::delete("/deletefromplaylist/")
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&range).unwrap())
            .send()
            .await
            .unwrap();
        refresh_callback.emit(());
    });

    /*use_callback(async move {
     */
    /*
    heyo you can clone a hook/hook handler and pass that into a spawn_local from the wasm-bindgen package
    it acts like an async move closure, so you pass in your async function and handler to update state when it's completed
    actually I think it takes in an async move. Anyway - look into spawn_local: https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/fn.spawn_local.html
    spawn_local in wasm_bindgen_futures - Rust
    Runs a Rust `Future` on the current thread.
    It's actually in wasm_bindgen_futures, not wasm_bindgen. My b
    */
}

#[function_component(DeleteRangeDialog)]
pub(crate) fn delete_range_dialog(props: &DeleteRangeDialogProps) -> Html {
    if !props.visible {
        html! { <></> }
    } else {
        let input = use_state(String::default);
        let onchange = {
            let input = input.clone();
            Callback::from(move |e: Event| {
                let target: Option<EventTarget> = e.target();
                let i = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
                if let Some(i) = i {
                    input.set(i.value());
                }
            })
        };
        let toggle_visible_callback = props.toggle_visible_callback.clone();
        let refresh_callback = props.refresh_callback.clone();
        let max = props.max;
        let submit_callback = Callback::from(move |_| {
            send_delete_range(max - 1, input.clone(), refresh_callback.clone())
        });

        html! {
        <div class="modal" tabindex="-1" role="dialog" style="display: block">
            <div class="modal-dialog" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">{"Smart Playlists"}</h5>
                    </div>
                    <div class="modal-body">
                        <div class="input-group mb-3">
                            <input type="text" class="form-control" placeholder="from-to" onchange={onchange}/>
                        </div>
                    </div>
                    <div class="modal-footer">
                    <CallbackButton
                        text="Close"
                        icon="/x-square.svg"
                        btype={ButtonType::Danger}
                        callback={toggle_visible_callback}
                    />
                    <CallbackButton
                        text="Delete"
                        icon="/trash.svg"
                        btype={ButtonType::Primary}
                        callback={submit_callback}
                    />
                    </div>
                </div>
            </div>
        </div>
        }
    }
}
