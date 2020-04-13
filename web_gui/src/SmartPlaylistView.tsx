import axios from 'axios';
import MyTreeView from './MyTreeView';

export default class SmartplaylistView extends MyTreeView {
    public static defaultProps = {
        query_for_details: false,
        query_params_list: [],
    };
    need_to_load(ids: number[]) {
        return false;
    }

    componentDidMount() {
        axios.get(this.props.url).then((response) => {
            this.setState({
                items: response.data,
            });
        });
    }

    handleDoubleClick(event, index: string) {
        event.stopPropagation();
        console.log("doing event");
        console.log(index);
        const i = parseInt(index, 10);
        axios.post("/smartplaylist/load/",
            { "index": i }
        );
    }
}