use crate::button::*;
use reqwasm::http::Request;
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct PlayDialogProps {
    pub(crate) visible: bool,
    pub(crate) toggle_visible_callback: Callback<()>,
}

fn submit(toggle: Callback<()>, input: UseStateHandle<String>) {
    wasm_bindgen_futures::spawn_local(async move {
        Request::post("/play/")
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&*input).unwrap())
            .send()
            .await
            .unwrap();
        toggle.emit(());
    });
}

#[function_component(PlayDialog)]
pub(crate) fn delete_range_dialog(props: &PlayDialogProps) -> Html {
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
    if !props.visible {
        html! { <></> }
    } else {
        let toggle_callback = props.toggle_visible_callback.clone();
        let submit_callback = Callback::from(move |_| {
            submit(toggle_callback.clone(), input.clone());
        });
        html! {
        <div class="modal" tabindex="-1" role="dialog" style="display: block">
            <div class="modal-dialog" role="document">
                <div class="modal-content">
                    <div class="modal-header">
                        <h5 class="modal-title">{"Play"}</h5>
                    </div>
                    <div class="modal-body">
                        <div class="input-group mb-3">
                            <input type="text" class="form-control" placeholder="Artist" onchange={onchange}/>
                        </div>
                    </div>
                    <div class="modal-footer">
                    <CallbackButton
                        text="Play"
                        icon="/play.svg"
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
