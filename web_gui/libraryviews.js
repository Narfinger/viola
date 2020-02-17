
import React from 'react'
import Typography from '@material-ui/core/Typography';
import Box from '@material-ui/core/Box';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import PropTypes from 'prop-types';
import { makeStyles } from '@material-ui/core/styles';
import axios from 'axios';
import MyTreeView from './mytreeview';
import SmartplaylistView from './smartplaylistview';
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
                <SmartplaylistView url="/smartplaylist/" query_for_details={false} />
            </TabPanel>
        </div >
    )
}