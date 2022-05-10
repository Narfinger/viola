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
    ShowFullPlaylistWindow,
    Save,
}

struct TreeView {
    name: String,
    ttype: Vec<TreeType>,
    visible: bool,
}

pub(crate) struct Sidebar {
    smartplaylist_visible: bool,
    smartplaylists: Smartplaylists,
    treeviews: Vec<TreeView>,
}

#[derive(Properties, PartialEq)]
pub(crate) struct SidebarProperties {
    pub(crate) visible: bool,
    pub(crate) close_callback: Callback<()>,
    pub(crate) reload_callback: Callback<()>,
    pub(crate) show_all_tracks_callback: Callback<()>,
}

impl Component for Sidebar {
    type Message = SidebarMsg;
    type Properties = SidebarProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        let treeviews = vec![
            TreeView {
                name: String::from("Artist"),
                ttype: vec![TreeType::Artist, TreeType::Album, TreeType::Track],
                visible: false,
            },
            TreeView {
                name: String::from("Album"),
                ttype: vec![TreeType::Album, TreeType::Track],
                visible: false,
            },
            TreeView {
                name: String::from("Track"),
                ttype: vec![TreeType::Track],
                visible: false,
            },
            TreeView {
                name: String::from("Genre"),
                ttype: vec![TreeType::Genre, TreeType::Artist, TreeType::Album],
                visible: false,
            },
        ];
        Self {
            smartplaylist_visible: false,
            smartplaylists: vec![],
            treeviews,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SidebarMsg::Close => {
                self.smartplaylist_visible = false;
                for mut i in self.treeviews.iter_mut() {
                    i.visible = false;
                }
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
                let old = self.treeviews.get(index).unwrap().visible;
                self.treeviews.get_mut(index).unwrap().visible = !old;
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
            SidebarMsg::ShowFullPlaylistWindow => {
                ctx.link().send_message(SidebarMsg::Close);
                ctx.props().show_all_tracks_callback.emit(());
                false
            }
            SidebarMsg::Save => {
                ctx.link().send_future(async move {
                    Request::post("/save/").send().await.unwrap();
                    SidebarMsg::Close
                });
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

        let treeviews = self.treeviews.iter().map(|t| {
        let style = if t.visible {
            "display: block"
        } else {
            ""
        };
        html! {
            <div class="modal" tabindex="-1" role="dialog" style={style}>
                <div class="modal-dialog" role="document">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title">{"Smart Playlists"}</h5>
                        </div>
                        <div class="modal-body">
                            <TreeViewLvl1
                                type_vec={t.ttype.clone()}
                                close_callback={ctx.link().callback(|_| SidebarMsg::Close)} />
                        </div>
                        <div class="modal-footer">
                            <CallbackButton
                                text="Close"
                                icon="/trash.svg"
                                btype={ButtonType::Danger}
                                callback={ctx.link().callback(move |_| SidebarMsg::Close)}
                            />
                        </div>
                    </div>
                </div>
            </div>
        }}).collect::<Html>();

        let treeviews_buttons = self.treeviews.iter().enumerate().map(|(index, tv)| {
            html!{
                <li class="nav-item" style="padding: 5px">
                    <CallbackButton
                        text={tv.name.clone()}
                        icon={"/list-nested.svg"}
                        btype={ButtonType::Primary}
                        callback = {ctx.link().callback(move |_| SidebarMsg::TreeViewToggle(index))}
                        />
                </li>
            }
        }).collect::<Html>();

        html! {
            <>
                {sm_modal}
                {treeviews}
                <div class={class_string} style="width: 20%; padding: 20px">
                    <ul class="navbar-nav">
                        <li class="nav-item" style="padding: 5px">
                            <CallbackButton
                                text={"Smartplaylist"}
                                icon={"/list-nested.svg"}
                                btype={ButtonType::Primary}
                                callback = {ctx.link().callback(|_| SidebarMsg::SmartPlaylistToggle)}
                                />
                        </li>
                        {treeviews_buttons}
                        <li class="nav-item" style="padding: 5px">
                            <CallbackButton
                                text={"Show Full Playlist Window"}
                                icon={"/window-fullscreen.svg"}
                                btype={ButtonType::Danger}
                                callback = {ctx.link().callback(|_| SidebarMsg::ShowFullPlaylistWindow)}
                                />
                        </li>
                        <li class="nav-item" style="padding: 5px">
                            <CallbackButton
                                text={"Save"}
                                icon={"/save.svg"}
                                btype={ButtonType::Danger}
                                callback = {ctx.link().callback(|_| SidebarMsg::Save)}
                                />
                        </li>
                    </ul>
                </div>
            </>
        }
    }
}
