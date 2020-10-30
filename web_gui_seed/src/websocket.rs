use seed::{prelude::*, *};
use viola_common::WsMessage;
const WS_URL: &str = "ws://127.0.0.1:8088/ws/";

pub fn create_websocket(orders: &impl Orders<crate::Msg>) -> WebSocket {
    let msg_sender = orders.msg_sender();

    WebSocket::builder(WS_URL, orders)
        .on_message(move |msg| decode_message(msg, msg_sender))
        .build_and_open()
        .unwrap()
}

fn decode_message(message: WebSocketMessage, msg_sender: std::rc::Rc<dyn Fn(Option<crate::Msg>)>) {
    if message.contains_text() {
        let msg = message
            .json::<viola_common::WsMessage>()
            .expect("Failed to decode WebSocket text message");
        match msg {
            WsMessage::Ping => {}
            WsMessage::PlayChanged(index) => {
                msg_sender(Some(crate::Msg::PlaylistIndexChange(index)))
            }
            WsMessage::CurrentTimeChanged(index) => {}
            WsMessage::ReloadTabs => msg_sender(Some(crate::Msg::InitPlaylistTabs)),
            WsMessage::ReloadPlaylist => {}
        };
    }
}
