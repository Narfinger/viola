use viola_common::schema::{playlists, playlisttracks};

use crate::loaded_playlist::LoadedPlaylist;
use crate::types::DBPool;
use viola_common::Track;

#[derive(Identifiable, Queryable)]
pub struct Playlist {
    pub id: i32,
    pub name: String,
    pub current_position: i32,
}

#[derive(Insertable)]
#[diesel(table_name = playlists)]
pub struct NewPlaylist {
    pub id: i32,
    pub name: String,
    pub current_position: i32,
}

#[derive(Identifiable, Queryable, Associations)]
#[diesel(table_name = playlisttracks, belongs_to(Track, foreign_key = playlist_id), belongs_to(Playlist, foreign_key = track_id))]
pub struct PlaylistTrack {
    id: i32,
    playlist_id: i32,
    track_id: i32,
    playlist_order: i32,
}

#[derive(Debug, Insertable, Associations)]
#[diesel(table_name = playlisttracks, belongs_to(Track, foreign_key = playlist_id), belongs_to(Playlist, foreign_key = track_id))]
pub struct NewPlaylistTrack {
    pub playlist_id: i32,
    pub track_id: i32,
    pub playlist_order: i32,
}

fn get_ordering(&(ref t, ref pt): &(Track, PlaylistTrack)) -> (i32, &Track) {
    (pt.playlist_order, t)
}

fn only_tracks<'a>(&(_, t): &'a (i32, &Track)) -> &'a Track {
    t
}

fn create_loaded_from_playlist(pl: &Playlist, r: &[(Track, PlaylistTrack)]) -> LoadedPlaylist {
    let mut unsorted = r.iter().map(get_ordering).collect::<Vec<(i32, &Track)>>();
    unsorted.sort_unstable_by(|&(i, _), &(j, _)| i.cmp(&j));

    let sorted = unsorted.iter().map(only_tracks).cloned().collect();
    LoadedPlaylist {
        id: pl.id,
        name: pl.name.clone(),
        items: sorted,
        current_position: pl.current_position as usize,
    }
}

#[must_use]
pub fn restore_playlists(db: &DBPool) -> Vec<LoadedPlaylist> {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use viola_common::schema::playlists::dsl::*;
    use viola_common::schema::playlisttracks::dsl::*;
    use viola_common::schema::tracks::dsl::*;

    let pls = playlists
        .order(viola_common::schema::playlists::dsl::id.asc())
        .load::<Playlist>(&mut *db.lock())
        .expect("Error restoring playlists");
    pls.iter()
        .map(|pl| {
            let t: Vec<(Track, PlaylistTrack)> = tracks
                .inner_join(playlisttracks)
                .filter(playlist_id.eq(pl.id))
                .load(&mut *db.lock())
                .expect("Error restoring a playlist");

            create_loaded_from_playlist(pl, &t)
        })
        .collect()
}
