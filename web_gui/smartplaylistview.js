import axios from 'axios';
import MyTreeView from './mytreeview';

export default class SmartplaylistView extends MyTreeView {
    need_to_load(ids) {
        return false;
    }

    componentDidMount() {
        axios.get(this.props.url).then((response) => {
            this.setState({
                items: response.data,
            });
        });
    }

    handleDoubleClick(event, index) {
        event.stopPropagation();
        console.log("doing event");
        console.log(index);
        let i = parseInt(index);
        axios.post("/smartplaylist/load/",
            { "index": i }
        );
    }
}

SmartplaylistView.defaultProps = {
    query_for_details: false,
};