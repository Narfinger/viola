use gloo_net::http::Request;
use viola_common::*;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub(crate) struct TabsComponent {
    edit: Option<usize>,
    current_edit_text: Option<String>,
}

#[derive(Debug)]
pub(crate) enum TabEditMsg {
    ShowEdit(usize),
    EditChange(Event),
    FinishedEdit(i32),
    ChangeName(usize, String),
}

#[derive(Debug)]
pub(crate) enum TabsMessage {
    Delete(usize),
    Change(usize),
    TabEdit(TabEditMsg),
    ReloadEmit,
}

#[derive(Properties, PartialEq)]
pub(crate) struct TabsProperties {
    pub(crate) tabs: PlaylistTabsJSON,
}

impl TabsComponent {
    fn view_edit_button(&self, pos: usize, ctx: &Context<Self>) -> Html {
        let tab_name = ctx.props().tabs.tabs.get(pos).unwrap().name.clone();
        html! {
            <div class="input-group mb-3">
            <div class="input-group-prepend">
              <button onclick={ctx.link().callback(move |_| TabsMessage::TabEdit(TabEditMsg::FinishedEdit(pos as i32)))} class="btn btn-primary" type="button">{"Change"}</button>
            </div>
            <input type="text" onchange={ctx.link().callback(move |e: Event| TabsMessage::TabEdit(TabEditMsg::EditChange(e)))} class="form-control" placeholder={tab_name}/>
          </div>
        }
    }
}

impl Component for TabsComponent {
    type Message = TabsMessage;
    type Properties = TabsProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        TabsComponent {
            edit: None,
            current_edit_text: None,
        }
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
                false
            }
            TabsMessage::Change(i) => {
                if Some(i) != self.edit {
                    self.edit = None;
                    ctx.link().send_future(async move {
                        Request::post("/playlisttab/")
                            .header("Content-Type", "application/json")
                            .body(serde_json::to_string(&i).unwrap())
                            .unwrap()
                            .send()
                            .await
                            .unwrap();
                        TabsMessage::ReloadEmit
                    });
                    true
                } else {
                    false
                }
            }
            TabsMessage::TabEdit(TabEditMsg::ChangeName(pos, name)) => {
                let current_position = ctx.props().tabs.tabs.get(pos).unwrap().current_position;
                // Notice that the position and ids are different
                let id = ctx.props().tabs.tabs.get(pos).unwrap().id;
                self.edit = None;
                self.current_edit_text = None;
                ctx.link().send_future(async move {
                    let val = PlaylistTabJSON {
                        id,
                        current_position,
                        name,
                    };
                    Request::put("/playlisttab/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&val).unwrap())
                        .unwrap()
                        .send()
                        .await
                        .unwrap();
                    TabsMessage::ReloadEmit
                });
                true
            }
            TabsMessage::TabEdit(TabEditMsg::EditChange(e)) => {
                let input: HtmlInputElement = e.target_unchecked_into();
                self.current_edit_text = Some(input.value());
                false
            }
            TabsMessage::TabEdit(TabEditMsg::ShowEdit(pos)) => {
                self.edit = Some(pos);
                self.current_edit_text = None;
                true
            }
            TabsMessage::TabEdit(TabEditMsg::FinishedEdit(pos)) => {
                let s = self.current_edit_text.as_ref().unwrap().clone();
                ctx.link()
                    .send_message(TabsMessage::TabEdit(TabEditMsg::ChangeName(
                        pos as usize,
                        s,
                    )));
                false
            }
            TabsMessage::ReloadEmit => false,
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
                        } else {"nav-link"}}
                        >
                        {
                            if self.edit == Some(pos) {
                                self.view_edit_button(pos, ctx)
                            } else {
                                html! {
                                <span ondblclick = {ctx.link().callback(move |_| TabsMessage::TabEdit(TabEditMsg::ShowEdit(pos)))}>
                                    {&tab.name}
                                </span>
                            }
                            }
                        }
                        if self.edit.is_none() {
                                <span style="padding-left: 5px;">
                                    <img src="/x-square.svg" height="16px" width="16px" onclick={ ctx.link().callback(move |_| TabsMessage::Delete(pos))} />
                                </span>
                        }
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
