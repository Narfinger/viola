use crate::button::*;
use yew::prelude::*;

#[derive(PartialEq)]
pub(crate) enum SidebarMsg {
    SmartPlaylistToggle,
}

pub(crate) struct Sidebar {
    smartplaylist_visible: bool,
}

#[derive(Properties, PartialEq)]
pub(crate) struct SidebarProperties {
    pub(crate) visible: bool,
}

impl Component for Sidebar {
    type Message = SidebarMsg;
    type Properties = SidebarProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            smartplaylist_visible: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SidebarMsg::SmartPlaylistToggle => {
                self.smartplaylist_visible = true;
                true
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
                            <h5 class="modal-title">{"Modal title"}</h5>
                            <button type="button" class="close" data-dismiss="modal" aria-label="Close">
                                <span aria-hidden="false">{"&times;"}</span>
                            </button>
                        </div>
                        <div class="modal-body">
                            <p>{"Modal body text goes here."}</p>
                        </div>
                        <div class="modal-footer">
                            <button type="button" class="btn btn-primary">{"Save changes"}</button>
                            <button type="button" class="btn btn-secondary" data-dismiss="modal">{"Close"}</button>
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
