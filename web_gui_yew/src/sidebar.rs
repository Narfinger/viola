use crate::button::*;
use crate::treeview::TreeViewLvl1;
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
    TreeViewToggle(usize),
}

pub(crate) struct Sidebar {
    smartplaylist_visible: bool,
    smartplaylists: Smartplaylists,
    treeview_visible: Vec<bool>,
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
            treeview_visible: vec![false],
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
            SidebarMsg::TreeViewToggle(index) => {
                self.treeview_visible[index] = !self.treeview_visible[index];
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
                    let s = viola_common::LoadSmartPlaylistJson { index };
                    Request::post("/smartplaylist/load/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&s).unwrap())
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

        let sm_modal = html! {
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

        let style = if self.treeview_visible[0] {
            "display: block"
        } else {
            ""
        };
        let a_modal = html! {
            <div class="modal" tabindex="-1" role="dialog" style={style}>
                <div class="modal-dialog" role="document">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title">{"Smart Playlists"}</h5>
                        </div>
                        <div class="modal-body">
                            <TreeViewLvl1 type_vec={vec![TreeType::Artist, TreeType::Album, TreeType::Track]} />
                        </div>
                        <div class="modal-footer">
                            <CallbackButton
                                text="Close"
                                icon="/trash.svg"
                                btype={ButtonType::Danger}
                                callback={ctx.link().callback(|_| SidebarMsg::TreeViewToggle(0))}
                            />
                        </div>
                    </div>
                </div>
            </div>
        };

        html! {
            <>
                {sm_modal}
                {a_modal}
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

                        <li class="nav-item" style="padding: 5px">
                            <CallbackButton
                                text={"Artist"}
                                icon={""}
                                btype={ButtonType::Primary}
                                callback = {ctx.link().callback(|_| SidebarMsg::TreeViewToggle(0))}
                                />
                        </li>
                    </ul>
                </div>
            </>
        }
    }
}
