#[cfg(feature = "db_support")]
#[macro_use]
extern crate diesel;

#[cfg(feature = "db_support")]
#[macro_use]
mod schema;

#[cfg(feature = "db_support")]
use crate::schema::tracks;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db_support", derive(AsChangeset, Identifiable, Queryable))]
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
