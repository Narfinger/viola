'use strict';

import ReactDOM from 'react-dom'
import React from 'react'
import { FixedSizeGrid as Grid } from 'react-window';
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
        return <button onClick={this.click}> {this.props.title}</button>
    }
}

function PlayButton(props) {
    if (props.play_state == PlayState.Stopped) {
        return <TransportButton title="Play" api="play"></TransportButton>
    };
    if (props.play_state == PlayState.Paused) {
        return <TransportButton title="Play" api="play"></TransportButton>
    };
    if (props.play_state == PlayState.Playing) {
        return <TransportButton title="Pause" api="pause"></TransportButton>
    }
    return <TransportButton title="Unspecified" api="NI"></TransportButton>
}

function Main() {
    return <div><div align="center">
        <TransportButton title="Prev" api="prev"></TransportButton>
        <PlayButton></PlayButton>
        <TransportButton title="Next" api="next"></TransportButton>
    </div>
        <div>
            <SongView></SongView>
        </div>
    </div>
};


const Cell = ({ columnIndex, rowIndex, style }) => (
    <div style={style}>
        Item {rowIndex},{columnIndex}
    </div>
);

class SongView extends React.Component {
    constructor(props) {
        super(props)
        this.state = { data: axios.get("/playlist/").then(| v | v).else([]) }
    }

    render() {
        <Grid
            columnCount={5}
            columnWidth={this.data.length}
            height={150}
            rowCount={1000}
            rowHeight={35}
            width={300}
        >
            <Cell></Cell>
        </Grid>

    }
}

const Example = () => (
    <List
        height={700}
        itemCount={10000}
        itemSize={35}
        width={800}
    >
        {Row}
    </List>
);



const domContainer = document.querySelector('#main_container');
ReactDOM.render(e(Main), domContainer);