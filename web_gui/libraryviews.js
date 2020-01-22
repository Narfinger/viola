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
        this.state = {
            items: [
            ]
        };

        this.handleChange = this.handleChange.bind(this);
        this.need_to_load = this.need_to_load.bind(this);
    }

    need_to_load(ids) {
        if (ids.length === 0) {
            return true;
        } else if (ids.length === 1) {
            console.log(this.state.items[ids[0]].children);
            return (this.state.items[ids[0]].children.length === 0)
        } else if (ids.length === 2) {
            return ((this.state.items[ids[0]].children.length === 0) || (this.state.items[ids[0]].children[ids[1]].length === 0))
        }
    }

    handleChange(event, nodeids) {
        if (nodeids.length !== 0) {
            let ids = nodeids[0].split("-");
            console.log(ids);

            if (this.need_to_load(ids)) {
                console.log("we would load");
                let names = [];
                names.push(this.state.items[ids[0]].name);
                if (ids.length === 2) {
                    names.push(this.state.items[ids[0]].children[ids[1]].name);
                } else {
                    names.push();
                }

                axios.get(this.props.url, {
                    params: {
                        artist: names[0],
                        album: names[1],
                        track: null,
                    }
                }).then((response) => {

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
        axios.get(this.props.url).then((response) => {
            this.setState({
                items: response.data
            });
        });
    }

    title_children(children, index, index2) {
        if (children.length === 0) {
            return <TreeItem nodeId={"l" + index + "-" + index2} key={"l" + index + "-" + index2} label="Loading" />
        } else {
            return children.map((v3, i3) => {
                return <TreeItem nodeId={index + "-" + index2 + "-" + i3} key={index + "-" + index2 + "-" + i3} label={v3.name}>
                </TreeItem>
            })
        }
    }

    album_children(children, index) {
        if (children.length === 0) {
            return <TreeItem nodeId={"l" + index} key={"l" + index} label="Loading" />
        } else {
            return children.map((v2, i2) => {
                return <TreeItem nodeId={index + "-" + i2} key={index + "-" + i2} label={v2.name}>
                    {this.title_children(v2.children, index, i2)}
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
                        return <TreeItem nodeId={String(index)} key={index} label={value.name}>
                            {this.album_children(value.children, index)}
                        </TreeItem>
                    })
                }
            </TreeView >
        </Paper>

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
                <Tab label="Full" {...a11yProps(0)} />
                <Tab label="Album" {...a11yProps(1)} />
                <Tab label="Track" {...a11yProps(2)} />
            </Tabs>
            <TabPanel value={value} index={0}>
                <MyTreeView url="/libraryview/artist/" />
            </TabPanel>
            <TabPanel value={value} index={1}>
                <MyTreeView url="/libraryview/albums/" />
            </TabPanel>
            <TabPanel value={value} index={2}>
                <MyTreeView url="/libraryview/tracks/" />
            </TabPanel>
        </div >
    )
}