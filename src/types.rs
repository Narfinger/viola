use diesel::SqliteConnection;
use gstreamer::Element;
use gtk::Builder;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use std::sync::Arc;
use std::rc::Rc;
use std::sync::RwLock;

use playlist::LoadedPlaylist;
use playlistmanager::PlaylistManager;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub type CurrentPlaylist = Arc<RwLock<LoadedPlaylist>>;
pub type Pipeline = Arc<RwLock<Element>>;
pub type GuiPtr = Arc<RwLock<Builder>>;
pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;
pub type PlaylistManagerPtr = Arc<RwLock<PlaylistManager>>;

pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}

/// Tells the GuiPtr and the gstreamer what action is performed. Splits the GuiPtr and the backend a tiny bit
#[derive(Debug, Eq, PartialEq)]
pub enum GStreamerAction {
    Next,
    Playing,
    Pausing,
    Previous,
    /// This means we selected one specific track
    Play(i32),
}
