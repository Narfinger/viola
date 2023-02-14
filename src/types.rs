use crate::loaded_playlist::LoadedPlaylist;
use crate::playlist_tabs::PlaylistTabs;
use parking_lot::{Mutex, RwLock};
use serde::Deserialize;
use std::sync::Arc;

pub(crate) const URL: &str = "http://127.0.0.1:8080";
pub(crate) const SOCKETADDR: &str = "127.0.0.1:8080";

pub(crate) type DBPool = Arc<Mutex<diesel::SqliteConnection>>;
pub(crate) type PlaylistTabsPtr = Arc<RwLock<PlaylistTabs>>;
pub(crate) type LoadedPlaylistPtr = RwLock<LoadedPlaylist>;

#[derive(Debug, Deserialize)]
pub struct ChangePlaylistTabJson {
    pub index: usize,
}
