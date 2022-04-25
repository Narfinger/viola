use viola_common::GStreamerAction;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub(crate) enum ButtonType {
    Info,
    Primary,
    Secondary,
    Danger,
}

#[derive(Clone, Properties, PartialEq)]
pub(crate) struct ButtonProbs {
    pub(crate) text: String,
    pub(crate) icon: String,
    pub(crate) btype: ButtonType,
    pub(crate) on_click: Option<GStreamerAction>,
}

pub(crate) enum ButtonMsg {
    Clicked,
}

pub(crate) struct Button;

impl Component for Button {
    type Message = ButtonMsg;
    type Properties = ButtonProbs;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let class = String::from("btn ")
            + match ctx.props().btype {
                ButtonType::Info => "btn-info",
                ButtonType::Primary => "btn-primary",
                ButtonType::Secondary => "btn-secondary",
                ButtonType::Danger => "btn-danger",
            };
        let onclick = ctx.link().callback(|_| ButtonMsg::Clicked);
        let icon: String = ctx.props().icon.clone();
        html! {
            <div class="col">
                <button class={class} icon={ icon } onclick={onclick}>{ &ctx.props().text}</button>
            </div>
        }
    }
}
