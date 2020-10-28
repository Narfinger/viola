use seed::{prelude::*, *};
const WS_URL: &str = "ws://127.0.0.1:9000/ws";

pub fn create_websocket(orders: &impl Orders<Msg>) -> WebSocket {
    let msg_sender = orders.msg_sender();

    WebSocket::builder(WS_URL, orders)
        .on_message(move |msg| decode_message(msg, msg_sender))
        .build_and_open()
        .unwrap()
}

fn decode_message(message: WebSocketMessage, msg_sender: Rc<dyn Fn(Option<Msg>)>) {
    if message.contains_text() {
        match message.text() {
            "PlayChanged" => {}
            "CurrentTimeChanged" => {}
            "ReloadTabs" => {}
            "ReloadPlaylist" => {}
            "Ping" => {}
        }
    }
}
