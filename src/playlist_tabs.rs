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
    fn items(&self) -> String;
}

impl PlaylistTabsExt for PlaylistTabsPtr {
    fn add(&self, lp: LoadedPlaylistPtr) {
        self.write().unwrap().current_pl = 0;
        self.write().unwrap().pls = vec![lp];
    }
}

impl LoadedPlaylistExt for PlaylistTabsPtr {
    fn get_current_track(&self) -> crate::db::Track {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.get_current_track()
    }

    fn get_playlist_full_time(&self) -> i64 {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.get_playlist_full_time()
    }

    fn current_position(&self) -> usize {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.current_position()
    }

    //fn items(&self) -> RwLockReadGuardRef<LoadedPlaylist, Vec<crate::db::Track>> {
    //    let i = self.read().unwrap().current_pl;
    //    let cur = self.read().unwrap();
    //    let value = cur.pls.get(i).unwrap();
    //    println!("this is inefficient");
    //    value.items().cloned()
    //}

    fn get_remaining_length(&self) -> u64 {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.get_remaining_length()
    }

    fn clean(&self) {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.clean();
    }
}

impl PlaylistControls for PlaylistTabsPtr {
    fn get_current_path(&self) -> PathBuf {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.get_current_path()
    }

    fn get_current_uri(&self) -> String {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.get_current_uri()
    }

    fn previous(&self) -> Option<usize> {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.previous()
    }

    fn set(&self, i: usize) -> usize {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.set(i)
    }

    fn next_or_eol(&self) -> Option<usize> {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.next_or_eol()
    }
}

struct PlaylistTabsSerializer {
    index: usize,
    
}

impl SavePlaylistExt for PlaylistTabsPtr {
    fn save(&self, db: &diesel::SqliteConnection) -> Result<(), diesel::result::Error> {
        for i in self.read().unwrap().pls.iter() {
            i.save(db)?;
        }
        Ok(())
    }

    fn items(&self) -> String {
        serde_json::serialize(
    }
}
