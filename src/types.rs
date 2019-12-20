use app_dirs::*;
use diesel::SqliteConnection;
use gstreamer::Element;
use gtk::Builder;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, RwLock};

use crate::gstreamer_wrapper::GStreamerMessage;
use crate::loaded_playlist::LoadedPlaylist;
//use crate::maingui::MainGui;
//use crate::playlist_tabs::PlaylistTabs;

pub const APP_INFO: AppInfo = AppInfo {
    name: "viola",
    author: "Narfinger",
};

pub const LENGTH_COLUMN: i32 = 4;
pub const YEAR_COLUMN: i32 = 5;
pub const PLAYCOUNT_COLUMN: i32 = 7;
pub const COLOR_COLUMN: u32 = 8;

pub type BuilderPtr = Arc<RwLock<Builder>>;
pub type GstreamerPipeline = Arc<RwLock<Element>>;
//pub type MainGuiPtr = Rc<MainGui>;
//pub type MainGuiWeakPtr = Weak<MainGui>;
pub type DBPool = Arc<Mutex<diesel::SqliteConnection>>;
//pub type PlaylistTabsPtr = Rc<RefCell<PlaylistTabs>>;
pub type LoadedPlaylistPtr = Arc<RwLock<LoadedPlaylist>>;

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
        }
    }
}
