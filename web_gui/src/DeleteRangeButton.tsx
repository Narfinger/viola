import * as React from "react";
import Paper from "@material-ui/core/Paper";
import Input from "@material-ui/core/Input";
import ExpandMoreIcon from "@material-ui/icons/ExpandMore";
import ChevronRightIcon from "@material-ui/icons/ChevronRight";
import TreeView from "@material-ui/lab/TreeView";
import TreeItem from "@material-ui/lab/TreeItem";
import AwesomeDebouncePromise from "awesome-debounce-promise";
import Popover from "@material-ui/core/Popover";
import LibraryMenu from "./LibraryMenu";
import axios from "axios";
import Button from "@material-ui/core/Button";
import TextField from "@material-ui/core/TextField";
import Dialog from "@material-ui/core/Dialog";
import DialogActions from "@material-ui/core/DialogActions";
import DialogContent from "@material-ui/core/DialogContent";
import DialogContentText from "@material-ui/core/DialogContentText";
import DialogTitle from "@material-ui/core/DialogTitle";

type DeleteRangeButtonState = {
  open: boolean;
  text: string;
};

export default class DeleteRangeButton extends React.Component<
  {},
  DeleteRangeButtonState
> {
  constructor(props) {
    super(props);
    this.state = {
      open: false,
      text: "",
    };
    this.handleClickOpen = this.handleClickOpen.bind(this);
    this.handleCancel = this.handleCancel.bind(this);
    this.handleSubmit = this.handleSubmit.bind(this);
    this.textfieldChange = this.textfieldChange.bind(this);
  }

  textfieldChange(event) {
    this.setState({ text: event.target.value });
  }

  handleClickOpen() {
    this.setState({ open: true });
  }

  handleCancel() {
    this.setState({ open: false, text: "" });
  }

  handleSubmit() {
    const range = this.state.text.split("-");
    axios.delete("/deletefromplaylist/", {
      data: {
        from: range[0],
        to: range[1],
      },
    });
  }

  render() {
    return (
      <div>
        <Button
          variant="contained"
          color="secondary"
          onClick={this.handleClickOpen}
        >
          DelRange
        </Button>
        <Dialog
          open={this.state.open}
          onClose={this.handleCancel}
          aria-labelledby="form-dialog-title"
        >
          <DialogTitle id="form-dialog-title">
            Delete Range from Playlist
          </DialogTitle>
          <DialogContent>
            <DialogContentText>Delete the range from-to.</DialogContentText>
            <TextField
              autoFocus
              margin="dense"
              id="name"
              label="range"
              fullWidth
              onChange={this.textfieldChange}
              value={this.state.text}
            />
          </DialogContent>
          <DialogActions>
            <Button onClick={this.handleCancel} color="primary">
              Cancel
            </Button>
            <Button onClick={this.handleSubmit} color="secondary">
              Delete
            </Button>
          </DialogActions>
        </Dialog>
      </div>
    );
  }
}
