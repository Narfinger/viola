use std::sync::Arc;
use std::sync::RwLock;
use diesel::SqliteConnection;
use gstreamer::Element;
use gtk::Builder;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;


use playlist::Playlist;

pub type CurrentPlaylist = Arc<RwLock<Playlist>>;
pub type Pipeline = Arc<RwLock<Element>>;
pub type Gui = Arc<RwLock<Builder>>;
pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;