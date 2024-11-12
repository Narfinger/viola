use log::info;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::Serialize;
use std::path::PathBuf;

use crate::playlist::{NewPlaylist, NewPlaylistTrack, Playlist};
use crate::types::LoadedPlaylistPtr;
use viola_common::Track;
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'#');

#[derive(Debug, Serialize)]
/// A loaded playlist
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
    fn clean(&mut self);

    /// update the current playcount only in the datastructure, not in the current database
    fn update_current_playcount(&mut self);
}

pub trait SavePlaylistExt {
    fn save(&self, db: &mut diesel::SqliteConnection) -> Result<(), diesel::result::Error>;
}

impl LoadedPlaylistExt for LoadedPlaylistPtr {
    fn get_current_track(&self) -> Track {
        self.items[self.current_position].clone()
    }

    fn get_playlist_full_time(&self) -> i64 {
        self.items.iter().map(|t| i64::from(t.length)).sum()
    }

    fn current_position(&self) -> usize {
        self.current_position
    }

    fn get_remaining_length(&self) -> u64 {
        let current_position = self.current_position();
        self
            .items
            .iter()
            .skip(current_position)
            .map(|t| t.length)
            .sum::<i32>() as u64
    }

    fn clean(&mut self) {
        let index = self.current_position();
        self.items.drain(0..index);
        self.current_position = 0;
    }

    fn update_current_playcount(&mut self) {
        let index = self.current_position();
        let item = self.items.get_mut(index).unwrap();
        item.playcount = Some(item.playcount.unwrap_or(0) + 1);
    }
}

impl SavePlaylistExt for LoadedPlaylistPtr {
    fn save(&self, db: &mut diesel::SqliteConnection) -> Result<(), diesel::result::Error> {
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        use viola_common::schema::playlists::dsl::*;
        use viola_common::schema::playlisttracks::dsl::*;

        info!("playlist id {:?}", self.id);

        let exists = diesel::select(diesel::dsl::exists(
            playlists.filter(viola_common::schema::playlists::id.eq(self.id)),
        ))
        .get_result(db)
        .expect("Error in db");

        if exists {
            // the playlist is already in the database
            diesel::update(playlists.find(self.id))
                .set(current_position.eq(self.current_position as i32))
                .execute(db)?;
        }

        let playlist: Playlist = if exists {
            playlists.find(self.id).first::<Playlist>(db)?
        } else {
            let t = vec![NewPlaylist {
                id: self.id,
                name: self.name.clone(),
                current_position: self.current_position as i32,
            }];
            diesel::insert_into(playlists).values(&t).execute(db)?;
            playlists.filter(name.eq(&self.name)).first(db)?
        };

        //deleting old tracks
        diesel::delete(playlisttracks)
            .filter(playlist_id.eq(playlist.id))
            .execute(db)?;

        //inserting new tracks
        info!("starting to gather");
        let vals = self
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

/// ways to control the playlist (does not control the gstreamer)
pub(crate) trait PlaylistControls {
    /// Get current track path
    fn get_current_path(&self) -> Option<PathBuf>;
    /// Get current track uri
    fn get_current_uri(&self) -> Option<String>;
    /// Get previous position, wraps to zero
    fn previous(&mut self) -> Option<usize>;
    /// set the current position and return it
    fn set(&mut self, _: usize) -> usize;
    /// delete the tracks in range where the range is inclusive
    fn delete_range(&mut self, _: std::ops::Range<usize>);
    /// sets the position to the next one or zero if we are eol. Returns None if we are eol otherwise the position.
    fn next_or_eol(&mut self) -> Option<usize>;
}

impl PlaylistControls for LoadedPlaylistPtr {
    fn get_current_path(&self) -> Option<PathBuf> {
        let mut pb = PathBuf::new();
        if let Some(t) = self.items.get(self.current_position) {
            pb.push(t.path.clone());
            Some(pb)
        } else {
            None
        }
    }

    fn get_current_uri(&self) -> Option<String> {
        info!("loading from playlist with name: {}", self.name);
        self.items
            .get(self.current_position)
            .as_ref()
            .map(|p| format!("file:////{}", utf8_percent_encode(&p.path, FRAGMENT)))
    }

    fn previous(&mut self) -> Option<usize> {
        let checked_res = self.current_position.checked_sub(1);
        if let Some(i) = checked_res {
            self.current_position = i;
        } else {
            self.current_position = 0;
        }
        checked_res
    }

    fn set(&mut self, i: usize) -> usize {
        self.current_position = i;
        self.current_position
    }

    fn delete_range(&mut self, range: std::ops::Range<usize>) {
        println!("removing with range: {:?}", &range);

        self.items.drain(range.start..=range.end);

        if self.current_position >= range.start && self.current_position <= range.end {
            self.current_position = 0;
        } else if self.current_position > range.end {
            self.current_position -= range.end - range.start;
        }
    }

    fn next_or_eol(&mut self) -> Option<usize> {
        if self.items.len() == 1 {
            None
        } else {
            let next_pos = {
                self.current_position += 1 % (self.items.len() - 1);
                self.current_position
            };

            if next_pos == 0 {
                None
            } else {
                Some(next_pos)
            }
        }
    }
}
