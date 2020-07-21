use app_dirs::*;
use gstreamer::Element;
use std::sync::{Arc, Mutex, RwLock};

use crate::gstreamer_wrapper::GStreamerMessage;
use crate::loaded_playlist::LoadedPlaylist;
//use crate::maingui::MainGui;
use crate::playlist_tabs::PlaylistTabs;

pub const APP_INFO: AppInfo = AppInfo {
    name: "viola",
    author: "Narfinger",
};

pub const LENGTH_COLUMN: i32 = 4;
pub const YEAR_COLUMN: i32 = 5;
pub const PLAYCOUNT_COLUMN: i32 = 7;
pub const COLOR_COLUMN: u32 = 8;

//pub type BuilderPtr = Arc<RwLock<Builder>>;
pub type GstreamerPipeline = Arc<RwLock<Element>>;
//pub type MainGuiPtr = Rc<MainGui>;
//pub type MainGuiWeakPtr = Weak<MainGui>;
pub type DBPool = Arc<Mutex<diesel::SqliteConnection>>;
//pub type PlaylistTabsPtr = Rc<RefCell<PlaylistTabs>>;
pub type PlaylistTabsPtr = Arc<RwLock<PlaylistTabs>>;
pub type LoadedPlaylistPtr = RwLock<LoadedPlaylist>;

/// General type to communicate with treeviews
#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralTreeViewJson<T> {
    pub value: String,
    pub children: Vec<T>,
    pub optional: Option<i32>,
}

pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
    ChangedDuration((u64, u64)),
}

impl From<GStreamerMessage> for PlayerStatus {
    fn from(item: GStreamerMessage) -> Self {
        match item {
            GStreamerMessage::Pausing => PlayerStatus::Paused,
            GStreamerMessage::Stopped => PlayerStatus::Stopped,
            GStreamerMessage::Playing => PlayerStatus::Playing,
            GStreamerMessage::ChangedDuration(i) => PlayerStatus::ChangedDuration(i),
            GStreamerMessage::Nop => PlayerStatus::Playing,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub from: usize,
    pub to: Option<usize>,
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {})",
            self.from,
            self.to.map_or(String::from("infty"), |s| s.to_string())
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ChangePlaylistTabJson {
    pub index: usize,
}

#[derive(Debug, Deserialize)]
pub struct LoadSmartPlaylistJson {
    pub index: usize,
}
