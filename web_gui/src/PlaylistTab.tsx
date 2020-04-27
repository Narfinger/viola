import * as React from "react";
import DeleteIcon from "@material-ui/icons/Delete";
import IconButton from "@material-ui/core/IconButton";
import Tab from "@material-ui/core/Tab";
import axios from "axios";

type PlaylistTabProps = {
  handleChange: (e: React.ChangeEvent, i: number) => void;
  index: number;
  t: string;
  label: string;
};

export default class PlaylistTab extends React.Component<PlaylistTabProps, {}> {
  constructor(props) {
    super(props);
    this.click = this.click.bind(this);
  }

  click(event) {
    console.log('tagname "' + event.target.tagName + '"');
    if (event.target.tagName === "SPAN") {
      this.props.handleChange(event, this.props.index);
    } else if (
      event.target.tagName === "svg" ||
      event.target.tagName === "path"
    ) {
      console.log("deleting playlisttab id " + this.props.index);
      axios.delete("/playlisttab/", { data: { index: this.props.index } });
      event.preventDefault();
    } else {
      return;
    }
  }

  render() {
    return (
      <div /*className={this.props.className}*/ onClick={this.click}>
        <Tab
          /*className={this.props.className}*/ label={this.props.t}
          value={this.props.index}
          key={this.props.index}
        />
        <IconButton aria-label="delete">
          <DeleteIcon fontSize="small" />
        </IconButton>
      </div>
    );
  }
}
