use std::sync::RwLock;

use crate::types::*;

#[derive(Debug)]
pub struct PlaylistTabs {
    pub current_playlist: RwLock<Option<usize>>,
    pub pls: Vec<LoadedPlaylistPtr>,
}

trait PlaylistTabsExt {
    fn current(&self) -> Option<&LoadedPlaylistPtr>;
}

impl PlaylistTabsExt for PlaylistTabs {
    fn current(&self) -> Option<&LoadedPlaylistPtr> {
        let index = *self.current_playlist.read().unwrap();
        if let Some(i) = index {
            self.pls.get(i)
        } else {
            None
        }
    }
}
