use parking_lot::RwLock;
use std::sync::Arc;
use std::{cmp::min, path::PathBuf};

use crate::loaded_playlist::{
    LoadedPlaylist, LoadedPlaylistExt, PlaylistControls, SavePlaylistExt,
};
use crate::playlist::restore_playlists;
use crate::preferences::Preferences;
use crate::types::*;
use preferences::PreferencesMap;

#[derive(Debug, Serialize)]
pub struct PlaylistTabs {
    current_pl: usize,
    pub current_playing_in: usize,
    pub pls: Vec<LoadedPlaylistPtr>,
}

pub fn load(pool: &DBPool) -> Result<PlaylistTabsPtr, diesel::result::Error> {
    let pls = restore_playlists(pool);
    if pls.is_empty() {
        //use crate::smartplaylist_parser::LoadSmartPlaylist;
        //pls.push(crate::smartplaylist_parser::construct_smartplaylists_from_config()[0].load(pool));
    }
    let converted_pls: Vec<LoadedPlaylistPtr> = pls.into_iter().map(RwLock::new).collect();
    let pls_struct = Arc::new(parking_lot::RwLock::new(PlaylistTabs {
        current_pl: 0,
        current_playing_in: 0,
        pls: converted_pls,
    }));
    pls_struct.restore_tab_position();
    Ok(pls_struct)
}

pub trait PlaylistTabsExt {
    fn add(&self, _: LoadedPlaylist);
    fn current<T>(&self, f: fn(&LoadedPlaylistPtr) -> T) -> T;
    fn delete(&self, _: &DBPool, _: usize);
    fn items_json(&self) -> String;
    fn items_for_json(&self, index: usize) -> String;
    fn current_tab(&self) -> usize;
    fn current_playing_in(&self) -> usize;
    fn update_current_playing_in(&self);
    fn set_tab(&self, index: usize);
    fn restore_tab_position(&self);
    fn save_tab_position(&self);
}

impl PlaylistTabsExt for PlaylistTabsPtr {
    fn add(&self, lp: LoadedPlaylist) {
        self.write().pls.push(RwLock::new(lp));
    }

    fn current<T>(&self, f: fn(&LoadedPlaylistPtr) -> T) -> T {
        let i = self.read().current_pl;
        f(self.as_ref().read().pls.get(i).as_ref().unwrap())
    }

    fn delete(&self, pool: &DBPool, index: usize) {
        let length = self.read().pls.len();
        let current_pl = self.read().current_pl;
        println!(
            "index {} | current {} | length {}",
            index, current_pl, length
        );
        if index < length {
            // delete in database
            use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
            use viola_common::schema::playlists::dsl::*;
            let lp = self.write().pls.swap_remove(index);
            if current_pl >= index {
                self.write().current_pl = 0;
            }
            let db = pool.lock();

            diesel::delete(playlists.filter(id.eq(lp.read().id)))
                .execute(&*db)
                .expect("Error deleting");
        }
    }

    fn items_json(&self) -> String {
        let i = self.read().current_pl;
        self.items_for_json(i)
    }

    fn items_for_json(&self, index: usize) -> String {
        let cur = self.read();
        let pl = cur.pls.get(index).unwrap();
        let items = crate::loaded_playlist::items(pl);
        serde_json::to_string::<Vec<viola_common::Track>>(items.as_ref())
            .expect("Error in serializing")
    }

    fn current_tab(&self) -> usize {
        self.read().current_pl
    }

    fn current_playing_in(&self) -> usize {
        self.read().current_playing_in
    }

    fn update_current_playing_in(&self) {
        let cur = self.read().current_pl;
        self.write().current_playing_in = cur;
    }

    fn set_tab(&self, index: usize) {
        let max = self.read().pls.len();
        self.write().current_pl = std::cmp::min(max - 1, index);
        self.save_tab_position();
    }

    fn restore_tab_position(&self) {
        let mut prefs_file =
            crate::utils::get_config_file(&crate::utils::ConfigWriteMode::Read).unwrap();
        //we need to split this because of how the allocation of the locks work
        let val = min(
            PreferencesMap::<String>::load_from(&mut prefs_file)
                .ok()
                .and_then(|m| m.get("tab").cloned())
                .and_then(|t| t.parse::<usize>().ok())
                .unwrap_or(0),
            self.read().pls.len() - 1,
        );
        println!("restored position {}", val);
        self.write().current_pl = val;
    }

    fn save_tab_position(&self) {
        info!("Saving tab position");
        let mut prefs = {
            let mut prefs_file =
                crate::utils::get_config_file(&crate::utils::ConfigWriteMode::Read).unwrap();
            PreferencesMap::<String>::load_from(&mut prefs_file).expect("Error in loading prefs")
        };
        prefs.insert(String::from("tab"), self.read().current_pl.to_string());
        let mut file_write =
            crate::utils::get_config_file(&crate::utils::ConfigWriteMode::Write).unwrap();
        prefs
            .save_to(&mut file_write)
            .expect("Error in writing prefs");
    }
}

impl LoadedPlaylistExt for PlaylistTabsPtr {
    fn get_current_track(&self) -> viola_common::Track {
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
        self.current(LoadedPlaylistExt::clean);
    }
}

impl PlaylistControls for PlaylistTabsPtr {
    fn get_current_path(&self) -> Option<PathBuf> {
        self.current(PlaylistControls::get_current_path)
    }

    fn get_current_uri(&self) -> Option<String> {
        self.current(PlaylistControls::get_current_uri)
    }

    fn previous(&self) -> Option<usize> {
        self.update_current_playing_in();
        self.current(PlaylistControls::previous)
    }

    fn set(&self, index: usize) -> usize {
        self.update_current_playing_in();

        let i = self.read().current_pl;
        let cur = self.read();
        let value = cur.pls.get(i).unwrap();
        value.set(index)
    }

    fn delete_range(&self, range: std::ops::Range<usize>) {
        let i = self.read().current_pl;
        let cur = self.read();
        let value = cur.pls.get(i).unwrap();
        value.delete_range(range);
    }

    fn next_or_eol(&self) -> Option<usize> {
        self.update_current_playing_in();
        self.current(PlaylistControls::next_or_eol)
    }
}

impl SavePlaylistExt for PlaylistTabsPtr {
    fn save(&self, db: &diesel::SqliteConnection) -> Result<(), diesel::result::Error> {
        for i in &self.read().pls {
            i.save(db)?;
        }
        info!("Saved all playlists");
        Ok(())
    }
}
