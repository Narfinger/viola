import * as React from "react";
import Drawer from '@material-ui/core/Drawer';
import Button from '@material-ui/core/Button';
import LibraryView from './LibraryView';

type LibraryDrawerState = {
    open: boolean,
}

export default class LibraryDrawer extends React.Component<{}, LibraryDrawerState> {
    constructor(props) {
        super(props);

        // This binding is necessary to make `this` work in the callback
        this.click = this.click.bind(this);
        this.close = this.close.bind(this);
        this.state = { open: false };
    }
    click() {
        this.setState({ open: true })
    }
    close() {
        this.setState({ open: false })
    }
    render() {
        return <div>
            <Button onClick={this.click} color="primary" >Lib</Button>
            <Drawer anchor="left" open={this.state.open} onClose={this.close}>
                <LibraryView></LibraryView>
            </Drawer>
        </div>
    }
}