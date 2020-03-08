import React from 'react'
import Typography from '@material-ui/core/Typography';
import { makeStyles } from '@material-ui/core/styles';
import { VariableSizeGrid as VSGrid } from 'react-window';
import Box from '@material-ui/core/Box';
import Button from '@material-ui/core/Button';
import DeleteIcon from '@material-ui/icons/Delete';
import IconButton from '@material-ui/core/IconButton';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import PropTypes from 'prop-types';
import axios from 'axios';

function convertSecondsToTime(seconds) {
    let date = new Date(0);
    date.setSeconds(seconds);
    return date.toISOString().substr(14, 5);
}

function columnWidths(index) {
    switch (index) {
        case 0: return 50; //number
        case 1: return 50; //tracknumber
        case 2: return 400; //title
        case 3: return 300; //artist
        case 4: return 300; //album
        case 5: return 200; //genre
        case 6: return 100; //year
        case 7: return 100; //time
        default: return 10000;
    }
}

class Cell extends React.PureComponent {
    constructor(props) {
        super(props)
        this.click = this.click.bind(this);
    }
    click() {
        console.log("cliicked");
        let c = {
            "t": "Play",
            c: this.props.rowIndex,
        };
        axios.post("/transport/", c);

        console.log(this.props);
    }

    render() {
        const { item, selected } = this.props.data[this.props.rowIndex];
        //console.log(style);
        let string = "";
        switch (this.props.columnIndex) {
            case 0: string = this.props.rowIndex; break;
            case 1: string = item.tracknumber; break;
            case 2: string = item.title; break;
            case 3: string = item.artist; break;
            case 4: string = item.album; break;
            case 5: string = item.genre; break;
            case 6: {
                if (item.year !== -1) {
                    string = item.year;
                } else {
                    string = "";
                }
                break;
            }
            case 7: string = convertSecondsToTime(item.length); break;
            default: string = "ERROR";
        }
        let style = JSON.parse(JSON.stringify(this.props.style));
        style.textOverflow = "ellipsis";
        style.width = columnWidths(this.props.columnIndex) + "px";
        if (selected) {
            style.color = "#FF0000";
        }
        return <div style={style} onDoubleClick={this.click}><Typography noWrap>{string}</Typography></div>

    }
}

function TabPanel(props) {
    const { children, value, index, ...other } = props;

    return (
        <Typography
            component="div"
            role="tabpanel"
            hidden={value !== index}
            id={`simple-tabpanel-${index}`}
            aria-labelledby={`simple-tab-${index}`}
            {...other}
        >
            {value === index && <Box p={3}>{children}</Box>}
        </Typography>
    );
}

function a11yProps(index) {
    return {
        id: `simple-tab-${index}`,
        'aria-controls': `simple-tabpanel-${index}`,
    };
}

class PlaylistTab extends React.Component {
    constructor(props) {
        super(props);
        this.click = this.bind.click();
    }

    click() {
        axios.delete("/playlisttab/", { "index": this.props.index });
    }

    render() {
        return <div><Tab label={this.props.t} value={this.props.index} key={this.props.index} /> <IconButton aria-label="delete" onClick={this.click}>
            <DeleteIcon fontSize="small" />
        </IconButton></div >
    }
}

export default class SongView extends React.Component {
    constructor(props) {
        super(props)
        this.state = {
            value: 0,
        };
        this.handleChange = this.handleChange.bind(this);
    }

    handleChange(event, newValue) {
        if (newValue !== this.state.value) {
            this.setState({ value: newValue });
            axios.post("/playlisttab/", { "index": newValue });
        }
    }

    render() {
        let items = this.props.pl.map((t) => ({ item: t, selected: false }));
        // sets the correct index to playing. if there is nothing playing, we don't set anything
        if (this.props.current !== -1 && items && this.props.playing) {
            console.log(this.props.current);
            items[this.props.current].selected = true;
        }

        return <div>
            <Tabs value={this.state.value} onChange={this.handleChange} aria-label="simple tabs example">
                {this.props.tabs.map((t, index) => <PlaylistTab key={index} t={t} index={index} />)}
            </Tabs>
            <VSGrid
                itemData={items}
                columnCount={8}
                columnWidth={columnWidths}
                height={650}
                rowCount={this.props.pl.length}
                rowHeight={(index) => { return 25; }}
                width={1600}
            >
                {Cell}
            </VSGrid>
        </div >
    }
}

