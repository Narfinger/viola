use reqwasm::http::Request;
use viola_common::*;
use yew::prelude::*;

pub(crate) struct TabsComponent {}

pub(crate) enum TabsMessage {
    Delete(usize),
    Change(usize),
    ReloadEmit,
}

#[derive(Properties, PartialEq)]
pub(crate) struct TabsProperties {
    pub(crate) reload_tabs_callback: Callback<()>,
    pub(crate) tabs: PlaylistTabsJSON,
}

impl Component for TabsComponent {
    type Message = TabsMessage;
    type Properties = TabsProperties;

    fn create(ctx: &Context<Self>) -> Self {
        TabsComponent {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TabsMessage::Delete(i) => {
                ctx.link().send_future(async move {
                    Request::delete(&format!("/playlisttab/{}/", i))
                        .send()
                        .await
                        .unwrap();
                    TabsMessage::ReloadEmit
                });
                ctx.props().reload_tabs_callback.emit(());
                false
            }
            TabsMessage::Change(i) => {
                ctx.link().send_future(async move {
                    Request::delete("/playlisttab/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&i).unwrap())
                        .send()
                        .await
                        .unwrap();
                    TabsMessage::ReloadEmit
                });
                false
            }
            TabsMessage::ReloadEmit => {
                ctx.props().reload_tabs_callback.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let inner = ctx.props()
            .tabs
            .tabs
            .iter()
            .enumerate()
            .map(|(pos, tab)| {
                html! {
                    <li class="nav-item">
                        <a href="#"
                            onclick={ ctx.link().callback(move |_| TabsMessage::Change(pos))}
                        class={if pos == ctx.props().tabs.current {
                            "nav-link active"
                        } else {"nav-link"}}>
                        {&tab.name}
                        <span style="padding-left: 5px;">
                            <img src="/x-square.svg" height="16px" width="16px" onclick={ ctx.link().callback(move |_| TabsMessage::Delete(pos))} />
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
