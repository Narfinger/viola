use diesel::SqliteConnection;
use gstreamer::Element;
use gtk::Builder;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use std::sync::Arc;
use std::rc::{Rc, Weak};
use std::sync::RwLock;
use std::cell::RefCell;

use gui::Gui;
use playlist_tabs::PlaylistTabs;

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
