use std::ops::Deref;

use db::Track;
use types::DBPool;

#[derive(Queryable)]
struct Playlist {
    id: i32,
    name: String,
    current_position: i32,
}

#[derive(Queryable)]
pub struct PlaylistTracks {
    id: i32,
    playlist_id: i32,
    track_id: i32,
}

pub struct LoadedPlaylist {
    pub name: String,
    pub items: Vec<Track>,
    pub current_position: i64,
}

pub fn restore_playlists(pool: &DBPool) -> Vec<LoadedPlaylist> {
    use schema::playlisttracks::dsl::*;
    use schema::playlists::dsl::*;
    use schema::tracks::dsl::*;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use diesel;

    let db = pool.get().unwrap();
    let results:Result<Vec<(Playlist, (PlaylistTracks, Track))>,diesel::result::Error> = playlists.inner_join(playlisttracks.inner_join(tracks)).load(db.deref());
    vec![]
}

pub fn playlist_from_directory(folder: &str, pool: &DBPool) -> LoadedPlaylist {
    use schema::tracks::dsl::*;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use diesel::TextExpressionMethods;

    let db = pool.get().unwrap();
    let results = tracks
                    .filter(path.like(format!("%{}%", folder)))
                    .order(path)
                    .load(db.deref())
                    .expect("Problem loading playlist");

    let playlistname = &folder[&folder.len() -10..];
    LoadedPlaylist {name: String::from(playlistname), items: results, current_position: 0}
}

pub fn get_current_uri(p: &LoadedPlaylist) -> String {
    format!("file:////{}", p.items[p.current_position as usize].path.replace(" ", "%20"))
}