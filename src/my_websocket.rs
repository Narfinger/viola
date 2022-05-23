use futures::{stream::SplitSink, SinkExt};
use std::{ops::DerefMut, sync::Arc};
use tokio::sync::RwLock;
use viola_common::WsMessage;
use warp::ws::Message;

pub type MyWs = Arc<RwLock<Option<SplitSink<warp::ws::WebSocket, warp::ws::Message>>>>;

pub async fn send_my_message(socket: &MyWs, msg: WsMessage) {
    info!("Sending msg to websocket {:?}", msg);
    let mut socket = socket.write().await;
    if let Some(sink) = socket.deref_mut() {
        let st = serde_json::to_string(&msg).expect("Error serializing");
        let msg = Message::text(&st);
        if sink.send(msg).await.is_err() {
	        info!("Closing Websocket");
            //sink.close().await.expect("Could not close socket");
            socket.take();
        }
    }
}
