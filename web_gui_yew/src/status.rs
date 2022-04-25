use viola_common::Track;
use yew::prelude::*;
pub(crate) enum StatusMsg {}

#[derive(Properties, PartialEq)]
pub(crate) struct StatusMsgProperties {
    pub(crate) current_track: Option<Track>,
}

pub(crate) struct Status {}

impl Component for Status {
    type Message = StatusMsg;
    type Properties = StatusMsgProperties;

    fn create(ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let window_string = "";
        let status = "";
        let track_status_string = "";
        let total_time_string = "";
        let repeat_once = "";
        let time_string = "";
        let current_time = "";
        let track_max = "";
        html! {
            <div class="row border border-dark" style="padding: 0.1em">
                <div class="col-md"><img src="/currentimage?nonce={}" /></div>
                <div class="col">{window_string}</div>
                <div class="col">{status}</div>
                <div class="col">{track_status_string}</div>
                <div class="col">{total_time_string}</div>
                <div class="col">{repeat_once}</div>
                <div class="col">{time_string}</div>
                <div class="col">
                    <div class="progress">
                        <div class="progress-bar" role="progressbar" aria-valuenow={current_time} aria-valuemin="0" aria-valuemax={track_max} />
                    </div>
                </div>
            </div>
        }
    }
}
