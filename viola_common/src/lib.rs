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

/// Actions we want to perform on gstreamer, such as playing and pausing
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
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

/// Messages that gstreamer sends such as the state it is going into
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum GStreamerMessage {
    Pausing,
    Stopped,
    Playing,
    Nop,
    ChangedDuration((u64, u64)), //in seconds
}

impl std::fmt::Display for GStreamerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GStreamerMessage::Pausing => write!(f, "Paused"),
            GStreamerMessage::Stopped => write!(f, "Stopped"),
            GStreamerMessage::Playing => write!(f, "Playing"),
            GStreamerMessage::Nop => write!(f, "NOP"),
            GStreamerMessage::ChangedDuration((_, _)) => write!(f, "NOP"),
        }
    }
}

impl From<GStreamerAction> for GStreamerMessage {
    fn from(action: GStreamerAction) -> Self {
        match action {
            GStreamerAction::Pausing => GStreamerMessage::Pausing,
            GStreamerAction::Stop => GStreamerMessage::Stopped,
            GStreamerAction::Seek(_) | GStreamerAction::RepeatOnce => GStreamerMessage::Nop,
            GStreamerAction::Next
            | GStreamerAction::Previous
            | GStreamerAction::Play(_)
            | GStreamerAction::Playing => GStreamerMessage::Playing,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "backend", derive(Message))]
#[cfg_attr(feature = "backend", rtype(result = "()"))]
pub enum WsMessage {
    PlayChanged(usize),
    CurrentTimeChanged(u64),
    ReloadTabs,
    ReloadPlaylist,
    Ping,
}

impl From<WsMessage> for String {
    fn from(msg: WsMessage) -> Self {
        serde_json::to_string(&msg).unwrap()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TreeType {
    Artist,
    Album,
    Track,
}
/// General type to query a treeview
#[derive(Debug, Serialize, Deserialize)]
pub struct TreeViewQuery {
    /// Which types we want in order
    pub types: Vec<TreeType>,
    /// which indices we want in order, having [1] means, we want the children of the second types[0]
    pub indices: Vec<usize>,
    /// Optional search string to restrict
    pub search: Option<String>,
}

pub type Smartplaylists = Vec<String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadSmartPlaylistJson {
    pub index: usize,
}
