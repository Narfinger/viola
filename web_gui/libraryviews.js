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
    }

    handleChange(event, nodeids) {
        if (nodeids.length !== 0) {
            let id = nodeids[0];
            let node = this.state.items[id];
            if (node.children.length !== 1) {
                axios.get(this.props.detailurl1 + encodeURI(node.name)).then((response) => {
                    let new_object = { name: node.name, children: response.data };
                    this.setState({
                        items: this.state.items.map((obj, index) => {
                            return id == index ? new_object : obj;
                        })
                    });
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

    children(children, index) {
        if (children.length === 0) {
            return <TreeItem nodeId={"l" + index} key={"l" + index} label="Loading" />
        } else {
            return children.map((v2, i2) => {
                return <TreeItem nodeId={index + "-" + i2} key={index + "-" + i2} label={v2.name} />
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
                            {this.children(value.children, index)}
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
                <MyTreeView url="/libraryview/artist/" detailurl1="/libraryview/albums/" />
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