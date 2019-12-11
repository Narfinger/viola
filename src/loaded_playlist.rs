use gtk;
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

use crate::db::Track;
use crate::types::DBPool;

#[derive(Clone, Debug)]
pub struct LoadedPlaylist {
    /// The id we have in the database for it. If none, means this was not yet saved
    pub id: i32,
    pub name: String,
    pub items: Vec<Track>,
    pub current_position: Arc<AtomicUsize>,
}

pub trait LoadedPlaylistExt {
    fn get_current_track(&self) -> &Track;
    fn get_playlist_full_time(&self) -> i64;
}

impl LoadedPlaylistExt for LoadedPlaylist {
    fn get_current_track(&self) -> &Track {
        &self.items[self.current_position.load(Ordering::Relaxed)]
    }

    fn get_playlist_full_time(&self) -> i64 {
        self.items.iter().map(|t| t.length as i64).sum()
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

impl PlaylistControls for LoadedPlaylist {
    fn get_current_path(&self) -> PathBuf {
        let mut pb = PathBuf::new();
        pb.push(&self.items[self.current_position.load(Ordering::Relaxed)].path);
        pb
    }

    fn get_current_uri(&self) -> String {
        info!("loading from playlist with name: {}", self.name);
        format!(
            "file:////{}",
            utf8_percent_encode(
                &self.items[self.current_position.load(Ordering::Relaxed) as usize].path,
                DEFAULT_ENCODE_SET
            )
            .to_string()
        )
    }

    fn previous(&self) -> String {
        self.current_position.fetch_sub(1, Ordering::Relaxed);
        self.get_current_uri()
    }

    fn next(&self) -> String {
        self.current_position.fetch_add(1, Ordering::Relaxed);
        self.get_current_uri()
    }

    fn set(&self, i: i32) -> String {
        self.current_position.swap(i as usize, Ordering::Relaxed);
        self.get_current_uri()
    }

    fn next_or_eol(&self, pool: &DBPool) -> Option<String> {
        {
            let track = self
                .items
                .get(self.current_position.load(Ordering::Relaxed));
            //update playlist counter
            let dbc = pool.clone();
            if let Some(t) = track {
                let id = t.id;
                gtk::idle_add(move || update_playcount(id, &dbc));
            }
        }

        let next_pos = self.current_position.fetch_add(1, Ordering::Relaxed);

        if self.items.get(next_pos as usize).is_some() {
            Some(self.next())
        } else {
            self.current_position.swap(0, Ordering::Relaxed);
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
