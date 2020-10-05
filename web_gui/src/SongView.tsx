import * as React from "react";
import { VariableSizeGrid as VSGrid } from "react-window";
import Tabs from "@material-ui/core/Tabs";
import axios from "axios";
import PlaylistTab from "./PlaylistTab";
import { columnWidths, Cell } from "./Cell";

type SongViewProps = {
  pl: any;
  current: number;
  current_playing: number;
  tabs: any;
  playing: boolean;
};

type SongViewState = {
  value: number;
};

export default class SongView extends React.Component<
  SongViewProps,
  SongViewState
  > {
  constructor(props) {
    super(props);
    this.state = {
      value: 0,
    };
    this.handleChange = this.handleChange.bind(this);
  }

  handleChange(event: React.ChangeEvent, newValue: number): void {
    if (newValue !== this.state.value) {
      this.setState({ value: newValue });
      axios.post("/playlisttab/", { index: newValue });
    }
  }

  setTab(i: number): void {
    this.setState({ value: i });
  }

  render(): JSX.Element {
    if (this.props.pl.length == 0) {
      return (<div></div>)
    }
    const items = this.props.pl.map((t) => ({
      item: t,
      selected: false,
      playing: this.props.playing,
    }));
    // sets the correct index to playing. if there is nothing playing, we don't set anything
    if (this.props.current !== -1 && items) {
      // console.log(this.props.current);
      items[this.props.current].selected = true;
    }

    return (
      <div>
        <Tabs
          value={this.state.value}
          /*onChange={this.handleChange}*/ aria-label="simple tabs example"
        >
          {this.props.tabs.map((t, index) => (
            <PlaylistTab
              handleChange={this.handleChange}
              key={index}
              t={t}
              index={index}
              label={t}
            />
          ))}
        </Tabs>
        <VSGrid
          itemData={items}
          columnCount={9}
          columnWidth={columnWidths}
          height={600}
          rowCount={this.props.pl.length}
          rowHeight={(index) => {
            return 25;
          }}
          width={1600}
        >
          {Cell}
        </VSGrid>
      </div>
    );
  }
}
