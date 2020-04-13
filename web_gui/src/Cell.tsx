import * as React from "react";
import Typography from '@material-ui/core/Typography';
import axios from 'axios';


export type TrackType = {
    trachnumber: number,
    title: string,
    artist: string,
    album: string,
    genre: string,
    year: number,
    length: number,
}

export function convertSecondsToTime(seconds: number) {
    const date = new Date(0);
    date.setSeconds(seconds);
    return date.toISOString().substr(14, 5);
}

export function columnWidths(index: number) {
    switch (index) {
        case 0: return 50; // number
        case 1: return 50; // tracknumber
        case 2: return 400; // title
        case 3: return 300; // artist
        case 4: return 300; // album
        case 5: return 200; // genre
        case 6: return 100; // year
        case 7: return 50; // time
        case 8: return 50
        default: return 10000;
    }
}

type CellProps = {
    columnIndex: number,
    rowIndex: number,
    data: {
        item: TrackType[],
        selected: number,
        playing: boolean,
    },
    style: any,
}
export class Cell extends React.PureComponent<CellProps, {}> {
    constructor(props) {
        super(props)
        this.click = this.click.bind(this);
    }
    click() {
        console.log("clicked");
        const c = {
            "t": "Play",
            c: this.props.rowIndex,
        };
        axios.post("/transport/", c);

        console.log(this.props);
    }

    render() {
        const { item, selected, playing } = this.props.data[this.props.rowIndex];
        // console.log(style);
        let st = "";
        switch (this.props.columnIndex) {
            case 0: st = this.props.rowIndex.toString(); break;
            case 1: st = item.tracknumber; break;
            case 2: st = item.title; break;
            case 3: st = item.artist; break;
            case 4: st = item.album; break;
            case 5: st = item.genre; break;
            case 6: {
                if (item.year !== -1) {
                    st = item.year;
                } else {
                    st = "";
                }
                break;
            }
            case 7: st = convertSecondsToTime(item.length); break;
            case 8: st = item.playcount; break;
            default: st = "ERROR";
        }
        const style = JSON.parse(JSON.stringify(this.props.style));
        style.textOverflow = "ellipsis";
        style.width = columnWidths(this.props.columnIndex) + "px";
        if (selected) {
            if (playing) {
                style.color = "#009100";
            } else {
                style.color = "#FF0000";
            }
        }
        return <div style={style} onDoubleClick={this.click}><Typography noWrap>{st}</Typography></div>

    }
}