use gloo_net::http::Request;
use std::rc::Rc;
use viola_common::{GStreamerAction, GStreamerMessage};

use yew::prelude::*;

use crate::utils::{self};

pub(crate) enum TracksComponentMsg {
    Play(MouseEvent, usize),
    Nop,
}

#[derive(Properties, PartialEq)]
pub(crate) struct TracksComponentProps {
    pub(crate) tracks: Vec<Rc<viola_common::Track>>,
    pub(crate) current_playing: usize,
    pub(crate) status: GStreamerMessage,
    pub(crate) not_current_tab: bool,
}

pub(crate) struct TracksComponent {}

fn unwrap_or_empty(i: &Option<i32>) -> String {
    if let Some(i) = i {
        i.to_string()
    } else {
        "".to_string()
    }
}

fn color_match(index: usize, ctx: &Context<TracksComponent>) -> (String, Html) {
    let props = ctx.props();
    if index == props.current_playing
        && props.status == GStreamerMessage::Playing
        && props.not_current_tab
    {
        (
            String::from("table-primary"),
            html! {
                <img src="/play.svg" />
            },
        )
    } else if index == props.current_playing && props.status == GStreamerMessage::Playing {
        (
            String::from("table-success"),
            html! {
            <img src="/play.svg" /> },
        )
    } else if index == props.current_playing && props.status == GStreamerMessage::Pausing {
        (
            String::from(""),
            html! {
            <img src="/pause.svg" /> },
        )
    } else {
        (String::from(""), html! {})
    }
}

impl Component for TracksComponent {
    type Message = TracksComponentMsg;
    type Properties = TracksComponentProps;

    fn create(_ctx: &Context<Self>) -> Self {
        TracksComponent {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TracksComponentMsg::Play(_ev, index) => {
                ctx.link().send_future(async move {
                    Request::post("/transport/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&GStreamerAction::Play(index)).unwrap())
                        .unwrap()
                        .send()
                        .await
                        .unwrap();
                    TracksComponentMsg::Nop
                });
            }
            TracksComponentMsg::Nop => {}
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let table_rows = ctx
            .props()
            .tracks
            .iter()
            .enumerate()
            .map(|(index, track)| {
                let (color, image) = color_match(index, ctx);
                let onclick = ctx
                    .link()
                    .callback(move |ev: MouseEvent| TracksComponentMsg::Play(ev, index));
                html! {
                    <tr class={color} ondblclick={onclick}>
                        <td style="width: 5%" >{image} {index}</td>
                        <td style="width: 2%" >{unwrap_or_empty(&track.tracknumber)}</td>
                        <td style="width: 25%">{&track.title}</td>
                        <td style="width: 20%">{&track.artist}</td>
                        <td style="width: 20%">{&track.album}</td>
                        <td style="width: 15%">{&track.genre}</td>
                        <td style="width: 5%" >{unwrap_or_empty(&track.year)}</td>
                        <td style="width: 5%" >{utils::format_time(track.length as u64)}</td>
                        <td style="width: 3%" >{&track.playcount.unwrap_or(0)}</td>
                    </tr>
                }
            })
            .collect::<Html>();
        html! {
            <div class="col-xs table-responsive" style="overflow: auto">
                <table class="table table-sm table-bordered">
                    <thead style="position: sticky">
                        <th style="width: 5%">{"#"}</th>
                        <th style="width: 2%">{"#T"}</th>
                        <th style="width: 25%">{"Title"}</th>
                        <th style="width: 20%">{"Artist"}</th>
                        <th style="width: 20%">{"Album"}</th>
                        <th style="width: 15%">{"Genre"}</th>
                        <th style="width: 5%">{"Year"}</th>
                        <th style="width: 5%">{"Length"}</th>
                        <th style="width: 3%">{"PlyCnt"}</th>
                    </thead>
                    <tbody>{ table_rows}
                    </tbody>
                </table>
            </div>
        }
    }
}
