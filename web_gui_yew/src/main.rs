use reqwasm::http::Request;
use yew::prelude::*;

enum TracksComponentMsg {
    IncreaseIndex,
    IncreasePlaycount(usize),
    RefreshList,
    RefreshListDone(Vec<viola_common::Track>),
}

struct TracksComponent {
    tracks: Vec<viola_common::Track>,
    index: u32,
}

fn unwrap_or_empty(i: &Option<i32>) -> String {
    if let Some(i) = i {
        i.to_string()
    } else {
        "".to_string()
    }
}

impl Component for TracksComponent {
    type Message = TracksComponentMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        TracksComponent {
            tracks: vec![],
            index: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TracksComponentMsg::RefreshList => {
                ctx.link().send_future(async move {
                    let new_tracks: Vec<viola_common::Track> =
                        Request::get("127.0.0.1:8080/playlist/")
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
        self.tracks
            .iter()
            .map(|track| {
                html! {
                    <tr>
                        <td>{self.index}</td>
                        <td>{unwrap_or_empty(&track.tracknumber)}</td>
                        <td>{&track.title}</td>
                        <td>{&track.artist}</td>
                        <td>{&track.album}</td>
                        <td>{&track.genre}</td>
                        <td>{unwrap_or_empty(&track.year)}</td>
                        <td>{track.length}</td>
                        <td>{unwrap_or_empty(&track.playcount)}</td>
                    </tr>
                }
            })
            .collect::<Html>()
    }
}

enum Msg {
    AddOne,
}

struct Model {
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();
        html! {
            <div>
                <button onclick={link.callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::start_app::<Model>();
}
