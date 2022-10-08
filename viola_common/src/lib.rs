#[cfg(feature = "backend")]
#[macro_use]
extern crate diesel;
#[cfg(feature = "backend")]
#[macro_use]
pub mod schema;
#[cfg(feature = "backend")]
use crate::schema::tracks;

use std::hash::Hash;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
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

impl Hash for Track {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

/// Actions we want to perform on gstreamer, such as playing and pausing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
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
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum GStreamerMessage {
    Pausing,
    Stopped,
    Playing,
    IncreasePlayCount(usize),
    Nop,
    ChangedDuration((u64, u64)), //in seconds
}

impl std::fmt::Display for GStreamerMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GStreamerMessage::Pausing => write!(f, "Paused"),
            GStreamerMessage::Stopped => write!(f, "Stopped"),
            GStreamerMessage::IncreasePlayCount(_) => write!(f, "IncreasePlayCount"),
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    PlayChanged(usize),
    CurrentTimeChanged(u64),
    ReloadTabs,
    ReloadPlaylist,
    Ping,
    GStreamerMessage(GStreamerMessage),
}

impl From<WsMessage> for String {
    fn from(msg: WsMessage) -> Self {
        serde_json::to_string(&msg).unwrap()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TreeType {
    Artist,
    Album,
    Track,
    Genre,
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

impl TreeViewQuery {
    /// Returns the treetype that is in the types vector after the last index.
    /// If treetype is [Artist, Album] and index is [0] we return Album
    pub fn get_after_last_ttype(&self) -> Option<&TreeType> {
        self.types.get(self.indices.len())
    }

    /// Returns the treetypes that are not yet indexed
    pub fn get_remaining_ttypes(&self) -> &[TreeType] {
        self.types.split_at(self.indices.len() + 1).1
    }

    pub fn get_indexed_ttypes(&self) -> &[TreeType] {
        self.types.split_at(self.indices.len() + 1).0
    }
}

pub type Smartplaylists = Vec<String>;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadSmartPlaylistJson {
    pub index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistTabJSON {
    pub name: String,
    pub current_position: usize,
    pub id: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaylistTabsJSON {
    pub current: usize,
    pub current_playing_in: usize,
    pub tabs: Vec<PlaylistTabJSON>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageQuery {
    pub nonce: String,
}
