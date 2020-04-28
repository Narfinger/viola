import * as React from "react";
import Popover from "@material-ui/core/Popover";
import Paper from "@material-ui/core/Paper";
import Input from "@material-ui/core/Input";
import Menu from "@material-ui/core/Menu";
import MenuItem from "@material-ui/core/MenuItem";


type LibraryMenuProps = {
  open: boolean;
  index: string;
  anchor: any;
  closeFn: () => void;
};
export default class LibraryMenu extends React.Component<LibraryMenuProps, {}> {
  constructor(props) {
    super(props);
    this.handleNewPL = this.handleNewPL.bind(this);
    this.handleAppendToPL = this.handleAppendToPL.bind(this);
  }

  handleNewPL() {
    console.log("would new");
    this.props.closeFn();
  }

  handleAppendToPL() {
    console.log("would append");
    this.props.closeFn();
  }

  render(): JSX.Element {
    return <Menu id="simple-menu" anchorEl={this.props.anchor} open={this.props.open} onClose={(e) => this.props.closeFn()}>
      <MenuItem onClick={this.handleNewPL}>New PL</MenuItem>
      <MenuItem onClick={this.handleAppendToPL}>Append to PL</MenuItem>
    </Menu>
  }
}
