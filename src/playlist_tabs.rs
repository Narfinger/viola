use std::sync::RwLock;

use crate::loaded_playlist::SavePlaylistExt;
use crate::types::*;

#[derive(Debug)]
pub struct PlaylistTabs {
    current_playlist: RwLock<Option<usize>>,
    pls: Vec<LoadedPlaylistPtr>,
}

pub trait PlaylistTabsExt {
    fn current(&self) -> Option<&LoadedPlaylistPtr>;
}

impl PlaylistTabsExt for PlaylistTabsPtr {
    fn current(&self) -> Option<&LoadedPlaylistPtr> {
        let index = *self.read().unwrap().current_playlist.read().unwrap();
        if let Some(i) = index {
            self.read().unwrap().pls.get(i)
        } else {
            None
        }
    }
}

impl SavePlaylistExt for PlaylistTabsPtr {
    fn save(&self, db: &diesel::SqliteConnection) -> Result<(), diesel::result::Error> {
        for i in self.read().unwrap().pls.iter() {
            i.save(db)?;
        }
        Ok(())
    }
}
