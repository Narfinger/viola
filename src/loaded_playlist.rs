use owning_ref::RwLockReadGuardRef;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS, NON_ALPHANUMERIC};
use std::ops::Deref;
use std::path::PathBuf;

use crate::db::Track;
use crate::playlist::{NewPlaylist, NewPlaylistTrack, Playlist};
use crate::types::{DBPool, LoadedPlaylistPtr};
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ');

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
    //fn items(&self) -> RwLockReadGuardRef<LoadedPlaylist, Vec<Track>>;
    fn get_remaining_length(&self) -> u64;
    fn clean(&self);
}

pub trait SavePlaylistExt {
    fn save(&self, db: &diesel::SqliteConnection) -> Result<(), diesel::result::Error>;
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

    //fn items(&self) -> RwLockReadGuardRef<LoadedPlaylist, Vec<Track>> {
    //    println!("This is really inefficient");
    //    RwLockReadGuardRef::new(self.read().unwrap()).map(|s| &s.items)
    //}

    fn get_remaining_length(&self) -> u64 {
        let current_position = self.current_position();
        self.read()
            .unwrap()
            .items
            .iter()
            .skip(current_position)
            .map(|t| t.length)
            .sum::<i32>() as u64
    }

    fn clean(&self) {
        let index = self.current_position();
        let mut s = self.write().unwrap();
        s.items = s.items.split_off(index);
        s.current_position = 0;
    }
}

impl SavePlaylistExt for LoadedPlaylistPtr {
    fn save(&self, db: &diesel::SqliteConnection) -> Result<(), diesel::result::Error> {
        use crate::schema::playlists::dsl::*;
        use crate::schema::playlisttracks::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

        let pl = self.read().expect("Could not read lock to save playlist");

        info!("playlist id {:?}", pl.id);

        let exists = diesel::select(diesel::dsl::exists(
            playlists.filter(crate::schema::playlists::id.eq(pl.id)),
        ))
        .get_result(db)
        .expect("Error in db");

        if exists {
            // the playlist is already in the database
            diesel::update(playlists.find(pl.id))
                .set(current_position.eq(pl.current_position as i32))
                .execute(db)?;
        }

        let playlist: Playlist = if exists {
            playlists.find(pl.id).first::<Playlist>(db)?
        } else {
            let t = vec![NewPlaylist {
                id: pl.id,
                name: pl.name.clone(),
                current_position: pl.current_position as i32,
            }];
            diesel::insert_into(playlists).values(&t).execute(db)?;
            playlists.filter(name.eq(&pl.name)).first(db)?
        };

        //deleting old tracks
        diesel::delete(playlisttracks)
            .filter(playlist_id.eq(playlist.id))
            .execute(db)?;

        //inserting new tracks
        info!("starting to gather");
        let vals = pl
            .items
            .iter()
            .enumerate()
            .map(|(index, track)| NewPlaylistTrack {
                playlist_id: playlist.id,
                track_id: track.id,
                playlist_order: index as i32,
            })
            .collect::<Vec<NewPlaylistTrack>>();
        info!("collected and inserting");
        //info!("All values {:?}", vals);
        diesel::insert_into(playlisttracks)
            .values(&vals)
            .execute(db)?;

        info!("done");

        Ok(())
    }
}

pub trait PlaylistControls {
    fn get_current_path(&self) -> PathBuf;
    fn get_current_uri(&self) -> String;
    fn previous(&self) -> Option<usize>;
    fn set(&self, _: usize) -> usize;
    fn next_or_eol(&self) -> Option<usize>;
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
            utf8_percent_encode(&s.items[s.current_position].path, FRAGMENT).to_string()
        )
    }

    fn previous(&self) -> Option<usize> {
        let mut s = self.write().unwrap();
        let checked_res = s.current_position.checked_sub(1);
        if let Some(i) = checked_res {
            s.current_position = i;
        } else {
            s.current_position = 0;
        }
        checked_res
    }

    fn set(&self, i: usize) -> usize {
        let mut s = self.write().unwrap();
        s.current_position = i as usize;
        s.current_position as usize
    }

    fn next_or_eol(&self) -> Option<usize> {
        let next_pos = {
            let mut s = self.write().unwrap();
            s.current_position += 1 % s.items.len();
            s.current_position
        };

        if next_pos != 0 {
            Some(next_pos)
        } else {
            None
        }
    }
}
