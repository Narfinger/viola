use indextree::{Arena, NodeId};
use reqwasm::http::Request;
use viola_common::*;
use yew::prelude::*;

pub(crate) enum SearchString {
    UseStoredSearch,
    EmptySearch,
}

#[derive(Properties, PartialEq)]
pub(crate) struct TreeViewLvl1Props {
    pub(crate) type_vec: Vec<viola_common::TreeType>,
    pub(crate) close_callback: Callback<()>,
}

pub(crate) enum TreeViewLvl1Msg {
    FillTreeView {
        search: SearchString,
        indices: Vec<usize>,
    },
    FillTreeViewRecv {
        result: Vec<String>,
        query: TreeViewQuery,
    },
    LoadFromTreeView(Vec<usize>),
    Done,
}

pub(crate) struct TreeViewLvl1 {
    tree: indextree::Arena<String>,
    root: indextree::NodeId,
}

impl TreeViewLvl1 {
    fn tree_index_to_nodeid(&self, tree_index: &[usize]) -> Option<NodeId> {
        match tree_index.len() {
            0 => Some(self.root),
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

    fn view_lvl3(&self, ctx: &Context<Self>, index: usize, index2: usize, nodeid2: NodeId) -> Html {
        if nodeid2.children(&self.tree).count() == 0 {
            html! { <> </> }
        } else {
            html! {
                    <ul>
                    {
                        nodeid2.children(&self.tree).enumerate().map(|(index3, nodeid3)| {
                            html! {
                                <li>
                                <span style="list-style-type: disclosure-closed">
                                    <span onclick={ctx.link().callback(move |_| TreeViewLvl1Msg::FillTreeView{
                                        indices: vec![index,index2, index3],
                                        search: SearchString::EmptySearch,
                                    })}>
                                    {self.tree.get(nodeid3).unwrap().get()}
                                    </span>
                                    <button
                                    class="btn btn-outline-primary btn-sm" style="margin-left: 25px"
                                    onclick={ctx.link().callback(move |_| TreeViewLvl1Msg::LoadFromTreeView(vec![index,index2,index3]))}>
                                {"Load"}
                                </button>
                                </span>
                                </li>
                            }
                        }).collect::<Html>()
                    }
                    </ul>
            }
        }
    }

    fn view_lvl2(&self, ctx: &Context<Self>, index: usize, nodeid: NodeId) -> Html {
        if nodeid.children(&self.tree).count() == 0 {
            html! {<> </>}
        } else {
            html! {
                <ul>
                {
                    nodeid.children(&self.tree).enumerate().map(|(index2, nodeid2)| {
                        let lvl3 = self.view_lvl3(&ctx, index, index2, nodeid2);
                        html! {
                            <li>
                            <span style="list-style-type: disclosure-closed">
                                <span onclick={ctx.link().callback(move |_| TreeViewLvl1Msg::FillTreeView{
                                    indices: vec![index, index2],
                                    search: SearchString::EmptySearch,
                                })}>
                                {self.tree.get(nodeid2).unwrap().get()}
                                </span>
                                <button
                                    class="btn btn-outline-primary btn-sm" style="margin-left: 25px"
                                    onclick={ctx.link().callback(move |_| TreeViewLvl1Msg::LoadFromTreeView(vec![index,index2]))}>
                                {"Load"}
                                </button>
                            </span>
                            {lvl3}
                            </li>
                        }
                    }).collect::<Html>()
                }
                </ul>
            }
        }
    }
}

impl Component for TreeViewLvl1 {
    type Message = TreeViewLvl1Msg;

    type Properties = TreeViewLvl1Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut tree = Arena::new();
        let root = tree.new_node("".to_string());
        ctx.link().send_message(TreeViewLvl1Msg::FillTreeView {
            indices: vec![],
            search: SearchString::EmptySearch,
        });
        Self { tree, root }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TreeViewLvl1Msg::FillTreeView { search, indices } => {
                let type_vec = ctx.props().type_vec.clone();

                ctx.link().send_future(async move {
                    let data = viola_common::TreeViewQuery {
                        indices: indices,
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
                    TreeViewLvl1Msg::FillTreeViewRecv {
                        result,
                        query: data,
                    }
                });
                false
            }
            TreeViewLvl1Msg::FillTreeViewRecv { result, query } => {
                let append_to = self.tree_index_to_nodeid(&query.indices).unwrap();
                if append_to.children(&self.tree).count() != 0 {
                    false
                } else {
                    for i in result {
                        let new_node = self.tree.new_node(i);
                        append_to.append(new_node, &mut self.tree);
                    }
                    true
                }
            }
            TreeViewLvl1Msg::LoadFromTreeView(indices) => {
                let data = viola_common::TreeViewQuery {
                    indices,
                    types: ctx.props().type_vec.clone(),
                    search: None,
                };
                ctx.link().send_future(async move {
                    let result: Vec<String> = Request::post("/libraryview/full/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&data).unwrap())
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    TreeViewLvl1Msg::Done
                });
                false
            }
            TreeViewLvl1Msg::Done => {
                ctx.props().close_callback.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let nodes = self
            .root
            .children(&self.tree)
            .enumerate()
            .map(|(index, nodeid)| {
                html! {
                    <li>
                        <span style="list-style-type: disclosure-closed">
                            <span onclick={ctx.link().callback(move |_|
                                TreeViewLvl1Msg::FillTreeView{
                                    indices: vec![index],
                                    search: SearchString::UseStoredSearch}
                                )}>
                                    {self.tree.get(nodeid).unwrap().get()}
                            </span>
                            <button
                                class="btn btn-outline-primary btn-sm" style="margin-left: 25px"
                                onclick={ctx.link().callback(move |_| TreeViewLvl1Msg::LoadFromTreeView(vec![index]))}>
                                {"Load"}
                            </button>
                        </span>
                        {self.view_lvl2(&ctx, index, nodeid)}
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
