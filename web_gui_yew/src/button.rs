use gloo_net::http::Request;
use viola_common::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct ButtonRowProps {
    pub(crate) status: GStreamerMessage,
    pub(crate) repeat_once_callback: Callback<()>,
    pub(crate) refresh_play_callback: Callback<()>,
    pub(crate) sidebar_callback: Callback<()>,
    pub(crate) delete_range_callback: Callback<()>,
}

#[function_component(Buttons)]
pub(crate) fn buttons(props: &ButtonRowProps) -> Html {
    let playpause_button = if props.status == GStreamerMessage::Playing {
        html! {

            <TransportButton text="Pause" icon="/pause.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Pausing)} callback={props.refresh_play_callback.clone()} />
        }
    } else {
        html! {
            <TransportButton text="Play" icon="/play.svg" btype={ButtonType::Success} on_click={Some(GStreamerAction::Playing)} callback={props.refresh_play_callback.clone()} />
        }
    };

    html! {
        <div class="flex gap-4 px-8 py-8">
            <div class="contents">
            <CallbackButton text="Menu" icon="/menu-button.svg" btype={ButtonType::Info} callback={props.sidebar_callback.clone()} />
            <TransportButton text="Prev" icon="/skip-backward.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Previous)} callback={props.refresh_play_callback.clone()} />
            {playpause_button}
            <TransportButton text="Pause" icon="/pause.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Pausing)} callback={props.refresh_play_callback.clone()} />
            <TransportButton text="Next" icon="/skip-forward.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Next)} callback={props.refresh_play_callback.clone()} />
            <TransportButton text="Again" icon="/arrow-repeat.svg" btype={ButtonType::Secondary} on_click={Some(GStreamerAction::RepeatOnce)} callback = {props.repeat_once_callback.clone()} />
            <UrlCallButton text="Clean" icon="/trash.svg" btype={ButtonType::Danger} url_call = {"/clean/"} />
            <CallbackButton text="Delete Range" icon="/trash.svg" btype={ButtonType::Danger} callback={props.delete_range_callback.clone()} />
        </div>
    </div>}
}

#[derive(Clone, PartialEq)]
pub(crate) enum ButtonType {
    Info,
    Primary,
    Secondary,
    Danger,
    Success,
}

#[derive(Clone, Properties, PartialEq)]
struct TransportButtonProps {
    text: String,
    icon: String,
    btype: ButtonType,
    on_click: Option<GStreamerAction>,
    callback: Callback<()>,
}

pub(crate) enum ButtonMsg {
    Clicked,
    Done,
}

fn icon(path: String, size: Option<usize>) -> Html {
    let size = (size.unwrap_or(24)).to_string() + "px";
    html! {
            <img src ={path} height={size.clone()} width={size} />
    }
}

struct TransportButton;
impl Component for TransportButton {
    type Message = ButtonMsg;
    type Properties = TransportButtonProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let class = String::from("btn ")
            + match ctx.props().btype {
                ButtonType::Info => "bg-cyan-500 text",
                ButtonType::Primary => "bg-cyan-500 text",
                ButtonType::Secondary => "btn-secondary",
                ButtonType::Danger => "btn-danger",
                ButtonType::Success => "btn-success",
            };
        let class = "flex-1 bg-cyan-500 text px-4 py-4 font-semibold text-sm text-white rounded-full";
        let onclick = ctx.link().callback(|_| ButtonMsg::Clicked);
        let icon_path: String = ctx.props().icon.clone();
        html! {
                <>
                    <button class={class} onclick={onclick}>
                    {icon(icon_path,None)}
                    { &ctx.props().text}</button>
                </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ButtonMsg::Clicked => {
                if let Some(on_click) = ctx.props().on_click {
                    ctx.link().send_future(async move {
                        Request::post("/transport/")
                            .header("Content-Type", "application/json")
                            .body(serde_json::to_string(&on_click).unwrap())
                            .unwrap()
                            .send()
                            .await
                            .unwrap();
                        ButtonMsg::Done
                    });
                }
            }
            ButtonMsg::Done => {
                ctx.props().callback.emit(());
            }
        };
        false
    }
}

pub(crate) struct UrlCallButton;

#[derive(Clone, Properties, PartialEq)]
pub(crate) struct UrlCallButtonProps {
    pub(crate) text: String,
    pub(crate) icon: String,
    pub(crate) btype: ButtonType,
    pub(crate) url_call: String,
}

impl Component for UrlCallButton {
    type Message = ButtonMsg;
    type Properties = UrlCallButtonProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ButtonMsg::Clicked => {
                ctx.link().send_future(async move {
                    Request::post("/clean/").send().await.unwrap();
                    ButtonMsg::Done
                });
            }
            ButtonMsg::Done => {}
        };
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let class = String::from("btn ")
            + match ctx.props().btype {
                ButtonType::Info => "btn-info",
                ButtonType::Primary => "btn-primary",
                ButtonType::Secondary => "btn-secondary",
                ButtonType::Danger => "btn-danger",
                ButtonType::Success => "btn-success",
            };
            let class = "flex-1 bg-cyan-500 text px-4 py-4 font-semibold text-sm text-white rounded-full";
        let onclick = ctx.link().callback(|_| ButtonMsg::Clicked);
        let icon_path: String = ctx.props().icon.clone();
        html! {
                <>
                    <button class={class} onclick={onclick}>
                    {icon(icon_path,None)}
                    { &ctx.props().text}</button>
                </>
        }
    }
}

pub(crate) struct CallbackButton;

#[derive(Clone, Properties, PartialEq)]
pub(crate) struct ButtonProps {
    pub(crate) text: String,
    pub(crate) icon: String,
    pub(crate) btype: ButtonType,
    pub(crate) callback: Callback<()>,
}

impl Component for CallbackButton {
    type Message = ButtonMsg;
    type Properties = ButtonProps;

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
                ButtonType::Success => "btn-success",
            };
            let class = "bg-cyan-500 text px-4 py-4 font-semibold text-sm text-white rounded-full";
        let onclick = ctx.link().callback(|_| ButtonMsg::Clicked);
        let icon_path: String = ctx.props().icon.clone();
        html! {
                <>
                    <button class={class} onclick={onclick}>
                    {icon(icon_path,None)}
                    { &ctx.props().text}</button>
                </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ButtonMsg::Clicked => {
                ctx.props().callback.emit(());
            }
            ButtonMsg::Done => {}
        };
        false
    }
}
