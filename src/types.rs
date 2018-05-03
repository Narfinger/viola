use diesel::SqliteConnection;
use gstreamer::Element;
use gtk::Builder;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use std::sync::Arc;
use std::rc::Rc;
use std::sync::RwLock;
use std::cell::RefCell;

use gui::Gui;
use loaded_playlist::LoadedPlaylist;
use playlist_tabs::PlaylistTabs;

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

pub type BuilderPtr = Arc<RwLock<Builder>>;
//pub type CurrentPlaylist = Arc<RwLock<LoadedPlaylist>>;
pub type GstreamerPipeline = Arc<RwLock<Element>>;
pub type GuiPtr = Rc<Gui>;
pub type DBPool = Pool<ConnectionManager<SqliteConnection>>;
pub type PlaylistTabsPtr = Rc<RefCell<PlaylistTabs>>;

pub enum PlayerStatus {
    Playing,
    Paused,
    Stopped,
}
