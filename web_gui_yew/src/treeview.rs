use indextree::{Arena, NodeId};
use reqwasm::http::Request;
use viola_common::*;
use yew::prelude::*;

/*
[derive(Properties, PartialEq)]
struct TreeViewLvl3Props {
    treeview: TreeView,
    model_index: usize,
    index: usize,
    index2: usize,
    //el: indextree::NodeId
}

#[function_component]
fn TreeViewLvl3(props: &TreeViewLvl3Props) -> Html {
    let children = el.children.
    html! {
        <ul style="list-style-type: disclosure-closed">
            {chilren}
    }

    ul![
        style!(St::from("list-style-type") => "disclosure-closed"),
        el
    .children(&treeview.tree)
    .filter(|node| !node.is_removed(&treeview.tree))
    .enumerate()
    .map(|(index3, el2)| {
        li![span![
            treeview.tree.get(el2).unwrap().get(),
            button![
                C!["btn", "btn-outline-primary", "btn-sm"],
                style!(St::MarginLeft => unit!(25,px)),
                attrs!(At::from("data-bs-dismiss") => "modal", At::from("data-bs-target") => "artisttree"),
                "Load",
                ev(Ev::Click, move |_| Msg::LoadFromTreeView {
                    tree_index: vec![index, index2, index3],
                    model_index
                })
            ],
        ]]
    })]
}

fn view_tree_lvl2(
    treeview: &TreeView,
    model_index: usize,
    index: usize,
    nodeid: indextree::NodeId,
) -> Node<Msg> {
    let no_children = nodeid.children(&treeview.tree).count() == 0;
    ul![
        IF!(no_children => style!(St::from("list-style-type") => "disclosure-closed")),
        IF!(!no_children => style!(St::from("list-style-type") => "disclosure-closed")),
    nodeid
    .children(&treeview.tree)
    .filter(|node| !node.is_removed(&treeview.tree))
    .enumerate()
    .map(|(index2, el)| {
        li![
            span![
                span![
                    treeview.tree.get(el).unwrap().get(),
                    mouse_ev(Ev::Click, move |_| Msg::FillTreeView {
                        model_index,
                        tree_index: vec![index, index2],
                        search: SearchString::UseStoredSearch,
                    }),
                ],
                button![
                    C!["btn", "btn-outline-primary", "btn-sm"],
                    style!(St::MarginLeft => unit!(25,px)),
                    attrs!(At::from("data-bs-dismiss") => "modal", At::from("data-bs-target") => "artisttree"),
                    "Load",
                    ev(Ev::Click, move |_| Msg::LoadFromTreeView {
                        tree_index: vec![index, index2],
                        model_index
                    })
                ],
            ],
                    view_tree_lvl3(treeview, model_index, index, index2, el)
        ]
                })
    ]
}

*/

pub(crate) enum SearchString {
    UseStoredSearch,
    EmptySearch,
}

#[derive(Properties, PartialEq)]
pub(crate) struct TreeViewLvl1Props {
    pub(crate) type_vec: Vec<viola_common::TreeType>,
}

pub(crate) enum Msg {
    FillTreeView {
        search: SearchString,
    },
    FillTreeViewRecv {
        result: Vec<String>,
        query: TreeViewQuery,
    },
    LoadFromTreeView,
}

pub(crate) struct TreeViewLvl1 {
    tree: indextree::Arena<String>,
    root: indextree::NodeId,
}

impl TreeViewLvl1 {
    fn tree_index_to_nodeid(&self, tree_index: &[usize]) -> Option<NodeId> {
        match tree_index.len() {
            0 => {
                // this means we are the second message, hence we need to clear our arena (and make a new root node)
                //let mut arena = indextree::Arena::new();
                //let root = arena.new_node("".to_string());
                //treeview.tree = arena;
                //treeview.root = root;
                //Some(treeview.root)
                None
            }
            1 => self.root.children(&self.tree).nth(tree_index[0]),
            2 => self
                .root
                .children(&self.tree)
                .nth(tree_index[0])
                .map(|t| t.children(&self.tree))
                .and_then(|mut t| t.nth(tree_index[1])),
            _ => None,
        }
    }
}

impl Component for TreeViewLvl1 {
    type Message = Msg;

    type Properties = TreeViewLvl1Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut tree = Arena::new();
        let root = tree.new_node("".to_string());
        ctx.link().send_message(Msg::FillTreeView {
            search: SearchString::EmptySearch,
        });
        Self { tree, root }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FillTreeView { search } => {
                let type_vec = ctx.props().type_vec.clone();

                ctx.link().send_future(async move {
                    let data = viola_common::TreeViewQuery {
                        indices: vec![],
                        types: type_vec,
                        search: None,
                    };
                    let result: Vec<String> = Request::post("/libraryview/partial/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&data).unwrap())
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    Msg::FillTreeViewRecv {
                        result,
                        query: data,
                    }
                });
                false
            }
            Msg::FillTreeViewRecv { result, query } => {
                for i in result {
                    let new_node = self.tree.new_node(i);
                    self.root.append(new_node, &mut self.tree);
                }
                true
            }
            Msg::LoadFromTreeView => todo!(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let nodes = self
            .root
            .children(&self.tree)
            .map(|nodeid| {
                html! {
                    <li>
                        <span style="list-style-type: disclosure-closed">
                            <span onclick={ctx.link().callback(move |_|
                                Msg::FillTreeView{
                                    search: SearchString::UseStoredSearch}
                                )}>
                                    {self.tree.get(nodeid).unwrap().get()}
                            </span>
                            <button
                                class="btn btn-outline-primary btn-sm" style="margin-left: 25px"
                                onclick={ctx.link().callback(|_| Msg::LoadFromTreeView)}>
                                {"Load"}
                            </button>
                        </span>
                    //<TreeViewLvl2 />
                    </li>

                }
            })
            .collect::<Html>();

        html! {
            <ul>
            {nodes}
            </ul>
        }
    }
}

/*
fn view_tree(props: &TreeProps) -> Html {
    //if let Some(treeview) = model.treeviews.get(model_index) {
    treeviews
        .iter()
        .enumerate()
        .map(|(model_index, t)| {
            div![
                C!["modal", "fade"],
                attrs![At::from("aria-hidden") => "true", At::Id => t.treeview_html.id],
                div![
                    C!["modal-dialog"],
                    div![
                        C!["modal-content"],
                        div![
                            C!["modal-body"],
                            div![
                                C!["row"],
                                div![
                                    C!["col"],
                                    input![
                                        C!["form-control"],
                                        attrs!(At::from("placeholder") => "Search"),
                                        input_ev(Ev::Input, move |search| Msg::FillTreeView {
                                            model_index,
                                            tree_index: vec![],
                                            search: SearchString::UpdateSearch(search),
                                        },)
                                    ],
                                ],
                                div![
                                    C!["col"],
                                    button![
                                        C!["btn", "btn-outline-primary", "btn-sm"],
                                        "Load All",
                                        ev(Ev::Click, move |_| Msg::FillTreeView {
                                            model_index,
                                            tree_index: vec![],
                                            search: SearchString::EmptySearch,
                                        })
                                    ]
                                ],
                            ],
                            if let Some(treeview) = model.treeviews.get(model_index) {
                                ul![treeview
                                    .root
                                    .children(&treeview.tree)
                                    .filter(|node| !node.is_removed(&treeview.tree))
                                    .take(treeview.current_window)
                                    .enumerate()
                                    .map(|(i, tree)| view_tree_lvl1(
                                        treeview,
                                        tree,
                                        model_index,
                                        i
                                    )),]
                            } else {
                                li![]
                            }
                        ]
                    ]
                ]
            ]
        })
        .collect()
    //} else {
    //    div![]
    //}
}
*/
