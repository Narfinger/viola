use log::info;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::Serialize;
use std::path::PathBuf;

use crate::playlist::{NewPlaylist, NewPlaylistTrack, Playlist};
use crate::types::LoadedPlaylistPtr;
use viola_common::Track;
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'#');

#[derive(Debug, Serialize)]
pub(crate) struct LoadedPlaylist {
    /// The id we have in the database for it. If none, means this was not yet saved
    pub id: i32,

    /// Name of the Playlist
    pub name: String,

    /// All the tracks in the playlist
    pub items: Vec<Track>,

    /// The current position of the playlist
    pub current_position: usize,
}

pub(crate) trait LoadedPlaylistExt {
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

pub trait SavePlaylistExt {
    fn save(&self, db: &mut diesel::SqliteConnection) -> Result<(), diesel::result::Error>;
}

pub(crate) fn items(
    pl: &LoadedPlaylistPtr,
) -> parking_lot::lock_api::MappedRwLockReadGuard<parking_lot::RawRwLock, Vec<Track>> {
    parking_lot::lock_api::RwLockReadGuard::<'_, parking_lot::RawRwLock, LoadedPlaylist>::map(
        pl.read(),
        |s| &s.items,
    )
}

impl LoadedPlaylistExt for LoadedPlaylistPtr {
    fn get_current_track(&self) -> Track {
        let s = self.read();
        s.items[s.current_position].clone()
    }

    fn get_playlist_full_time(&self) -> i64 {
        let s = self.read();
        s.items.iter().map(|t| i64::from(t.length)).sum()
    }

    fn current_position(&self) -> usize {
        self.read().current_position
    }

    fn get_remaining_length(&self) -> u64 {
        let current_position = self.current_position();
        self.read()
            .items
            .iter()
            .skip(current_position)
            .map(|t| t.length)
            .sum::<i32>() as u64
    }

    fn clean(&self) {
        let index = self.current_position();
        let mut s = self.write();
        s.items.drain(0..index);
        s.current_position = 0;
    }

    fn update_current_playcount(&self) {
        let index = self.current_position();
        let mut s = self.write();
        let item = s.items.get_mut(index).unwrap();
        item.playcount = Some(item.playcount.unwrap_or(0) + 1);
    }
}

impl SavePlaylistExt for LoadedPlaylistPtr {
    fn save(&self, db: &mut diesel::SqliteConnection) -> Result<(), diesel::result::Error> {
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        use viola_common::schema::playlists::dsl::*;
        use viola_common::schema::playlisttracks::dsl::*;

        let pl = self.read();

        info!("playlist id {:?}", pl.id);

        let exists = diesel::select(diesel::dsl::exists(
            playlists.filter(viola_common::schema::playlists::id.eq(pl.id)),
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

pub(crate) trait PlaylistControls {
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

impl PlaylistControls for LoadedPlaylistPtr {
    fn get_current_path(&self) -> Option<PathBuf> {
        let mut pb = PathBuf::new();
        let s = self.read();
        if let Some(t) = s.items.get(s.current_position) {
            pb.push(t.path.clone());
            Some(pb)
        } else {
            None
        }
    }

    fn get_current_uri(&self) -> Option<String> {
        let s = self.read();
        info!("loading from playlist with name: {}", s.name);
        s.items
            .get(s.current_position)
            .as_ref()
            .map(|p| format!("file:////{}", utf8_percent_encode(&p.path, FRAGMENT)))
    }

    fn previous(&self) -> Option<usize> {
        let mut s = self.write();
        let checked_res = s.current_position.checked_sub(1);
        if let Some(i) = checked_res {
            s.current_position = i;
        } else {
            s.current_position = 0;
        }
        checked_res
    }

    fn set(&self, i: usize) -> usize {
        let mut s = self.write();
        s.current_position = i;
        s.current_position
    }

    fn delete_range(&self, range: std::ops::Range<usize>) {
        let mut s = self.write();
        println!("removing with range: {:?}", &range);

        s.items.drain(range.start..=range.end);

        if s.current_position >= range.start && s.current_position <= range.end {
            s.current_position = 0;
        } else if s.current_position > range.end {
            s.current_position -= range.end - range.start;
        }
    }

    fn next_or_eol(&self) -> Option<usize> {
        if self.read().items.len() == 1 {
            None
        } else {
            let next_pos = {
                let mut s = self.write();
                s.current_position += 1 % (s.items.len() - 1);
                s.current_position
            };

            if next_pos == 0 {
                None
            } else {
                Some(next_pos)
            }
        }
    }
}
