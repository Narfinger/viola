#[cfg(feature = "backend")]
#[macro_use]
extern crate actix_derive;
#[cfg(feature = "backend")]
use actix_derive::{Message, MessageResponse};

#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "backend")]
#[macro_use]
pub mod schema;
#[cfg(feature = "backend")]
use crate::schema::tracks;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(AsChangeset, Identifiable, Queryable))]
pub struct Track {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub tracknumber: Option<i32>,
    pub year: Option<i32>,
    pub path: String,
    pub length: i32,
    pub albumpath: Option<String>,
    pub playcount: Option<i32>,
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum GStreamerAction {
    Next,
    Playing,
    Pausing,
    Previous,
    Stop,
    // This means we selected one specific track
    Play(usize),
    Seek(u64),
    RepeatOnce, // Repeat the current playing track after it finishes
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "backend", derive(Message))]
#[cfg_attr(feature = "backend", rtype(result = "()"))]
pub enum WsMessage {
    PlayChanged { index: usize },
    CurrentTimeChanged { index: u64 },
    ReloadTabs,
    ReloadPlaylist,
    Ping,
}

impl From<WsMessage> for String {
    fn from(msg: WsMessage) -> Self {
        serde_json::to_string(&msg).unwrap()
    }
}
