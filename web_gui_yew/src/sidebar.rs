use crate::button::*;
use reqwasm::http::Request;
use viola_common::*;
use yew::prelude::*;

#[derive(PartialEq)]
pub(crate) enum SidebarMsg {
    Close,
    SmartPlaylistToggle,
    LoadSmartPlaylistNames,
    LoadSmartPlaylistNamesDone(Vec<String>),
    LoadSmartPlaylist(usize),
}

pub(crate) struct Sidebar {
    smartplaylist_visible: bool,
    smartplaylists: Vec<String>,
}

#[derive(Properties, PartialEq)]
pub(crate) struct SidebarProperties {
    pub(crate) visible: bool,
    pub(crate) close_callback: Callback<()>,
    pub(crate) reload_callback: Callback<()>,
}

impl Component for Sidebar {
    type Message = SidebarMsg;
    type Properties = SidebarProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            smartplaylist_visible: false,
            smartplaylists: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SidebarMsg::Close => {
                self.smartplaylist_visible = false;
                ctx.props().close_callback.emit(());
                self.smartplaylists = vec![];
                true
            }
            SidebarMsg::SmartPlaylistToggle => {
                self.smartplaylist_visible = true;
                ctx.link().send_message(SidebarMsg::LoadSmartPlaylistNames);
                true
            }
            SidebarMsg::LoadSmartPlaylistNames => {
                ctx.link().send_future(async move {
                    let pls: Vec<String> = Request::get("/smartplaylist/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap_or_default();
                    SidebarMsg::LoadSmartPlaylistNamesDone(pls)
                });
                false
            }
            SidebarMsg::LoadSmartPlaylistNamesDone(v) => {
                self.smartplaylists = v;
                true
            }
            SidebarMsg::LoadSmartPlaylist(index) => {
                ctx.link().send_future(async move {
                    Request::post("/smartplaylist/load/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&index).unwrap())
                        .send()
                        .await
                        .unwrap();
                    SidebarMsg::Close
                });
                ctx.props().reload_callback.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let class_string = if ctx.props().visible {
            "col-xs"
        } else {
            "col-xs collapse"
        };
        let style = if self.smartplaylist_visible {
            "display: block"
        } else {
            ""
        };

        let modal = html! {
            <div class="modal" tabindex="-1" role="dialog" style={style}>
                <div class="modal-dialog" role="document">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title">{"Smart Playlists"}</h5>
                        </div>
                        <div class="modal-body">
                            <ul>
                                {self.smartplaylists.iter().enumerate().map(|(i, s)| {
                                    html!{
                                        <li
                                        onclick={ctx.link().callback(move |_| SidebarMsg::LoadSmartPlaylist(i))}
                                        >{s}
                                        </li>
                                    }
                                }).collect::<Html>()}
                            </ul>
                        </div>
                        <div class="modal-footer">
                            <CallbackButton
                                text="Close"
                                icon="/trash.svg"
                                btype={ButtonType::Danger}
                                callback={ctx.link().callback(|_| SidebarMsg::Close)}
                            />
                        </div>
                    </div>
                </div>
            </div>
        };

        html! {
            <>
                {modal}
                <div class={class_string} style="width: 20%; padding: 20px">
                    <ul class="navbar-nav">
                        <li class="nav-item" style="padding: 5px">
                            <CallbackButton
                                text={"Smartplaylist"}
                                icon={""}
                                btype={ButtonType::Primary}
                                callback = {ctx.link().callback(|_| SidebarMsg::SmartPlaylistToggle)}
                                />
                        </li>
                    </ul>
                </div>
            </>
        }
    }
}
