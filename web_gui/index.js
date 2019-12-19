
import ReactDOM from 'react-dom'
import React from 'react'
import Button from '@material-ui/core/Button';
import Grid from '@material-ui/core/Grid';
import { VariableSizeGrid as VSGrid } from 'react-window';
import axios from 'axios';
const e = React.createElement;

const PlayState = Object.freeze({
    Stopped: 1,
    Paused: 2,
    Playing: 3
});

const ButtonEvent = Object.freeze({
    Next: 1,
    Previous: 2,
    Pause: 3,
    Play: 4,
});

class TransportButton extends React.Component {
    constructor(props) {
        super(props);

        // This binding is necessary to make `this` work in the callback
        this.click = this.click.bind(this);
    }
    click() {
        this.props.click(this.props.event);
    }
    render() {
        return <Button variant="contained" color="primary" onClick={this.click}> {this.props.title}</Button>
    }
}

function PlayButton(props) {
    if (props.play_state === PlayState.Stopped) {
        return <TransportButton title="Play" event={ButtonEvent.Play} click={props.click}></TransportButton>
    };
    if (props.play_state === PlayState.Paused) {
        return <TransportButton title="Play" event={ButtonEvent.Play} click={props.click}></TransportButton>
    };
    if (props.play_state === PlayState.Playing) {
        return <TransportButton title="Pause" event={ButtonEvent.Pause} click={props.click}></TransportButton>
    }
    return <TransportButton title="Unspecified"></TransportButton>
}

class Main extends React.Component {
    constructor(props) {
        super(props)
        this.state = {
            status: PlayState.Stopped,
            current: -1,
            pl: [],
        };

        this.handleButtonPush = this.handleButtonPush.bind(this);
        this.refresh = this.refresh.bind(this);
        this.ws = new WebSocket("ws://" + window.location.hostname + ":" + window.location.port + "/ws/")
    }


    componentDidMount() {
        console.log("we mounted");
        axios.get("/playlist/").then((response) => this.setState({
            pl: response.data
        }));

        this.ws.onopen = () => {
            // on connecting, do nothing but log it to the console
            console.log('connected')
        }

        this.ws.onmessage = evt => {
            var msg = JSON.parse(evt.data);
            console.log(msg);
            switch (msg.type) {
                case "Ping": break;
                case "PlayChanged": this.setState({ current: msg.index }); break;
                default:
            }
        }

        this.ws.onclose = () => {
            console.log('disconnected')
            // automatically try to reconnect on connection loss

        }
    }

    handleButtonPush(e) {
        if (e === ButtonEvent.Play) {
            axios.get("/transport/play");
            this.setState({ status: PlayState.Playing });
        } else if (e === ButtonEvent.Pause) {
            axios.get("/transport/pause");
            this.setState({ status: PlayState.Paused });
        } else if (e === ButtonEvent.Previous) {
            axios.get("/transport/prev");
            console.log("previous");
        } else if (e === ButtonEvent.Next) {
            axios.get("/transport/next");
            console.log("next");
        } else {
            console.log("Unspecified!");
        }
    }

    refresh() {
        axios.get("/currentid/").then((response) => {
            this.setState({ current: response.data });
        })
    }

    render() {
        return <div>
            <Grid container spacing={1}>
                <Grid item xs={3}>
                    <TransportButton title="Prev" event="ButtonEvent.Previous" click={this.handleButtonPush}></TransportButton>
                </Grid>
                <Grid item xs={3}>
                    <PlayButton play_state={this.state.status} click={this.handleButtonPush}></PlayButton>
                </Grid>
                <Grid item xs={3}>
                    <TransportButton title="Next" api="next" event="ButtonEvent.Next" click={this.handleButtonPush}></TransportButton>
                </Grid>
                <Grid item xs={12}>
                    <SongView current={this.state.current} pl={this.state.pl}></SongView>
                </Grid>
            </Grid>
        </div >
    }
}

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
        const { item, selected } = this.props.data[this.props.rowIndex];
        if (selected) {
            let style = JSON.parse(JSON.stringify(this.props.style));
            style.color = "#FF0000";
            console.log(style);
            switch (this.props.columnIndex) {
                case 0: return <div style={style}>{this.props.rowIndex}</div>
                case 1: return <div style={style}>{item.title}</div>
                case 2: return <div style={style}>{item.artist}</div>
                case 3: return <div style={style}>{item.album}</div>
                case 4: return <div style={style}>{item.genre}</div>
                default: return <div style={this.props.style}>ERROR</div>
            }
        } else {
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
}

class SongView extends React.Component {
    render() {
        let items = this.props.pl.map((t) => ({ item: t, selected: false }));
        if (this.props.current !== -1 && items) {
            console.log(this.props.current);
            items[this.props.current].selected = true;
        }
        return <div><div>
            <VSGrid
                itemData={items}
                columnCount={5}
                columnWidth={columnWidths}
                height={700}
                rowCount={this.props.pl.length}
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