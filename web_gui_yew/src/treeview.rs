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

enum SearchString {
    UseStoredSearch,
}

#[derive(Properties, PartialEq)]
struct TreeViewLvl1Props {
    tree: indextree::Arena<String>,
    root: indextree::NodeId,
    nodeid: indextree::NodeId,
    model_index: usize,
    index: usize,
}

enum Msg {
    FillTreeView {
        model_index: usize,
        tree_index: Vec<usize>,
        search: SearchString,
    },
    LoadFromTreeView {
        model_index: usize,
        tree_index: Vec<usize>,
    },
}

struct TreeViewLvl1 {}

impl Component for TreeViewLvl1 {
    type Message = Msg;

    type Properties = TreeViewLvl1Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        //let no_children = ctx.props().nodeid.children(&ctx.props().tree).count() == 0;
        let model_index = ctx.props().model_index;
        let tree_index = vec![ctx.props().index];
        let tree_indexc = vec![ctx.props().index];

        html! {
            <li>
                <span style="list-style-type: disclosure-closed">
                    <span onclick={ctx.link().callback(move |_|
                        Msg::FillTreeView{
                            model_index,
                            tree_index: tree_index.clone(),
                            search: SearchString::UseStoredSearch}
                        )}>
                            {ctx.props().tree.get(ctx.props().nodeid).unwrap().get()}
                    </span>
                    <button
                        class="btn btn-outline-primary btn-sm" style="margin-left: 25px"
                        onclick={ctx.link().callback(move |_|
                            Msg::LoadFromTreeView{
                                tree_index: tree_indexc.clone(),
                                model_index})}>
                        {"Load"}
                    </button>
                </span>
            //<TreeViewLvl2 />
            </li>
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
