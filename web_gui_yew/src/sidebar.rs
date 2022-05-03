use crate::button::*;
use yew::prelude::*;

#[derive(PartialEq)]
pub(crate) enum SidebarMsg {
    SmartPlaylistToggle,
}

pub(crate) struct Sidebar {}

#[derive(Properties, PartialEq)]
pub(crate) struct SidebarProperties {
    pub(crate) visible: bool,
}

impl Component for Sidebar {
    type Properties = SidebarProperties;
    type Message = SidebarMsg;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let class_string = if ctx.props().visible {
            "col-xs"
        } else {
            "col-xs collapse"
        };
        html! {
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
        }
    }
}
