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
#[derive(Debug, Serialize, Deserialize)]
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

/// General type to communicate with treeviews
#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralTreeViewJson<T> {
    pub value: String,
    pub children: Vec<T>,
    pub optional: Option<i32>,
}

pub type Album = GeneralTreeViewJson<Track>;
pub type Artist = GeneralTreeViewJson<Album>;
pub type Smartplaylists = Vec<String>;

impl From<(String, Option<i32>)> for Album {
    fn from(s: (String, Option<i32>)) -> Self {
        Album {
            value: s.0,
            children: vec![],
            optional: s.1,
        }
    }
}

impl From<String> for Artist {
    fn from(s: String) -> Self {
        Artist {
            value: s,
            optional: None,
            children: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadSmartPlaylistJson {
    pub index: usize,
}
