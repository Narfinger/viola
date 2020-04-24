import * as ReactDOM from 'react-dom'
import { HotKeys } from "react-hotkeys";
import * as React from "react";
import Button from '@material-ui/core/Button';
import Grid from '@material-ui/core/Grid';
import axios from 'axios';
import SongView from './SongView';
import { TransportButton, ButtonEvent, PlayState, PlayButton } from './TransportButton';
import LibraryDrawer from './LibraryDrawer';
import DeleteRangeButton from './DeleteRangeButton';
import { convertSecondsToTime, TrackType } from './Cell';

const e = React.createElement;

function playstate_from_string(input: string) {
    if (input === "Stopped") {
        return PlayState.Stopped;
    } else if (input === "Playing") {
        return PlayState.Playing;
    } else if (input === "Pausing") {
        return PlayState.Paused;
    }
}

function playstate_to_string(input: PlayState) {
    if (input === PlayState.Stopped) {
        return "Stopped";
    } else if (input === PlayState.Playing) {
        return "Playing";
    } else if (input === PlayState.Paused) {
        return "Paused";
    }
}

const keyMap = {
    PLAYPAUSE: ["space", "c"]
};

type MainState = {
    status: PlayState,
    current: number,
    pl: TrackType[],
    pltime: string,
    image_hash: string,
    imageHash: number,
    eventblock: boolean,
    tabs: object[],
    time_state: number,
    repeat: boolean,
}

class Main extends React.Component<{}, MainState> {
    ws: WebSocket;
    songview: React.RefObject<SongView>;

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
            time_state: 0,
            repeat: false,
        };

        this.handleButtonPush = this.handleButtonPush.bind(this);
        this.refresh = this.refresh.bind(this);
        this.clean = this.clean.bind(this);
        this.again = this.again.bind(this);
        this.ws = new WebSocket("ws://" + window.location.hostname + ":" + window.location.port + "/ws/")
        this.songview = React.createRef();
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
            });
            this.songview.current.setTab(response.data.current_tab);
        })

        this.ws.onopen = () => {
            // on connecting, do nothing but log it to the console
            console.log('websocket connected')
        }

        this.ws.onmessage = evt => {
            const msg = JSON.parse(evt.data);
            //console.log(msg);
            switch (msg.type) {
                case "Ping": break;
                case "PlayChanged": {
                    console.log(msg);
                    this.setState({ current: msg.index, status: PlayState.Playing, time_state: 0, repeat: false });
                    this.refresh();
                    break;
                }
                case "CurrentTimeChanged": {
                    this.setState({ time_state: parseInt(msg.index, 10) });
                    break;
                }
                case "ReloadPlaylist": {
                    console.log(msg);
                    axios.get("/playlist/").then((response) => this.setState({
                        pl: response.data,
                        time_state: 0,
                    }));
                    break;
                }
                case "ReloadTabs": {
                    console.log(msg);
                    axios.get("/playlisttab/").then((response) => {
                        this.setState({
                            tabs: response.data.tabs,
                            time_state: 0,
                        });
                        this.songview.current.setTab(response.data.current_tab);
                    });
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
        this.refresh();
    }

    again() {
        axios.post("/repeat/");
        this.setState({ repeat: true });
    }

    save() {
        console.log("trying to save");
        axios.post("/save/");
    }

    handleButtonPush(event) {
        if (!this.state.eventblock) {
            if (event === ButtonEvent.Play) {
                axios.post("/transport/", { "t": "Playing" });
                this.setState({ status: PlayState.Playing });
            } else if (event === ButtonEvent.Pause) {
                axios.post("/transport/", { "t": "Pausing" });
                this.setState({ status: PlayState.Paused });
            } else if (event === ButtonEvent.Previous) {
                axios.post("/transport/", { "t": "Previous" });
            } else if (event === ButtonEvent.Next) {
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
        const is_playing = this.state.status === PlayState.Playing;
        const left_to_go = this.state.pl.length - this.state.current;
        const cover_src = "/currentimage/?" + this.state.imageHash;
        let current_total_time = "";
        if (this.state.pl && this.state.pl[this.state.current] && this.state.pl[this.state.current]) {
            current_total_time = convertSecondsToTime(this.state.pl[this.state.current].length);
        }
        const timestate = convertSecondsToTime(this.state.time_state);
        let repeat = "";
        if (this.state.repeat) {
            repeat = "repeat";
        }


        return <HotKeys keyMap={keyMap} handlers={this.hotkey_handlers}>
            <div>
                <Grid container spacing={1}>
                    <Grid item xs={1}>
                        <LibraryDrawer></LibraryDrawer>
                    </Grid>
                    <Grid item xs={2}>
                        <TransportButton title="Prev" click={this.handleButtonPush} event={ButtonEvent.Previous}></TransportButton>
                    </Grid>
                    <Grid item xs={2}>
                        <PlayButton play_state={this.state.status} click={this.handleButtonPush}></PlayButton>
                    </Grid>
                    <Grid item xs={1}>
                        <TransportButton title="Next" click={this.handleButtonPush} event={ButtonEvent.Next}></TransportButton>
                    </Grid>
                    <Grid item xs={1}>
                        <Button variant="contained" color="primary" onClick={this.again}>Again</Button>
                    </Grid>
                    <Grid item xs={1}>
                        <Button variant="contained" color="secondary" onClick={this.clean}>Clean</Button>
                    </Grid>
                    <Grid item xs={1}>
                        <DeleteRangeButton></DeleteRangeButton>
                    </Grid>
                    <Grid item xs={1}>
                        <Button variant="contained" color="primary" onClick={this.save}>Save</Button>
                    </Grid>
                    <Grid item xs={1}>
                        {playstate_to_string(this.state.status)}
                    </Grid>
                    <Grid item xs={12}>
                        <SongView ref={this.songview} current={this.state.current} pl={this.state.pl} playing={is_playing} tabs={this.state.tabs} />
                    </Grid>
                    <Grid container alignItems="center">
                        <Grid item xs={2}>
                            <img alt="" height="100px" width="100px" src={cover_src} />
                        </Grid>
                        <Grid item xs={2}>
                            Playlist Count: {this.state.pl.length}
                        </Grid>
                        <Grid item xs={2}>
                            Tracks left: {left_to_go}
                        </Grid>
                        <Grid item xs={2}>
                            Time: {this.state.pltime}
                        </Grid>
                        <Grid item xs={2}>
                            Time: {timestate}---{current_total_time}
                        </Grid>
                        <Grid item xs={1}>
                            {repeat}
                        </Grid>
                    </Grid>
                </Grid>
            </div >
        </HotKeys>
    }
}
ReactDOM.render(<Main></Main>, document.querySelector('#main_container'));