import axios from 'axios';
import MyTreeView from './mytreeview';

export default class SmartplaylistView extends MyTreeView {
    handleDoubleClick(event, index) {
        event.stopPropagation();
        console.log("doing event");
        axios.post("/smartplaylist/load/", {
            index
        });
    }
}

SmartplaylistView.defaultProps = {
    query_for_details: true,
  };