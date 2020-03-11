import ReactDOM from 'react-dom'
import { HotKeys } from "react-hotkeys";
import React from 'react'
import Drawer from '@material-ui/core/Drawer';
import Paper from '@material-ui/core/Paper';
import Button from '@material-ui/core/Button';
import Grid from '@material-ui/core/Grid';
import TreeView from '@material-ui/lab/TreeView';
import TreeItem from '@material-ui/lab/TreeItem';
import Typography from '@material-ui/core/Typography';
import Box from '@material-ui/core/Box';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import { VariableSizeGrid as VSGrid } from 'react-window';
import axios from 'axios';
import LibraryView from './libraryviews';
import SongView from './songview';


const e = React.createElement;

const PlayState = Object.freeze({
    Stopped: 1,
    Paused: 2,
    Playing: 3
});

function playstate_from_string(input) {
    if (input === "Stopped") {
        return PlayState.Stopped;
    } else if (input === "Playing") {
        return PlayState.Playing;
    } else if (input === "Pausing") {
        return PlayState.Paused;
    }
}

function playstate_to_string(input) {
    if (input === PlayState.Stopped) {
        return "Stopped";
    } else if (input === PlayState.Playing) {
        return "Playing";
    } else if (input === PlayState.Paused) {
        return "Paused";
    }
}

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

const keyMap = {
    PLAYPAUSE: ["space", "c"]
};

class Main extends React.Component {
    constructor(props) {
        super(props)
        this.state = {
            status: PlayState.Stopped,
            current: -1,
            pl: [],
            pltime: "",
            image_hash: "",
            imageHash: Date.now(),
            eventblock: false,
            tabs: [],
            initial_tab: 0,
        };

        this.handleButtonPush = this.handleButtonPush.bind(this);
        this.refresh = this.refresh.bind(this);
        this.clean = this.clean.bind(this);
        this.again = this.again.bind(this);
        this.ws = new WebSocket("ws://" + window.location.hostname + ":" + window.location.port + "/ws/")
    }

    hotkey_handlers = {
        PLAYPAUSE: event => {
            if (this.state.status === PlayState.Paused || this.state.status === PlayState.Stopped) {
                this.handleButtonPush(ButtonEvent.Play);
            } else if (this.state.status === PlayState.Playing) {
                this.handleButtonPush(ButtonEvent.Pause);
            } else {
                console.log("status is weird");
                console.log(this.state.status);
            }
        },
    }


    componentDidMount() {
        console.log("we mounted");
        axios.get("/playlist/").then((response) => {
            this.setState({
                pl: response.data
            });
            this.refresh();
        }
        );
        axios.get("/playlisttab/").then((response) => {
            this.setState({
                tabs: response.data.tabs,
                initial_tab: response.data.current_tab,
            });
        })

        this.ws.onopen = () => {
            // on connecting, do nothing but log it to the console
            console.log('websocket connected')
        }

        this.ws.onmessage = evt => {
            var msg = JSON.parse(evt.data);
            console.log(msg);
            switch (msg.type) {
                case "Ping": break;
                case "PlayChanged": {
                    this.setState({ current: msg.index, status: PlayState.Playing });
                    this.refresh();
                    break;
                }
                case "ReloadPlaylist": {
                    axios.get("/playlist/").then((response) => this.setState({
                        pl: response.data,
                    }));
                    break;
                }
                case "ReloadTabs": {
                    axios.get("/playlisttab/").then((response) => this.setState({
                        tabs: response.data,
                        initial_tab: response.data.current_tab,
                    }))
                    break;
                }
                default:
            }
        }

        this.ws.onclose = () => {
            console.log('websocket disconnected')
            // automatically try to reconnect on connection loss

        }
    }

    clean() {
        axios.post("/clean/");
        axios.get("/playlist/").then((response) => this.setState({
            pl: response.data
        }));
        axios.get("/currentid/").then((response) => {
            this.setState({ current: response.data });
        })
        this.refresh();
    }

    again() {
        axios.post("/repeat/");
    }

    save() {
        console.log("trying to save")``;
        axios.post("/save/");
    }

    handleButtonPush(e) {
        if (!this.state.eventblock) {
            if (e === ButtonEvent.Play) {
                axios.post("/transport/", { "t": "Playing" });
                this.setState({ status: PlayState.Playing });
            } else if (e === ButtonEvent.Pause) {
                axios.post("/transport/", { "t": "Pausing" });
                this.setState({ status: PlayState.Paused });
            } else if (e === ButtonEvent.Previous) {
                axios.post("/transport/", { "t": "Previous" });
            } else if (e === ButtonEvent.Next) {
                axios.post("/transport/", { "t": "Next" });
            } else {
                console.log("Unspecified!");
            }
            this.setState({ eventblock: true });
            setTimeout(() => this.setState({ eventblock: false }), 1000);
        }
    }

    refresh() {
        axios.get("/currentid/").then((response) => {
            this.setState({ current: response.data });
        });
        axios.get("/transport/").then((response) => {
            this.setState({
                status: playstate_from_string(response.data)
            });
        });
        axios.get("/pltime/").then((response) => {
            this.setState({ pltime: response.data });
        });
        this.setState({ imageHash: Date.now() });
    }

    render() {
        return <HotKeys keyMap={keyMap} handlers={this.hotkey_handlers}>
            <div>
                <Grid container spacing={1}>
                    <Grid item xs={1}>
                        <LibraryDrawer></LibraryDrawer>
                    </Grid>
                    <Grid item xs={2}>
                        <TransportButton title="Prev" event="ButtonEvent.Previous" click={this.handleButtonPush} event={ButtonEvent.Previous}></TransportButton>
                    </Grid>
                    <Grid item xs={2}>
                        <PlayButton play_state={this.state.status} click={this.handleButtonPush}></PlayButton>
                    </Grid>
                    <Grid item xs={1}>
                        <TransportButton title="Next" api="next" event="ButtonEvent.Next" click={this.handleButtonPush} event={ButtonEvent.Next}></TransportButton>
                    </Grid>
                    <Grid item xs={1}>
                        <Button variant="contained" color="primary" onClick={this.again}>Again</Button>
                    </Grid>
                    <Grid item xs={1}>
                        <Button variant="contained" color="secondary" onClick={this.clean}>Clean</Button>
                    </Grid>
                    <Grid item xs={1}>
                        <Button variant="contained" color="secondary" onClick={this.save}>Save</Button>
                    </Grid>
                    <Grid item xs={1}>
                        {playstate_to_string(this.state.status)}
                    </Grid>
                    <Grid item xs={10}>
                        <SongView current={this.state.current} pl={this.state.pl} playing={this.state.status === PlayState.Playing} tabs={this.state.tabs} initial_tab={this.state.initial_tab} />
                    </Grid>
                    <Grid item xs={10}>
                        <img height="100px" width="100px" src={"/currentimage/" + "?" + this.state.imageHash} /> Playlist Count: {this.state.pl.length} | Left to go: {this.state.pl.length - this.state.current} | Time: {this.state.pltime}
                    </Grid>
                </Grid>
            </div >
        </HotKeys>
    }
}

class LibraryDrawer extends React.Component {
    constructor(props) {
        super(props);

        // This binding is necessary to make `this` work in the callback
        this.click = this.click.bind(this);
        this.close = this.close.bind(this);
        this.state = { open: false };
    }
    click() {
        this.setState({ open: true })
    }
    close() {
        this.setState({ open: false })
    }
    render() {
        return <div>
            <Button onClick={this.click} color="primary" >Lib</Button>
            <Drawer anchor="left" open={this.state.open} onClose={this.close}>
                <LibraryView></LibraryView>
            </Drawer>
        </div>
    }
}

const domContainer = document.querySelector('#main_container');
ReactDOM.render(e(Main), domContainer);