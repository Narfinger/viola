use humantime::format_duration;
use std::time::Duration;

use reqwasm::http::Request;
use yew::prelude::*;

pub(crate) enum TracksComponentMsg {
    IncreaseIndex,
    IncreasePlaycount(usize),
    RefreshList,
    RefreshListDone(Vec<viola_common::Track>),
}

pub(crate) struct TracksComponent {
    pub(crate) tracks: Vec<viola_common::Track>,
    pub(crate) index: u32,
}

pub(crate) fn unwrap_or_empty(i: &Option<i32>) -> String {
    if let Some(i) = i {
        i.to_string()
    } else {
        "".to_string()
    }
}

impl Component for TracksComponent {
    type Message = TracksComponentMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(TracksComponentMsg::RefreshList);
        TracksComponent {
            tracks: vec![],
            index: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TracksComponentMsg::RefreshList => {
                ctx.link().send_future(async move {
                    let new_tracks: Vec<viola_common::Track> = Request::get("/playlist/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap_or_default();
                    TracksComponentMsg::RefreshListDone(new_tracks)
                });
                false
            }
            TracksComponentMsg::RefreshListDone(new_tracks) => {
                self.tracks = new_tracks;
                true
            }
            TracksComponentMsg::IncreaseIndex => {
                self.index += 1;
                true
            }
            TracksComponentMsg::IncreasePlaycount(i) => {
                if let Some(ref mut t) = self.tracks.get_mut(i) {
                    t.playcount = Some(t.playcount.unwrap_or(0) + 1);
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let table_rows = self
            .tracks
            .iter()
            .enumerate()
            .map(|(index, track)| {
                html! {
                    <tr>
                        <td>{index}</td>
                        <td>{unwrap_or_empty(&track.tracknumber)}</td>
                        <td>{&track.title}</td>
                        <td>{&track.artist}</td>
                        <td>{&track.album}</td>
                        <td>{&track.genre}</td>
                        <td>{unwrap_or_empty(&track.year)}</td>
                        <td>{format_duration(Duration::from_secs(track.length as u64)).to_string().replace(' ', "")}</td>
                        <td>{unwrap_or_empty(&track.playcount)}</td>
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
