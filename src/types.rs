use app_dirs::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use gstreamer::Element;
use gtk::Builder;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::RwLock;

use gstreamer_wrapper::GStreamerMessage;
use maingui::MainGui;
use playlist_tabs::PlaylistTabs;

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
pub type MainGuiPtr = Rc<MainGui>;
pub type MainGuiWeakPtr = Weak<MainGui>;
pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;
pub type PlaylistTabsPtr = Rc<RefCell<PlaylistTabs>>;

pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}

impl From<GStreamerMessage> for PlayerStatus {
    fn from(item: GStreamerMessage) -> Self {
        match item {
            GStreamerMessage::Pausing => PlayerStatus::Paused,
            GStreamerMessage::Stopped => PlayerStatus::Stopped,
            GStreamerMessage::Playing => PlayerStatus::Playing,
        }
    }
}
