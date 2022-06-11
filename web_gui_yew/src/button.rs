use reqwasm::http::Request;
use viola_common::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct ButtonRowProps {
    pub(crate) status: GStreamerMessage,
    pub(crate) repeat_once_callback: Callback<()>,
    pub(crate) clean_callback: Callback<()>,
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
    <div class="row">
        <div class="col">
            <CallbackButton text="Menu" icon="/menu-button.svg" btype={ButtonType::Info} callback={props.sidebar_callback.clone()} />
        </div>
        <div class="col">
            <TransportButton text="Prev" icon="/skip-backward.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Previous)} callback={props.refresh_play_callback.clone()} />
        </div>
        <div class="col">
            {playpause_button}
        </div>
        <div class="col">
            <TransportButton text="Pause" icon="/pause.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Pausing)} callback={props.refresh_play_callback.clone()} />
        </div>
        <div class="col">
            <TransportButton text="Next" icon="/skip-forward.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Next)} callback={props.refresh_play_callback.clone()} />
        </div>
        <div class="col">
            <TransportButton text="Again" icon="/arrow-repeat.svg" btype={ButtonType::Secondary} on_click={Some(GStreamerAction::RepeatOnce)} callback = {props.repeat_once_callback.clone()} />
        </div>
        <div class="col">
            <UrlCallButton text="Clean" icon="/trash.svg" btype={ButtonType::Danger}  callback={props.clean_callback.clone()} url_call = {"/clean/"} />
        </div>
        <div class="col-2">
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
        <span style="padding-right: 5px">
            <img src ={path} height={size.clone()} width={size} />
        </span>
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
                ButtonType::Info => "btn-info",
                ButtonType::Primary => "btn-primary",
                ButtonType::Secondary => "btn-secondary",
                ButtonType::Danger => "btn-danger",
                ButtonType::Success => "btn-success",
            };
        let onclick = ctx.link().callback(|_| ButtonMsg::Clicked);
        let icon_path: String = ctx.props().icon.clone();
        html! {
                <div class="col">
                    <button class={class} onclick={onclick}>
                    {icon(icon_path,None)}
                    { &ctx.props().text}</button>
                </div>
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
    pub(crate) callback: Callback<()>,
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
            ButtonMsg::Done => {
                ctx.props().callback.emit(());
            }
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
        let onclick = ctx.link().callback(|_| ButtonMsg::Clicked);
        let icon_path: String = ctx.props().icon.clone();
        html! {
                <div class="col">
                    <button class={class} onclick={onclick}>
                    {icon(icon_path,None)}
                    { &ctx.props().text}</button>
                </div>
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
        let onclick = ctx.link().callback(|_| ButtonMsg::Clicked);
        let icon_path: String = ctx.props().icon.clone();
        html! {
                <div class="col">
                    <button class={class} onclick={onclick}>
                    {icon(icon_path,None)}
                    { &ctx.props().text}</button>
                </div>
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
