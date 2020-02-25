import React from 'react';
import Paper from '@material-ui/core/Paper';
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';
import ChevronRightIcon from '@material-ui/icons/ChevronRight';
import TreeView from '@material-ui/lab/TreeView';
import TreeItem from '@material-ui/lab/TreeItem';
import axios from 'axios';


export default class MyTreeView extends React.Component {
    constructor(props) {
        super(props)

        this.state = {
            items: [
            ]
        };

        this.handleChange = this.handleChange.bind(this);
        this.need_to_load = this.need_to_load.bind(this);
        this.handleDoubleClick = this.handleDoubleClick.bind(this);
    }

    need_to_load(ids) {
        if (!this.props.query_for_details) {
            return false;
        } else if (ids.length === 0) {
            return true;
        } else if (ids.length === 1) {
            console.log(this.state.items[ids[0]].children);
            return (this.state.items[ids[0]].children.length === 0)
        } else if (ids.length === 2) {
            return ((this.state.items[ids[0]].children.length === 0) || (this.state.items[ids[0]].children[ids[1]].children.length === 0))
        }
    }

    handleChange(event, nodeids) {
        if (this.props.query_for_details && nodeids.length !== 0) {
            let ids = nodeids[0].split("-");

            if (this.need_to_load(ids)) {
                let state = null;
                // format we look at is
                // {"type": "album", "content": "foo"};


                let names = [];
                names.push(this.state.items[ids[0]].value);
                if (ids.length === 2) {
                    names.push(this.state.items[ids[0]].children[ids[1]].value);
                } else {
                    names.push();
                }
                //console.log(this.props.query_params_list);
                //console.log(ids);

                state = this.props.query_params_list[ids.length];
                let query_param = {"lvl": {"type": state, "content": names}};
                //console.log("We could query the following");
                //console.log(query_param);

                axios.post(this.props.url, query_param).then((response) => {
                    if (ids.length === 1) {
                        let new_object = { value: names[0], children: response.data[0].children };
                        //console.log(new_object);
                        this.setState({
                            items: this.state.items.map((obj, index) => {
                                return ids[0] == index ? new_object : obj;
                            })
                        })
                    } else if (ids.length === 2) {
                        let new_object = {value: names[1], children: response.data[0].children[0].children };
                        this.setState({
                            items: this.state.items.map((obj, index) => {
                                if (ids[0] != index) {
                                    return obj;
                                } else {
                                    let nb = { value: names[0], children: obj.children.map((objv2, indexv2) => {
                                        return ids[1] == indexv2 ? new_object: objv2;
                                    })};
                                    return nb;
                                }
                            })
                        });
                    }

                    //let new_object = { name: node.name, children: response.data };
                    //this.setState({
                    //    items: this.state.items.map((obj, index) => {
                    //        return ids[0] == index ? new_object : obj;
                    //    })
                    //});
                })
            }
        }
    }

    componentDidMount() {
        let query_param = {};
        if (this.props.query_params_list[0]!="Artist") {
            query_param = {"lvl": {"type": this.props.query_params_list[0],
        "content":[]}};
        } else {
            query_param = {"lvl": {"type": this.props.query_params_list[0]}};
        }
        axios.post(this.props.url,query_param).then((response) => {
            let data = response.data;
            if (this.props.query_params_list.length == 1) {
                data = response.data[0].children[0].children;
            } else if (this.props.query_params_list.length == 2) {
                data = response.data[0].children;
            }
            this.setState({
                items: data,
            });
        });
    }

    handleDoubleClick(event, index) {
        event.stopPropagation();
        console.log("doing event");
        let ids = index.split("-");
        let values = [];
        let current = this.state.items;
        for(let id of ids) {
            let val = current[parseInt(id)];
            values.push(val.value);
            current = val.children;
        }
        console.log(values);
        let type = this.props.query_params_list[Math.min(ids.length,this.props.query_params_list.length-1)];
        console.log(type);
        axios.post("/libraryview/load/", {"lvl": {
            "type": type,
            "content": values,
        }});
        /*
        console.log(name);
        console.log(level);
        let type = this.props.query_params_list[level+1];
        console.log(type);
        axios.post("/libraryview/load/", {
            "type": type,
            "content": [name],
        });
        */
    }

    third_level_children(children, index, index2) {
        if (children.length === 0) {
            if (this.props.query_for_details) {
                return <TreeItem nodeId={"l" + index + "-" + index2} key={"l" + index + "-" + index2} label="Loading" />
            }
        } else {

            return children.map((v3, i3) => {
                let label = "";
                if (v3.optional) {
                    label += v3.optional + "-";
                }
                label += v3.value;
                let i = index + "-" + index2 + "-" + i3;
                return <TreeItem nodeId={i} key={i} label={label} onDoubleClick={(e) => this.handleDoubleClick(e, i)} />
            })
        }
    }

    second_level_children(children, index) {
        if ((!children || children.length == 0) && this.props.query_for_details) {
                return <TreeItem nodeId={"l" + index} key={"l" + index} label="Loading" />
        } else if (!children || children.length == 0) {
            return
        } else {
            return children.map((v2, i2) => {
                let value = null;
                if (v2.optional) {
                    value = v2.optional + "-" + v2.value;
                } else {
                    value = v2.value;
                };
                let i = index + "-" + i2;
                return <TreeItem nodeId={i} key={i} label={value} onDoubleClick={(e) => this.handleDoubleClick(e,i)} >
                    {this.third_level_children(v2.children, index, i2)}
                </TreeItem>
            })
        }
    }

    render() {
        return <Paper style={{ maxHeight: 800, width: 800, overflow: 'auto' }}>
            <TreeView height="60vh"
                defaultCollapseIcon={<ExpandMoreIcon />}
                defaultExpandIcon={<ChevronRightIcon />}
                onNodeToggle={this.handleChange}
            >
                {
                    this.state.items.map((value, index) => {
                        let i = String(index);
                        return <TreeItem nodeId={i} key={i} label={value.value} onDoubleClick={(e) => this.handleDoubleClick(e, i)} >
                            {this.second_level_children(value.children, index)}
                        </TreeItem>
                    })
                }
            </TreeView >
        </Paper >

    }
}

MyTreeView.defaultProps = {
    query_for_details: true,
  };