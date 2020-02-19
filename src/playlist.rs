use crate::schema::{playlists, playlisttracks};
use diesel;
use std::ops::Deref;

use crate::db::Track;
use crate::loaded_playlist::LoadedPlaylist;
use crate::types::DBPool;

#[derive(Identifiable, Queryable, Associations)]
pub struct Playlist {
    pub id: i32,
    pub name: String,
    pub current_position: i32,
}

#[derive(Insertable)]
#[table_name = "playlists"]
pub struct NewPlaylist {
    pub id: i32,
    pub name: String,
    pub current_position: i32,
}

#[derive(Identifiable, Queryable, Associations)]
#[table_name = "playlisttracks"]
#[belongs_to(Track, foreign_key = "playlist_id")]
#[belongs_to(Playlist, foreign_key = "track_id")]
pub struct PlaylistTrack {
    id: i32,
    playlist_id: i32,
    track_id: i32,
    playlist_order: i32,
}

#[derive(Debug, Insertable, Associations)]
#[table_name = "playlisttracks"]
pub struct NewPlaylistTrack {
    pub playlist_id: i32,
    pub track_id: i32,
    pub playlist_order: i32,
}

fn get_ordering(&(ref t, ref pt): &(Track, PlaylistTrack)) -> (i32, &Track) {
    (pt.playlist_order, t)
}

fn only_tracks<'a>(&(_, ref t): &'a (i32, &Track)) -> &'a Track {
    t
}

fn create_loaded_from_playlist(
    pl: &Playlist,
    r: &[(Track, PlaylistTrack)],
) -> Result<LoadedPlaylist, diesel::result::Error> {
    let mut unsorted = r.iter().map(get_ordering).collect::<Vec<(i32, &Track)>>();
    unsorted.sort_unstable_by(|&(i, _), &(j, _)| i.cmp(&j));

    let sorted = unsorted.iter().map(only_tracks).cloned().collect();
    Ok(LoadedPlaylist {
        id: pl.id,
        name: pl.name.clone(),
        items: sorted,
        current_position: pl.current_position as usize,
    })
}

pub fn restore_playlists(db: &DBPool) -> Result<Vec<LoadedPlaylist>, diesel::result::Error> {
    use crate::schema::playlists::dsl::*;
    use crate::schema::playlisttracks::dsl::*;
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    let pls = playlists.load::<Playlist>(db.lock().expect("DB Error").deref())?;
    pls.iter()
        .map(|pl| {
            let t: Vec<(Track, PlaylistTrack)> = tracks
                .inner_join(playlisttracks)
                .filter(playlist_id.eq(pl.id))
                .load(db.lock().expect("DB Error").deref())?;

            create_loaded_from_playlist(pl, &t)
        })
        .collect()
}
