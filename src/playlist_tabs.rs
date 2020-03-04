use owning_ref::RwLockReadGuardRef;
use std::sync::{Arc, RwLock};

use crate::loaded_playlist::{LoadedPlaylist, SavePlaylistExt};
use crate::playlist::restore_playlists;
use crate::types::*;

#[derive(Debug)]
pub struct PlaylistTabs {
    current_pl: usize,
    pls: Vec<LoadedPlaylistPtr>,
}

pub fn load(pool: &DBPool) -> Result<PlaylistTabsPtr, diesel::result::Error> {
    let pls = restore_playlists(pool)?;
    let converted_pls: Vec<LoadedPlaylistPtr> = pls
        .into_iter()
        .map(|pl| Arc::new(RwLock::new(pl)))
        .collect();
    Ok(Arc::new(RwLock::new(PlaylistTabs {
        current_pl: 0,
        pls: converted_pls,
    })))
}

pub trait PlaylistTabsExt {
    fn current(&self) -> Option<LoadedPlaylistPtr>;
    fn add(&self, _: LoadedPlaylistPtr);
}

impl PlaylistTabsExt for PlaylistTabsPtr {
    fn current(&self) -> Option<LoadedPlaylistPtr> {
        let i = self.read().unwrap().current_pl;
        self.read().unwrap().pls.get(i).cloned()
    }

    fn add(&self, lp: LoadedPlaylistPtr) {
        self.write().unwrap().current_pl = 0;
        self.write().unwrap().pls = vec![lp];
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
