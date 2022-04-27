use humantime::format_duration;
use std::{rc::Rc, time::Duration};
use viola_common::GStreamerMessage;

use reqwasm::http::Request;
use yew::prelude::*;

use crate::utils;

pub(crate) enum TracksComponentMsg {
    IncreaseIndex,
    //IncreasePlaycount(usize),
}

#[derive(Properties, PartialEq)]
pub(crate) struct TracksComponentProps {
    pub(crate) tracks: Rc<Vec<viola_common::Track>>,
    pub(crate) current_playing: usize,
    pub(crate) status: GStreamerMessage,
}

pub(crate) struct TracksComponent {}

pub(crate) fn unwrap_or_empty(i: &Option<i32>) -> String {
    if let Some(i) = i {
        i.to_string()
    } else {
        "".to_string()
    }
}

impl Component for TracksComponent {
    type Message = TracksComponentMsg;
    type Properties = TracksComponentProps;

    fn create(ctx: &Context<Self>) -> Self {
        TracksComponent {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let table_rows = ctx
            .props()
            .tracks
            .iter()
            .enumerate()
            .map(|(index, track)| {
                let (color, image) = if index == ctx.props().current_playing
                    && ctx.props().status == GStreamerMessage::Playing
                {
                    ("style: foreground-color: red", "")
                } else {
                    ("", "")
                };
                html! {
                    <tr {color}>
                        <td>{image} {index}</td>
                        <td>{unwrap_or_empty(&track.tracknumber)}</td>
                        <td>{&track.title}</td>
                        <td>{&track.artist}</td>
                        <td>{&track.album}</td>
                        <td>{&track.genre}</td>
                        <td>{unwrap_or_empty(&track.year)}</td>
                        <td>{utils::format_time(track.length as u64)}</td>
                        <td>{&track.playcount.unwrap_or(0)}</td>
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
