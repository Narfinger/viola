use crate::loaded_playlist::LoadedPlaylist;
use crate::playlist_tabs::PlaylistTabs;
use gstreamer::Element;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

pub const LENGTH_COLUMN: i32 = 4;
pub const YEAR_COLUMN: i32 = 5;
pub const PLAYCOUNT_COLUMN: i32 = 7;
pub const COLOR_COLUMN: u32 = 8;
pub const URL: &str = "http://127.0.0.1:8080";
pub const SOCKETADDR: &str = "127.0.0.1:8080";

//pub type BuilderPtr = Arc<RwLock<Builder>>;
pub type GstreamerPipeline = Arc<RwLock<Element>>;
//pub type MainGuiPtr = Rc<MainGui>;
//pub type MainGuiWeakPtr = Weak<MainGui>;
pub type DBPool = Arc<Mutex<diesel::SqliteConnection>>;
//pub type PlaylistTabsPtr = Rc<RefCell<PlaylistTabs>>;
pub type PlaylistTabsPtr = Arc<RwLock<PlaylistTabs>>;
pub type LoadedPlaylistPtr = RwLock<LoadedPlaylist>;

#[derive(Debug, Deserialize)]
pub struct ChangePlaylistTabJson {
    pub index: usize,
}
