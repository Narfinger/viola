
import ReactDOM from 'react-dom'
import React from 'react'
import Button from '@material-ui/core/Button';
import Grid from '@material-ui/core/Grid';
import { VariableSizeGrid as VSGrid } from 'react-window';
import axios from 'axios';
const e = React.createElement;

var PlayState = {
    Stopped: 1,
    Paused: 2,
    Playing: 3
};


class TransportButton extends React.Component {
    constructor(props) {
        super(props)
        this.state = { title: "", url: "" }
    }
    click() {
        axios.get("/transport/{this.state.url}");
    }
    render() {
        return <Button ariant="contained" color="primary" onClick={this.click}> {this.props.title}</Button>
    }
}

function PlayButton(props) {
    if (props.play_state === PlayState.Stopped) {
        return <TransportButton title="Play" api="play"></TransportButton>
    };
    if (props.play_state === PlayState.Paused) {
        return <TransportButton title="Play" api="play"></TransportButton>
    };
    if (props.play_state === PlayState.Playing) {
        return <TransportButton title="Pause" api="pause"></TransportButton>
    }
    return <TransportButton title="Unspecified" api="NI"></TransportButton>
}

function Main() {
    return <div className={classes.root}>
        <Grid container spacing={1}>
            <Grid item xs={3}>
                <TransportButton title="Prev" api="prev"></TransportButton>
            </Grid>
            <Grid item xs={3}>
                <PlayButton></PlayButton>
            </Grid>
            <Grid item xs={12}>
                <TransportButton title="Next" api="next"></TransportButton>
            </Grid>
            <Grid item xs={12}>
                <SongView></SongView>
            </Grid>
        </Grid>
    </div>
};


function columnWidths(index) {
    switch (index) {
        case 0: return 50;  //number
        case 1: return 400; //title
        case 2: return 300; //artist
        case 3: return 300; //album
        case 4: return 200; //genre
        default: return 10000;
    }
}

class Cell extends React.PureComponent {
    render() {
        const item = this.props.data[this.props.rowIndex];
        switch (this.props.columnIndex) {
            case 0: return <div style={this.props.style}>{this.props.rowIndex}</div>
            case 1: return <div style={this.props.style}>{item.title}</div>
            case 2: return <div style={this.props.style}>{item.artist}</div>
            case 3: return <div style={this.props.style}>{item.album}</div>
            case 4: return <div style={this.props.style}>{item.genre}</div>
            default: return <div style={this.props.style}>ERROR</div>
        }
    }
}

class SongView extends React.Component {
    constructor(props) {
        super(props);
        this.state = { pl: [] };
    }

    componentDidMount() {
        console.log("we mounted");
        axios.get("/playlist/").then((response) => this.setState({
            pl: response.data,
        }, console.log(this.state.pl))).catch(function () { });
    }

    render() {
        return <div><div>
            <VSGrid
                itemData={this.state.pl}
                columnCount={5}
                columnWidth={columnWidths}
                height={700}
                rowCount={this.state.pl.length}
                rowHeight={(index) => { return 25; }}
                width={1500}
            >
                {Cell}
            </VSGrid>
        </div></div>
    }
}



const domContainer = document.querySelector('#main_container');
ReactDOM.render(e(Main), domContainer);