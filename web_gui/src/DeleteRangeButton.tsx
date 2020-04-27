import * as React from "react";
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

  textfieldChange(event): void {
    this.setState({ text: event.target.value });
  }

  handleClickOpen(): void {
    this.setState({ open: true });
  }

  handleCancel(): void {
    this.setState({ open: false, text: "" });
  }

  handleSubmit(): void {
    const range = this.state.text.split("-");
    axios.delete("/deletefromplaylist/", {
      data: {
        from: range[0],
        to: range[1],
      },
    });
  }

  render(): JSX.Element {
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
