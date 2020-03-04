use owning_ref::{RwLockReadGuardRef, VecRef};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::loaded_playlist::{
    LoadedPlaylist, LoadedPlaylistExt, PlaylistControls, SavePlaylistExt,
};
use crate::playlist::restore_playlists;
use crate::types::*;

#[derive(Debug, Serialize)]
pub struct PlaylistTabs {
    current_pl: usize,
    pls: Vec<LoadedPlaylistPtr>,
}

pub fn load(pool: &DBPool) -> Result<PlaylistTabsPtr, diesel::result::Error> {
    let pls = restore_playlists(pool)?;
    let converted_pls: Vec<LoadedPlaylistPtr> = pls.into_iter().map(|pl| RwLock::new(pl)).collect();
    Ok(Arc::new(RwLock::new(PlaylistTabs {
        current_pl: 0,
        pls: converted_pls,
    })))
}

pub trait PlaylistTabsExt {
    fn add(&self, _: LoadedPlaylistPtr);
    fn current<T>(&self, f: fn(&LoadedPlaylistPtr) -> T) -> T;
}

impl PlaylistTabsExt for PlaylistTabsPtr {
    fn add(&self, lp: LoadedPlaylistPtr) {
        self.write().unwrap().current_pl = 0;
        self.write().unwrap().pls = vec![lp];
    }

    fn current<T>(&self, f: fn(&LoadedPlaylistPtr) -> T) -> T {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let pl = cur.pls.get(i).unwrap();
        f(&pl)
    }
}

impl LoadedPlaylistExt for PlaylistTabsPtr {
    fn get_current_track(&self) -> crate::db::Track {
        self.current(LoadedPlaylistExt::get_current_track)
        //value.get_current_track()
    }

    fn get_playlist_full_time(&self) -> i64 {
        self.current(LoadedPlaylistExt::get_playlist_full_time)
    }

    fn current_position(&self) -> usize {
        self.current(LoadedPlaylistExt::current_position)
    }

    fn items(&self) -> Vec<crate::db::Track> {
        self.current(LoadedPlaylistExt::items)
    }

    fn get_remaining_length(&self) -> u64 {
        self.current(LoadedPlaylistExt::get_remaining_length)
    }

    fn clean(&self) {
        self.current(LoadedPlaylistExt::clean)
    }
}

impl PlaylistControls for PlaylistTabsPtr {
    fn get_current_path(&self) -> PathBuf {
        self.current(PlaylistControls::get_current_path)
    }

    fn get_current_uri(&self) -> String {
        self.current(PlaylistControls::get_current_uri)
    }

    fn previous(&self) -> Option<usize> {
        self.current(PlaylistControls::previous)
    }

    fn set(&self, i: usize) -> usize {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.set(i)
    }

    fn next_or_eol(&self) -> Option<usize> {
        self.current(PLaylistControls::next_or_eol)
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
