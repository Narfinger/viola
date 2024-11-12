use log::info;
use parking_lot::RwLock;
use serde::Serialize;
use viola_common::Track;
use std::ops::DerefMut;
use std::sync::Arc;
use std::{cmp::min, path::PathBuf};

use crate::loaded_playlist::{
    LoadedPlaylist, LoadedPlaylistExt, PlaylistControls, SavePlaylistExt,
};
use crate::playlist::restore_playlists;
use crate::types::*;
use preferences::Preferences;
use preferences::PreferencesMap;

/// Holding all playlisttabs
#[derive(Debug, Serialize)]
pub(crate) struct PlaylistTabs {
    current_pl: usize,
    current_playing_in: usize,
    pub(crate) pls: Vec<LoadedPlaylistPtr>,
}

/// load the playlisttabs from the database
pub(crate) fn load(pool: &DBPool) -> Result<PlaylistTabsPtr, diesel::result::Error> {
    let pls = restore_playlists(pool);
    if pls.is_empty() {
        //use crate::smartplaylist_parser::LoadSmartPlaylist;
        //pls.push(crate::smartplaylist_parser::construct_smartplaylists_from_config()[0].load(pool));
    }
    let converted_pls: Vec<LoadedPlaylistPtr> = pls.into_iter().collect();
    let pls_struct = Arc::new(parking_lot::RwLock::new(PlaylistTabs {
        current_pl: 0,
        current_playing_in: 0,
        pls: converted_pls,
    }));
    pls_struct.restore_tab_position();
    Ok(pls_struct)
}

pub(crate) trait PlaylistTabsExt {
    /// Add a loaded playlists to the tab structure
    fn add(&self, _: LoadedPlaylist);
    /// execute f on the current playlist
    fn current<T>(&self, f: fn(&LoadedPlaylistPtr) -> T) -> T;
    /// same but for mut
    fn current_mut<T>(&self, f: fn(&mut LoadedPlaylist) -> T) -> T;
    /// delete the playlist given by item
    fn delete(&self, _: &DBPool, _: usize);
    /// produces the json string corresponding to the items
    fn items_json(&self) -> String;
    fn items_for_json(&self, index: usize) -> String;
    /// gives the current selected tab
    fn current_tab(&self) -> usize;
    /// gives the tab we are currently playing a track in
    fn current_playing_in(&self) -> usize;
    /// update the current playing track in
    fn update_current_playing_in(&self);
    /// set the current tab to index
    fn set_tab(&self, index: usize);
    /// restore the current selected tab (the index) from the database
    fn restore_tab_position(&self);
    /// save the current selected tab (the index) to the database
    fn save_tab_position(&self);
    ///
    fn update_current_playcount(&self);
}

impl PlaylistTabsExt for PlaylistTabsPtr {
    fn add(&self, lp: LoadedPlaylist) {
        self.write().pls.push(lp);
    }

    fn current<T>(&self, f: fn(&LoadedPlaylist) -> T) -> T {
        let i = self.read().current_pl;
        f(self.as_ref().read().pls.get(i).as_ref().unwrap())
    }

    fn current_mut<T>(&self, f: fn(&mut LoadedPlaylist) -> T) -> T {
        let i = self.read().current_pl;
        f(self.as_ref().write().pls.get_mut(i).unwrap())
    }

    fn delete(&self, pool: &DBPool, index: usize) {
        let length = self.read().pls.len();
        let current_pl = self.read().current_pl;
        info!(
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
            let mut db = pool.lock();

            diesel::delete(playlists.filter(id.eq(lp.id)))
                .execute(db.deref_mut())
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
        let items = &pl.items;
        serde_json::to_string::<Vec<viola_common::Track>>(items)
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
        info!("restored position {}", val);
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

    fn update_current_playcount(&self) {
        let index = self.read().current_pl;
        self.write()
            .pls
            .get_mut(index)
            .unwrap()
            .update_current_playcount();
    }
}

pub(crate) trait LoadedPlaylistExtImut {
    /// Returns the current track
    fn get_current_track(&self) -> Track;

    /// get the added time of the whole playlist
    fn get_playlist_full_time(&self) -> i64;

    /// returns the raw current_position
    fn current_position(&self) -> usize;

    /// get the remaining length, ignoring already played tracks and the current playling track
    fn get_remaining_length(&self) -> u64;

    /// removes all tracks that are smaller than the current position
    fn clean(&self);

    /// update the current playcount only in the datastructure, not in the current database
    fn update_current_playcount(&self);
}

impl LoadedPlaylistExtImut for PlaylistTabsPtr {
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
        self.current_mut(LoadedPlaylistExt::clean);
    }

    fn update_current_playcount(&self) {
        self.current_mut(LoadedPlaylistExt::update_current_playcount);
    }
}

/// ways to control the playlist (does not control the gstreamer)
pub(crate) trait PlaylistControlsImut {
    /// Get current track path
    fn get_current_path(&self) -> Option<PathBuf>;
    /// Get current track uri
    fn get_current_uri(&self) -> Option<String>;
    /// Get previous position, wraps to zero
    fn previous(&self) -> Option<usize>;
    /// set the current position and return it
    fn set(&self, _: usize) -> usize;
    /// delete the tracks in range where the range is inclusive
    fn delete_range(&self, _: std::ops::Range<usize>);
    /// sets the position to the next one or zero if we are eol. Returns None if we are eol otherwise the position.
    fn next_or_eol(&self) -> Option<usize>;
}

impl PlaylistControlsImut for PlaylistTabsPtr {
    fn get_current_path(&self) -> Option<PathBuf> {
        self.current(PlaylistControls::get_current_path)
    }

    fn get_current_uri(&self) -> Option<String> {
        self.current(PlaylistControls::get_current_uri)
    }

    fn previous(&self) -> Option<usize> {
        self.update_current_playing_in();
        self.current_mut(PlaylistControls::previous)
    }

    fn set(&self, index: usize) -> usize {
        self.update_current_playing_in();
        let i = self.read().current_pl;
        self.write().pls.get_mut(i).unwrap().set(index)
    }

    fn delete_range(&self, range: std::ops::Range<usize>) {
        let i = self.read().current_pl;
        self.write().pls.get_mut(i).unwrap().delete_range(range);
    }

    fn next_or_eol(&self) -> Option<usize> {
        self.update_current_playing_in();
        self.current_mut(PlaylistControls::next_or_eol)
    }
}

impl SavePlaylistExt for PlaylistTabsPtr {
    fn save(&self, db: &mut diesel::SqliteConnection) -> Result<(), diesel::result::Error> {
        for i in &self.read().pls {
            i.save(db)?;
        }
        info!("Saved all playlists");
        Ok(())
    }
}
