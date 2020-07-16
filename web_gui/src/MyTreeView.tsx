import * as React from "react";
import Paper from "@material-ui/core/Paper";
import Input from "@material-ui/core/Input";
import ExpandMoreIcon from "@material-ui/icons/ExpandMore";
import ChevronRightIcon from "@material-ui/icons/ChevronRight";
import TreeView from "@material-ui/lab/TreeView";
import TreeItem from "@material-ui/lab/TreeItem";
import AwesomeDebouncePromise from "awesome-debounce-promise";
import Popover from "@material-ui/core/Popover";
import LibraryMenu from "./LibraryMenu";
import axios from "axios";

enum QueryType {
  Artist,
  Album,
  Track,
}

type MyTreeItemNodeProp = {
  index?: string,
  start: QueryType,
}

class MyTreeItemNode {
  children = [];
  id: string = "";
  title: string = "";
  start: QueryType = QueryType.Artist;

  constructor(children, id, start, title) {
    this.children = children;
    this.id = id;
    this.start = start;
    this.title = title;
  }

  populate_children() {
    if (this.children.length === 0) {
      axios.post("/libraryview/partial/", {
        index: this.id == "" ? [] : this.id.split("-"),
        start: this.start,
      }).then((response) => {
        this.children = response.data.map((val, index) => {
          return new MyTreeItemNode([], this.id + "-" + String(index), this.start, val);
        }
        )
      })
    }
  }
}

class MyTreeItemRender extends React.Component<{ mynode: MyTreeItemNode }, {}> {
  render(): JSX.Element {
    return <TreeItem nodeId={this.props.mynode.id} label={this.props.mynode.title} key={this.props.mynode.id}>
      {this.props.mynode.children.map((val) => {
        return <MyTreeItemRender mynode={val} />
      })}
    </TreeItem>
  }
}

type MyTreeViewState = {
  main: MyTreeItemNode;
  search: string;
  menuOpen: boolean;
  menuIndex: string;
  anchor: any;
};

type MyTreeViewProps = {
  close_fn: () => void;
  start: QueryType;
};

export default class MyTreeView extends React.Component<MyTreeViewProps, MyTreeViewState> {
  public static defaultProps = {
    query_for_details: true,
  };

  constructor(props) {
    super(props);
    this.handleChange = this.handleChange.bind(this);
    this.state = {
      main: new MyTreeItemNode([], "", this.props.start, ""),
      search: "",
      menuOpen: false,
      menuIndex: "",
      anchor: null,
    }
  }

  searchChange(): void {

  }

  componentDidMount() {
    let new_main = this.state.main;
    new_main.populate_children();
    console.log(this.state.main);
    //react can't do nested updates because of course it can't
    this.setState({ main: new_main },
      () => {
        console.log("inside");
        console.log(this.state.main);
        console.log(this.state.main.children);
      });
  }

  handleChange(event, nodeids: string[]): void {
    //let cur = this.state.main;
    //for (const i in nodeids) {
    //  cur = cur.children[i];
    //}
    //cur.populate_children();
    //this.setState({ main: this.state.main });
  }

  render(): JSX.Element {
    return (
      <Paper style={{ maxHeight: 800, width: 800, overflow: "auto" }}>
        <form noValidate autoComplete="off">
          <Input defaultValue="" onChange={this.searchChange} />
        </form>
        <LibraryMenu open={this.state.menuOpen} index={this.state.menuIndex} anchor={this.state.anchor} closeFn={() => this.setState({ menuOpen: false })} />
        <TreeView
          defaultCollapseIcon={<ExpandMoreIcon />}
          defaultExpandIcon={<ChevronRightIcon />}
          onNodeToggle={this.handleChange}
        >
          <TreeItem nodeId="test" label="test" key="test">
            {this.state.main.children.map((val) => {
              return <MyTreeItemRender mynode={val} />
            })}
          </TreeItem>
        </TreeView>
      </Paper>
    );
  }
}

/*
export default class MyTreeView extends React.Component<MyTreeViewProps, MyTreeViewState> {
        refreshDebounced: any;

  public static defaultProps = {
        query_for_details: true,
  };

  constructor(props) {
        super(props);

    this.state = {
        items: [],
      search: "",
      menuOpen: false,
      menuIndex: "",
      anchor: null,
    };

    this.refresh = this.refresh.bind(this);
    this.handleChange = this.handleChange.bind(this);
    this.need_to_load = this.need_to_load.bind(this);
    this.handleClick = this.handleClick.bind(this);
    this.handleDoubleClick = this.handleDoubleClick.bind(this);
    this.refreshDebounced = AwesomeDebouncePromise(this.refresh, 500);
    this.searchChange = this.searchChange.bind(this);
  }

  searchChange(e: React.ChangeEvent<HTMLInputElement>): void {
        this.setState({ search: e.target.value });
    this.refreshDebounced();
  }

  need_to_load(ids: number[]): boolean {
    if (!this.props.query_for_details) {
      return false;
    } else if (ids.length === 0) {
      return true;
    } else if (ids.length === 1) {
      //console.log(this.state.items[ids[0]].children);
      return this.state.items[ids[0]].children.length === 0;
    } else if (ids.length === 2) {
      return (
        this.state.items[ids[0]].children.length === 0 ||
        this.state.items[ids[0]].children[ids[1]].children.length === 0
      );
    }
  }

  handleChange(event, nodeids: string[]): void {
    if (this.props.query_for_details && nodeids.length !== 0) {
      const ids = nodeids[0].split("-").map(parseInt);

      if (this.need_to_load(ids)) {
        const queryParam = {search: this.state.search, pql: this.props.query_params_list.slice(ids.length) };
        queryParam.pql[0].query = ids.slice(-1)[0];
        axios.post(this.props.url, queryParam).then((response) => {
          //fill into the tree
        });
      }
    }
  }

  componentDidMount(): void {
          this.refresh();
  }

  refresh(): void {
    const queryParam = {search: this.state.search, pql: [this.props.query_params_list[0]] };
    console.log(queryParam);
    axios.post(this.props.url, queryParam).then((response) => {
          let data = response.data;
      if (this.props.query_params_list.length === 1) {
          data = response.data[0].children[0].children;
      } else if (this.props.query_params_list.length === 2) {
          data = response.data[0].children;
      }
      this.setState({
          items: data,
      });
    });
  }

  handleClick(event: React.MouseEvent, index: string): void {
          this.setState({
            menuOpen: true,
            menuIndex: index,
            anchor: event.target,
          });
    this.props.close_fn();
    event.preventDefault();
  }

  handleDoubleClick(event: React.MouseEvent, index: string): void {
    const ids = index.split("-");
    const values = [];
    let current = this.state.items;
    //fill with queries
    for (const id of ids) {
      const val = current[parseInt(id, 10)];
      values.push(val.value);
      current = val.children;
    }
    console.log(values);
    let q = Object.assign(this.props.query_params_list);


    q.forEach(function (item, index, array) {
          item.query = values[index];
    });

    console.log("load query send");
    console.log(q);

    this.props.close_fn();
    axios.post("/libraryview/load/", q);
  }

  third_level_children(
    children: Node[],
    index: number,
    index2: number
  ): JSX.Element | JSX.Element[] {
    if (children.length === 0) {
      if (this.props.query_for_details) {
        const new_index = "l" + index + "-" + index2;
        return <TreeItem nodeId={new_index} key={new_index} label="Loading" />;
      }
    } else {
      return children.map((v3, i3) => {
        const label = "" + v3.value;
        const i = index + "-" + index2 + "-" + i3;
        return (
          <TreeItem
          nodeId={i}
          key={i}
          label={label}
          onContextMenu={(e): void => this.handleClick(e, i)}
          onDoubleClick={(e): void => this.handleDoubleClick(e, i)}
        />
        );
      });
    }
  }

  second_level_children(
    index: number,
    children?: Node[]
  ): JSX.Element | JSX.Element[] {
    if ((!children || children.length === 0) && this.props.query_for_details) {
      return (
        <TreeItem nodeId={"l" + index} key={"l" + index} label="Loading" />
      );
    } else if (!children || children.length === 0) {
      return;
    } else {
      return children.map((v2, i2) => {
        const value = v2.value;
        const i = index + "-" + i2;
        return (
          <TreeItem
          nodeId={i}
          key={i}
          label={value}
          onContextMenu={(e): void => this.handleClick(e, i)}
          onDoubleClick={(e): void => this.handleDoubleClick(e, i)}
        >
          {this.third_level_children(v2.children, index, i2)}
        </TreeItem>
        );
      });
    }
  }

  render(): JSX.Element {
    return (
      <Paper style={{ maxHeight: 800, width: 800, overflow: "auto" }}>
          <form noValidate autoComplete="off">
            <Input defaultValue="" onChange={this.searchChange} />
          </form>
          <TreeView
            defaultCollapseIcon={<ExpandMoreIcon />}
            defaultExpandIcon={<ChevronRightIcon />}
            onNodeToggle={this.handleChange}
          >
            <LibraryMenu open={this.state.menuOpen} index={this.state.menuIndex} anchor={this.state.anchor} closeFn={() => this.setState({ menuOpen: false })} />
            {this.state.items.map((value, index) => {
              const i = String(index);
              return <TreeItem
                nodeId={i}
                key={i}
                label={value.value}
                onContextMenu={(e): void => this.handleClick(e, i)}
                onDoubleClick={(e): void => this.handleDoubleClick(e, i)}
              >
                {this.second_level_children(index, value.children)}
              </TreeItem>
            })}
          </TreeView>
        </Paper>
    );
  }
}
*/
