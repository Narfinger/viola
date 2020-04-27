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

type Node = {
  children?: Node[];
  value?: string;
};

type Tree = Node[];

type MyTreeViewProps = {
  query_for_details: boolean;
  query_params_list: string[];
  url: string;
};

type MyTreeViewState = {
  items: Tree;
  search: string;
  menu_open: boolean;
};
export default class MyTreeView extends React.Component<
  MyTreeViewProps,
  MyTreeViewState
> {
  refreshDebounced: any;

  public static defaultProps = {
    query_for_details: true,
  };

  constructor(props) {
    super(props);

    this.state = {
      items: [],
      search: "",
      menu_open: false,
    };

    this.refresh = this.refresh.bind(this);
    this.handleChange = this.handleChange.bind(this);
    this.need_to_load = this.need_to_load.bind(this);
    this.handleDoubleClick = this.handleDoubleClick.bind(this);
    this.refreshDebounced = AwesomeDebouncePromise(this.refresh, 500);
    this.searchChange = this.searchChange.bind(this);
  }

  searchChange(e: React.ChangeEvent<HTMLInputElement>) {
    this.setState({ search: e.target.value });
    this.refreshDebounced();
  }

  need_to_load(ids: number[]) {
    if (!this.props.query_for_details) {
      return false;
    } else if (ids.length === 0) {
      return true;
    } else if (ids.length === 1) {
      console.log(this.state.items[ids[0]].children);
      return this.state.items[ids[0]].children.length === 0;
    } else if (ids.length === 2) {
      return (
        this.state.items[ids[0]].children.length === 0 ||
        this.state.items[ids[0]].children[ids[1]].children.length === 0
      );
    }
  }

  handleChange(event, nodeids: string[]) {
    if (this.props.query_for_details && nodeids.length !== 0) {
      const ids = nodeids[0].split("-").map(parseInt);

      if (this.need_to_load(ids)) {
        let state = null;
        // format we look at is
        // {"type": "album", "content": "foo"};

        const names = [];
        names.push(this.state.items[ids[0]].value);
        if (ids.length === 2) {
          names.push(this.state.items[ids[0]].children[ids[1]].value);
        } else {
          names.push();
        }

        state = this.props.query_params_list[ids.length];
        const queryParam = { lvl: { type: state, content: names }, search: "" };

        axios.post(this.props.url, queryParam).then((response) => {
          if (ids.length === 1) {
            const newObject = {
              value: names[0],
              children: response.data[0].children,
            };
            this.setState({
              items: this.state.items.map((obj, index) => {
                return ids[0] === index ? newObject : obj;
              }),
            });
          } else if (ids.length === 2) {
            const newObject = {
              value: names[1],
              children: response.data[0].children[0].children,
            };
            this.setState({
              items: this.state.items.map((obj, index) => {
                if (ids[0] !== index) {
                  return obj;
                } else {
                  return {
                    value: names[0],
                    children: obj.children.map((objv2, indexv2) => {
                      return ids[1] === indexv2 ? newObject : objv2;
                    }),
                  };
                }
              }),
            });
          }
        });
      }
    }
  }

  componentDidMount() {
    this.refresh();
  }

  refresh() {
    const queryParam = {
      search: this.state.search,
      lvl: { type: this.props.query_params_list[0], content: [] },
    };
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

  handleDoubleClick(event: React.MouseEvent, index: string) {
    const ids = index.split("-");
    const values = [];
    let current = this.state.items;
    for (const id of ids) {
      const val = current[parseInt(id, 10)];
      values.push(val.value);
      current = val.children;
    }
    const type = this.props.query_params_list[
      Math.min(ids.length, this.props.query_params_list.length - 1)
    ];
    const param = {
      search: this.state.search,
      lvl: { type: type, content: values },
    };
    axios.post("/libraryview/load/", param);
  }

  third_level_children(children: Node[], index: number, index2: number) {
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
            onDoubleClick={(e) => this.handleDoubleClick(e, i)}
          />
        );
      });
    }
  }

  second_level_children(index: number, children?: Node[]) {
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
            onDoubleClick={(e) => this.handleDoubleClick(e, i)}
          >
            {this.third_level_children(v2.children, index, i2)}
          </TreeItem>
        );
      });
    }
  }

  render() {
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
          {this.state.items.map((value, index) => {
            const i = String(index);
            let menu: LibraryMenu = new LibraryMenu({});
            return (
              <TreeItem
                nodeId={i}
                key={i}
                label={value.value}
                onDoubleClick={(e) => this.handleDoubleClick(e, i)}
              >
                {this.second_level_children(index, value.children)}
              </TreeItem>
            );
          })}
        </TreeView>
      </Paper>
    );
  }
}
