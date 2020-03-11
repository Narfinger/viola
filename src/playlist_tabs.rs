use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::loaded_playlist::{
    LoadedPlaylist, LoadedPlaylistExt, PlaylistControls, SavePlaylistExt,
};
use crate::playlist::restore_playlists;
use crate::types::*;

#[derive(Debug, Serialize)]
pub struct PlaylistTabs {
    pub current_pl: usize,
    pub pls: Vec<LoadedPlaylistPtr>,
}

pub fn load(pool: &DBPool) -> Result<PlaylistTabsPtr, diesel::result::Error> {
    let pls = restore_playlists(pool)?;
    let converted_pls: Vec<LoadedPlaylistPtr> = pls.into_iter().map(RwLock::new).collect();
    Ok(Arc::new(RwLock::new(PlaylistTabs {
        current_pl: 0,
        pls: converted_pls,
    })))
}

pub trait PlaylistTabsExt {
    fn add(&self, _: LoadedPlaylist);
    fn current<T>(&self, f: fn(&LoadedPlaylistPtr) -> T) -> T;
    fn delete(&self, _: &DBPool, _: usize);
    fn items(&self) -> String;
    fn current_tab(&self) -> usize;
}

impl PlaylistTabsExt for PlaylistTabsPtr {
    fn add(&self, lp: LoadedPlaylist) {
        self.write().unwrap().pls.push(RwLock::new(lp));
    }

    fn current<T>(&self, f: fn(&LoadedPlaylistPtr) -> T) -> T {
        let i = self.read().as_ref().unwrap().current_pl;
        f(self.as_ref().read().unwrap().pls.get(i).as_ref().unwrap())
    }

    fn delete(&self, pool: &DBPool, index: usize) {
        let length = self.read().unwrap().pls.len();
        let current_pl = self.read().unwrap().current_pl;
        println!(
            "index {} | current {} | length {}",
            index, current_pl, length
        );
        if index < length {
            let lp = self.write().unwrap().pls.swap_remove(index);
            if current_pl >= index {
                self.write().unwrap().current_pl = 0;
            }

            // delete in database
            use crate::schema::playlists::dsl::*;
            use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
            use std::ops::Deref;
            let db = pool.lock().expect("DB Error");

            diesel::delete(playlists.filter(id.eq(lp.read().unwrap().id)))
                .execute(db.deref())
                .expect("Error deleting");
        }
    }

    fn items(&self) -> String {
        use crate::loaded_playlist::items;
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let pl = cur.pls.get(i).unwrap();
        let items = items(&pl);
        serde_json::to_string(&*items).unwrap()
    }

    fn current_tab(&self) -> usize {
        self.read().unwrap().current_pl
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

    //fn items(&self) -> RwLockReadGuardRef<LoadedPlaylist, Vec<crate::db::Track>> {
    //    self.current(LoadedPlaylistExt::items)
    //}

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

    fn set(&self, index: usize) -> usize {
        let i = self.read().unwrap().current_pl;
        let cur = self.read().unwrap();
        let value = cur.pls.get(i).unwrap();
        value.set(index)
    }

    fn next_or_eol(&self) -> Option<usize> {
        self.current(PlaylistControls::next_or_eol)
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
