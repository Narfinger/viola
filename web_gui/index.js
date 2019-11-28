'use strict';

import ReactDOM from 'react-dom'
import React from 'react'
import axios from 'axios';
const e = React.createElement;

var PlayState = {
    Stopped: 1,
    Paused: 2,
    Playing: 3
};


function TransportButton(props) {
    return <a href="#" onClick="axios.get(/api/transport/{props.api})">{props.title}</a>
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
    return <Button title="Unspecified"></Button>
}


const domContainer = document.querySelector('#like_button_container');
ReactDOM.render(e(LikeButton), domContainer);