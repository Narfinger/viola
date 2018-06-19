use app_dirs::*;
use diesel::SqliteConnection;
use gstreamer::Element;
use gtk::Builder;
use diesel::r2d2::{Pool, ConnectionManager};
use std::sync::Arc;
use std::rc::{Rc, Weak};
use std::sync::RwLock;
use std::cell::RefCell;

use gui::Gui;
use playlist_tabs::PlaylistTabs;
use gstreamer_wrapper::GStreamerMessage;

pub const APP_INFO: AppInfo = AppInfo{name: "viola", author: "Narfinger"};


pub type BuilderPtr = Arc<RwLock<Builder>>;
pub type GstreamerPipeline = Arc<RwLock<Element>>;
pub type GuiPtr = Rc<Gui>;
pub type GuiWeakPtr = Weak<Gui>;
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