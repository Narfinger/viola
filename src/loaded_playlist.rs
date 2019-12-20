use gtk;
use owning_ref::RwLockReadGuardRef;
use std::ops::Deref;
use std::path::PathBuf;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

use crate::db::Track;
use crate::types::{DBPool, LoadedPlaylistPtr};

#[derive(Clone, Debug)]
pub struct LoadedPlaylist {
    /// The id we have in the database for it. If none, means this was not yet saved
    pub id: i32,
    pub name: String,
    pub items: Vec<Track>,
    pub current_position: usize,
}

pub trait LoadedPlaylistExt {
    fn get_current_track(&self) -> Track;
    fn get_playlist_full_time(&self) -> i64;
    fn current_position(&self) -> usize;
    fn items(&self) -> RwLockReadGuardRef<LoadedPlaylist, Vec<Track>>;
    fn clean(&self);
}

impl LoadedPlaylistExt for LoadedPlaylistPtr {
    fn get_current_track(&self) -> Track {
        let s = self.read().unwrap();
        s.items[s.current_position].clone()
    }

    fn get_playlist_full_time(&self) -> i64 {
        let s = self.read().unwrap();
        s.items.iter().map(|t| t.length as i64).sum()
    }

    fn current_position(&self) -> usize {
        self.read().unwrap().current_position
    }

    fn items(&self) -> RwLockReadGuardRef<LoadedPlaylist, Vec<Track>> {
        println!("This is really inefficient");
        RwLockReadGuardRef::new(self.read().unwrap()).map(|s| &s.items)
    }

    fn clean(&self) {
        let index = self.current_position();
        let mut s = self.write().unwrap();
        s.items = s.items.split_off(index);
        s.current_position = 1;
    }
}

pub trait PlaylistControls {
    fn get_current_path(&self) -> PathBuf;
    fn get_current_uri(&self) -> String;
    fn previous(&self) -> String;
    fn next(&self) -> String;
    fn set(&self, _: i32) -> String;
    fn next_or_eol(&self, _: &DBPool) -> Option<String>;
}

impl PlaylistControls for LoadedPlaylistPtr {
    fn get_current_path(&self) -> PathBuf {
        let mut pb = PathBuf::new();
        let s = self.read().unwrap();
        pb.push(&s.items[s.current_position].path);
        pb
    }

    fn get_current_uri(&self) -> String {
        let s = self.read().unwrap();
        info!("loading from playlist with name: {}", s.name);
        format!(
            "file:////{}",
            utf8_percent_encode(&s.items[s.current_position].path, DEFAULT_ENCODE_SET).to_string()
        )
    }

    fn previous(&self) -> String {
        {
            let mut s = self.write().unwrap();
            s.current_position -= 1;
        }
        self.get_current_uri()
    }

    fn next(&self) -> String {
        {
            let mut s = self.write().unwrap();
            s.current_position += 1;
        }
        self.get_current_uri()
    }

    fn set(&self, i: i32) -> String {
        {
            let mut s = self.write().unwrap();
            s.current_position = i as usize;
        }
        self.get_current_uri()
    }

    fn next_or_eol(&self, pool: &DBPool) -> Option<String> {
        let mut s = self.write().unwrap();
        {
            let track = s.items.get(s.current_position);
            //update playlist counter
            let dbc = pool.clone();
            if let Some(t) = track {
                let id = t.id;
                gtk::idle_add(move || update_playcount(id, &dbc));
            }
        }

        s.current_position += 1;
        let next_pos = s.current_position;

        if s.items.get(next_pos as usize).is_some() {
            Some(self.next())
        } else {
            s.current_position = 0;
            None
        }
    }
}

fn update_playcount(t_id: i32, db: &DBPool) -> gtk::Continue {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SaveChangesDsl};

    if let Ok(mut track) = tracks
        .filter(id.eq(t_id))
        .first::<Track>(db.lock().expect("DB Error").deref())
    {
        track.playcount = Some(1 + track.playcount.unwrap_or(0));
        if track
            .save_changes::<Track>(db.lock().expect("DB Error").deref())
            .is_err()
        {
            error!("Some problem with updating play status (cannot update)");
        }
    } else {
        error!("Some problem with updating play status (gettin track)");
    }
    gtk::Continue(false)
}
