use viola_common::{GStreamerMessage, Track};
use yew::prelude::*;

use crate::utils;
pub(crate) enum StatusMsg {}

#[derive(Properties, PartialEq)]
pub(crate) struct StatusMsgProperties {
    pub(crate) number_of_tracks: usize,
    pub(crate) current_status: GStreamerMessage,
    pub(crate) current_track: Option<Track>,
    pub(crate) current_track_time: u64,
    pub(crate) total_track_time: u64,
    pub(crate) remaining_time_playing: u64,
    pub(crate) repeat_once: bool,
}

pub(crate) struct Status {}

impl Component for Status {
    type Message = StatusMsg;
    type Properties = StatusMsgProperties;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        if let Some(ref track) = ctx.props().current_track {
            let status = ctx.props().current_status.to_string();
            let track_status_string = format!(
                "Playing: {} - {} - {}",
                track.title, track.artist, track.album
            );
            let total_time_string = utils::format_time(ctx.props().total_track_time)
                + &format!(
                    "({})",
                    utils::format_time(ctx.props().remaining_time_playing)
                );
            let repeat_once = if ctx.props().repeat_once {
                "Repeat"
            } else {
                ""
            };
            let time_string = String::from("Time: ")
                + &utils::format_time(ctx.props().current_track_time)
                + "--"
                + &utils::format_time(track.length as u64);
            let track_percentage_width = format!(
                "{}%",
                ((ctx.props().current_track_time as f32 / track.length as f32) * 100.0).round()
            );
            html! {
                <div class="row border border-dark" style="padding: 0.1em">
                    <div class="col-md"><img src="/currentimage?nonce={}" width=100 height=100 /></div>
                    <div class="col">{ctx.props().number_of_tracks}</div>
                    <div class="col">{status}</div>
                    <div class="col">{track_status_string}</div>
                    <div class="col">{total_time_string}</div>
                    <div class="col">{repeat_once}</div>
                    <div class="col">{time_string}</div>
                    <div class="col">
                        <div class="progress">
                            <div class="progress-bar" role="progressbar" style="width:"{track_percentage_width}
                            aria-valuenow={format!("{}", ctx.props().current_track_time)} aria-valuemin="0"
                            aria-valuemax={format!("{}", track.length)} />
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {
            <div class="row border border-dark" style="padding: 0.1em">
                <div classs="col-md"/>
                <div class="col" />
                <div class="col">{"Nothing Playing"}</div>
                <div class="col" />
                <div class="col"/>
                <div class="col"/>
                <div class="col"/>
                <div class="col" />
            </div>}
        }
    }
}
