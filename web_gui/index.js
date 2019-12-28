
import ReactDOM from 'react-dom'
import React from 'react'
import Paper from '@material-ui/core/Paper';
import Button from '@material-ui/core/Button';
import Grid from '@material-ui/core/Grid';
import TreeView from '@material-ui/lab/TreeView';
import TreeItem from '@material-ui/lab/TreeItem';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import { VariableSizeGrid as VSGrid } from 'react-window';
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
            {value === index && <Box p={3}>{children}</Box>}
        </Typography>
    );
}

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
        this.clean = this.clean.bind(this);
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
                case "PlayChanged": this.setState({ current: msg.index, status: PlayState.Playing }); break;
                default:
            }
        }

        this.ws.onclose = () => {
            console.log('disconnected')
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

    }

    handleButtonPush(e) {
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
                <Grid item xs={3}>
                    <Button variant="contained" color="secondary" onClick={this.clean}>Clean</Button>
                </Grid>
                <Grid item xs={2}>
                    <ArtistTreeView></ArtistTreeView>
                </Grid>
                <Grid item xs={10}>
                    {/* <Tabs orientation="vertical"
                        variant="scrollable">
                        <Tab label="Artist View"></Tab>
                    </Tabs>
    <TabPanel index={0}> */}
                    <SongView current={this.state.current} pl={this.state.pl}></SongView>
                    {/*}  </TabPanel> */}
                </Grid>
            </Grid>
        </div >
    }
}

function columnWidths(index) {
    switch (index) {
        case 0: return 100;  //number
        case 1: return 400; //title
        case 2: return 300; //artist
        case 3: return 300; //album
        case 4: return 200; //genre
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
        if (selected) {
            let style = JSON.parse(JSON.stringify(this.props.style));
            style.color = "#FF0000";
            //console.log(style);
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
                case 0: return <div style={this.props.style} onDoubleClick={this.click}>{this.props.rowIndex}</div>
                //<PlayIndexButton style={this.props.style} index={this.props.rowIndex}></PlayIndexButton>//<div style={this.props.style}>{this.props.rowIndex}</div>
                case 1: return <div style={this.props.style} onDoubleClick={this.click}>{item.title}</div>
                case 2: return <div style={this.props.style} onDoubleClick={this.click}>{item.artist}</div>
                case 3: return <div style={this.props.style} onDoubleClick={this.click}>{item.album}</div>
                case 4: return <div style={this.props.style} onDoubleClick={this.click}>{item.genre}</div>
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

class ArtistTreeView extends React.Component {
    constructor(props) {
        super(props)
        this.state = {
            items: [
                { name: "v1", children: [] },
                { name: "v2", children: [] },
                {
                    name: "v3", children: [{ name: "vv3", children: [] }, { name: "vvv3", children: [] },
                    { name: "vvv3", children: [] },

                    ]
                },
                { name: "v4", children: [] },
                { name: "v5", children: [] }
            ]
        };
    }

    componentDidMount() {
        axios.get("/libraryview/artist/").then((response) => this.setState({
            items: response.data
        }));
    }


    render() {
        return <Paper style={{ maxHeight: 800, overflow: 'auto' }}>
            <TreeView height="60vh">
                {
                    this.state.items.map((value, index) => {
                        return <TreeItem nodeId={index} key={index} label={value.name}>
                            {value.children.map((v2, i2) => {
                                return <TreeItem nodeId={10000 * index + i2} key={10000 * index + i2} label={v2.name}></TreeItem>
                            })}
                        </TreeItem>
                    })
                }
            </TreeView >
        </Paper>
    }
}

const domContainer = document.querySelector('#main_container');
ReactDOM.render(e(Main), domContainer);