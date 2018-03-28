use std::sync::Arc;
use std::sync::RwLock;
use diesel::SqliteConnection;
use gstreamer::Element;
use gtk::Builder;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;


use playlist::LoadedPlaylist;

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
pub type Gui = Arc<RwLock<Builder>>;
pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;

pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped
}

/// Tells the gui and the gstreamer what action is performed. Splits the gui and the backend a tiny bit
#[derive(Debug, Eq, PartialEq)]
pub enum GStreamerAction {
    Next,
    Playing,
    Pausing,
    Previous,
    /// This means we selected one specific track
    Play(i32),
}
