use reqwasm::http::Request;
use viola_common::*;
use yew::prelude::*;

pub(crate) struct TabsComponent {
    tabs: Vec<PlaylistTabJSON>,
    current: usize,
}

pub(crate) enum TabsMessage {
    Load,
    LoadDone(PlaylistTabsJSON),
    Add(usize),
    Delete(usize),
    Change(usize),
}

impl Component for TabsComponent {
    type Message = TabsMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(TabsMessage::Load);
        TabsComponent {
            current: 0,
            tabs: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TabsMessage::Load => {
                ctx.link().send_future(async move {
                    let tabs: PlaylistTabsJSON = Request::get("/playlisttab/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    TabsMessage::LoadDone(tabs)
                });
                false
            }
            TabsMessage::LoadDone(loaded_tabs) => {
                self.current = loaded_tabs.current;
                self.tabs = loaded_tabs.tabs;
                true
            }
            TabsMessage::Add(_) => todo!(),
            TabsMessage::Delete(_) => todo!(),
            TabsMessage::Change(i) => {
                ctx.link().send_future(async move {
                    Request::post("/playlisttab/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&i).unwrap())
                        .send()
                        .await
                        .unwrap();
                    TabsMessage::Load
                });
                self.current = i;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let inner = self
            .tabs
            .iter()
            .enumerate()
            .map(|(pos, tab)| {
                html! {
                    <li class="nav-item">
                        <a href="#"
                            onclick={ ctx.link().callback(move |_| TabsMessage::Change(pos))}
                        class={if pos == self.current {
                            "nav-link active"
                        } else {"nav-link"}}>
                        {&tab.name}
                        <span style="padding-left: 5px;" onclick={ ctx.link().callback(move |_| TabsMessage::Delete(pos))}>
                            <img src="/x-square.svg" height="8px" width="8px" />
                        </span>
                        </a>
                    </li>
                }
            })
            .collect::<Html>();

        html! {
            <div class="container">
                <div class="row">
                    <div class="col">
                        <ul class="nav nav-tabs">
                        {inner}
                        </ul>
                    </div>
                </div>
            </div>
        }
    }
}
