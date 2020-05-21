import * as React from "react";
import Typography from "@material-ui/core/Typography";
import Box from "@material-ui/core/Box";
import Paper from "@material-ui/core/Paper";
import Tabs from "@material-ui/core/Tabs";
import Tab from "@material-ui/core/Tab";
import * as PropTypes from "prop-types";
import { makeStyles, withStyles } from "@material-ui/core/styles";
import MyTreeView from "./MyTreeView";
import SmartplaylistView from "./SmartplaylistView";
import { render } from "react-dom";

function TabPanel(props): JSX.Element {
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
      {value === index && (
        <Box p={3} width={30}>
          {children}
        </Box>
      )}
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
    "aria-controls": `vertical-tabpanel-${index}`,
  };
}

const styles = theme => ({
  root: {
    flexGrow: 1,
    backgroundColor: theme.palette.background.paper,
    display: 'flex',
    height: 500,
    width: 500,
  },
  tabs: {
    borderRight: `1px solid ${theme.palette.divider}`,
    width: 20,
  },
});

type LibraryViewProps = {
  close_fn: () => void;
  classes: any;
}

type LibraryViewState = {
  value: any;
}

class LibraryView extends React.Component<LibraryViewProps, LibraryViewState> {
  constructor(props) {
    super(props);
    this.handleChange = this.handleChange.bind(this);
    this.state = {
      value: 0,
    }
  }

  handleChange(event, newValue) {
    this.setState({ value: newValue });
  }

  render() {
    const { classes } = this.props;
    return (
      <div className={classes.root} >
        <Tabs
          orientation="vertical"
          variant="scrollable"
          value={this.state.value}
          onChange={this.handleChange}
        >
          <Tab label="SMP" {...a11yProps(0)} />
          <Tab label="Full" {...a11yProps(1)} />
          <Tab label="Album" {...a11yProps(2)} />
          <Tab label="Track" {...a11yProps(3)} />
        </Tabs>
        <TabPanel value={this.state.value} index={0}>
          <SmartplaylistView url="/smartplaylist/" close_fn={this.props.close_fn} />
        </TabPanel>
        <TabPanel value={this.state.value} index={1}>
          <MyTreeView close_fn={this.props.close_fn}
            url="/libraryview/partial/"
            query_params_list={["Artist", "Album", "Track"]}
          />
        </TabPanel>
        <TabPanel value={this.state.value} index={2}>
          <MyTreeView
            close_fn={this.props.close_fn}
            url="/libraryview/partial/"
            query_params_list={["Album", "Track"]}
          />
        </TabPanel>
        <TabPanel value={this.state.value} index={3}>
          <MyTreeView close_fn={this.props.close_fn} url="/libraryview/partial/" query_params_list={["Track"]} />
        </TabPanel>
      </div>
    );
  }
}

export default withStyles(styles)(LibraryView);
