import ReactDOM from 'react-dom'
import React from 'react'
import Paper from '@material-ui/core/Paper';
import Button from '@material-ui/core/Button';
import Grid from '@material-ui/core/Grid';
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';
import ChevronRightIcon from '@material-ui/icons/ChevronRight';
import TreeView from '@material-ui/lab/TreeView';
import TreeItem from '@material-ui/lab/TreeItem';
import Typography from '@material-ui/core/Typography';
import Box from '@material-ui/core/Box';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import PropTypes from 'prop-types';
import { makeStyles } from '@material-ui/core/styles';
import axios from 'axios';
const e = React.createElement;


function TabPanel(props) {
    const { children, value, index, ...other } = props;

    return (
        <Typography
            component="div"
            role="tabpanel"
            hidden={value !== index}
            id={`vertical-tabpanel-${index}`}
            aria-labelledby={`vertical-tab-${index}`}
            {...other}
        >
            {value === index && <Box p={3} width={30}>{children}</Box>}
        </Typography>
    );
}

TabPanel.propTypes = {
    children: PropTypes.node,
    index: PropTypes.any.isRequired,
    value: PropTypes.any.isRequired,
};

function a11yProps(index) {
    return {
        id: `vertical-tab-${index}`,
        'aria-controls': `vertical-tabpanel-${index}`,
    };
}

const useStyles = makeStyles(theme => ({
    root: {
        //flexGrow: 1,
        backgroundColor: theme.palette.background.paper,
        display: 'flex',
        height: 500,
        width: 800,
    },
    tabs: {
        borderRight: `1px solid ${theme.palette.divider}`,
        width: 100,
    },
}));

class MyTreeView extends React.Component {
    constructor(props) {
        super(props)

        let list = ["Artist", "Album", "Track"];
        if (props.query_params == "Album") {
            list.pop();
        } else if (props.query_params == "Track") {
            list.pop();
            list.pop();
        }

        this.state = {
            query_params_list: list,
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

            console.log("ids");
            console.log(nodeids);
            console.log(ids);
            console.log(ids.length);
            console.log(this.need_to_load(ids));

            if (this.need_to_load(ids)) {
                console.log("we would load");
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
                //console.log(this.state.query_params_list);
                //console.log(ids);
                state = this.state.query_params_list[ids.length];
                let query_param = {"type": state, "content": names};
                //console.log("We could query the following");
                //console.log(query_param);

                axios.post(this.props.url, query_param).then((response) => {
                    console.log(response.data[0].children);
                    if (ids.length === 1) {
                        let new_object = { value: names[0], children: response.data[0].children };
                        //console.log(new_object);
                        this.setState({
                            items: this.state.items.map((obj, index) => {
                                return ids[0] == index ? new_object : obj;
                            })
                        })
                    } else if (ids.length === 2) {
                        console.log("we found 2 ids");
                        let new_object = {value: names[1], children: response.data[0].children[0].children };
                        this.setState({
                            items: this.state.items.map((obj, index) => {
                                if (ids[0] != index) {
                                    return obj;
                                } else {
                                    console.log("the object stuff");
                                    console.log(new_object);
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
        console.log("we mounted treeview " + this.props.url);
        console.log(this.props.query_param);
        axios.post(this.props.url,
            { "type": this.state.query_params_list[0]}).then((response) => {
                this.setState({
                    items: response.data
                });
            });
    }

    handleDoubleClick(name, level, event) {
        console.log("doing event");
        block events
        console.log(name);
        console.log(level);
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
                return <TreeItem nodeId={index + "-" + index2 + "-" + i3} key={index + "-" + index2 + "-" + i3} label={label} onDoubleClick={(e) => this.handleDoubleClick(v3.value, 2, e)} />
            })
        }
    }

    second_level_children(children, index) {
        if (children.length === 0) {
            if (this.props.query_for_details) {
                return <TreeItem nodeId={"l" + index} key={"l" + index} label="Loading" />
            }
        } else {
            return children.map((v2, i2) => {
                let value = null;
                if (v2.optional) {
                    value = v2.optional + "-" + v2.value;
                } else {
                    value = v2.value;
                };
                return <TreeItem nodeId={index + "-" + i2} key={index + "-" + i2} label={value} onDoubleClick={(e) => this.handleDoubleClick(v2.value, 1, e)} >
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
                        return <TreeItem nodeId={String(index)} key={index} label={value.value} onDoubleClick={(e) => this.handleDoubleClick(value.value, 0, e)} >
                            {this.second_level_children(value.children, index)}
                        </TreeItem>
                    })
                }
            </TreeView >
        </Paper >

    }
}

export default function LibraryView() {
    const classes = useStyles();
    const [value, setValue] = React.useState(0);

    const handleChange = (event, newValue) => {
        setValue(newValue);
    };

    //try to make the size dynamic
    return (
        <div className={classes.root} >
            <Tabs
                orientation="vertical"
                variant="scrollable"
                value={value}
                onChange={handleChange}
                className={classes.tabs}
            >
                <Tab label="Full"  {...a11yProps(0)} />
                <Tab label="Album" {...a11yProps(1)} />
                <Tab label="Track" {...a11yProps(2)} />
                <Tab label="SMP" {...a11yProps(3)} />
            </Tabs>
            <TabPanel value={value} index={0}>
                <MyTreeView url="/libraryview/partial/" query_for_details={true} query_param="Artist" />
            </TabPanel>
            <TabPanel value={value} index={1}>
                <MyTreeView url="/libraryview/partial/" query_for_details={true} query_param="Album" />
            </TabPanel>
            <TabPanel value={value} index={2}>
                <MyTreeView url="/libraryview/partial/" query_for_details={true} query_param="Track" />
            </TabPanel>
            <TabPanel value={value} index={3}>
                <MyTreeView url="/smartplaylist/" query_for_details={false} />
            </TabPanel>
        </div >
    )
}