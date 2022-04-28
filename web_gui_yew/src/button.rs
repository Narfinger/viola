use reqwasm::http::Request;
use viola_common::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct ButtonRowProps {
    pub(crate) status: GStreamerMessage,
    pub(crate) repeat_once_callback: Callback<()>,
}

#[function_component(Buttons)]
pub(crate) fn buttons(props: &ButtonRowProps) -> Html {
    let playpause_button = if props.status == GStreamerMessage::Playing {
        html! {

            <Button text="Pause" icon="/pause.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Pausing)} />
        }
    } else {
        html! {
            <Button text="Play" icon="/play.svg" btype={ButtonType::Success} on_click={Some(GStreamerAction::Playing)} />
        }
    };

    html! {
    <div class="row">
        <div class="col">
            <Button text="Menu" icon="/menu-button.svg" btype={ButtonType::Info} on_click={None} />
        </div>
        <div class="col">
            <Button text="Prev" icon="/skip-backward.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Previous)}  />
        </div>
        <div class="col">
            {playpause_button}
        </div>
        <div class="col">
            <Button text="Pause" icon="/pause.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Pausing)} />
        </div>
        <div class="col">
            <Button text="Next" icon="/skip-forward.svg" btype={ButtonType::Primary} on_click={Some(GStreamerAction::Next)} />
        </div>
        <div class="col">
            <Button text="Again" icon="/arrow-repeat.svg" btype={ButtonType::Secondary} on_click={Some(GStreamerAction::RepeatOnce)} callback = {props.repeat_once_callback.clone()} />
        </div>
        <div class="col">
            <Button text="Clean" icon="/trash.svg" btype={ButtonType::Danger} on_click={None}/>
        </div>
        <div class="col-2">
            <Button text="Delete Range" icon="/trash.svg" btype={ButtonType::Danger} on_click={None} />
        </div>
    </div>}
}

#[derive(Clone, PartialEq)]
enum ButtonType {
    Info,
    Primary,
    Secondary,
    Danger,
    Success,
}

#[derive(Clone, Properties, PartialEq)]
struct ButtonProbs {
    text: String,
    icon: String,
    btype: ButtonType,
    on_click: Option<GStreamerAction>,
    callback: Option<Callback<()>>,
}

enum ButtonMsg {
    Clicked,
    Nop,
}

struct Button;

fn icon(path: String, size: Option<usize>) -> Html {
    let size = (size.unwrap_or(24)).to_string() + "px";
    html! {
        <span style="padding-right: 5px">
            <img src ={path} height={size.clone()} width={size} />
        </span>
    }
}

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
                        ButtonMsg::Nop
                    });
                }
                if let Some(ref callback) = ctx.props().callback {
                    callback.emit(());
                }
            }
            ButtonMsg::Nop => {}
        };
        false
    }
}
