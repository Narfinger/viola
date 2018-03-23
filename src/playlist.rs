use std::ops::Deref;
use diesel;

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
    playlist_order: i32,
}

pub struct LoadedPlaylist {
    pub id: Option<i32>,
    pub name: String,
    pub items: Vec<Track>,
    pub current_position: i32,
}

fn get_ordering(&(ref t, ref pt): &(Track, PlaylistTracks)) -> (i32, &Track) {
    (pt.playlist_order, t)
}

fn only_tracks<'a>(&(ref i, ref t): &'a (i32, &Track)) -> &'a Track {
    t
}

fn create_loaded_from_playlist(pl: &Playlist, r: &Vec<(Track,PlaylistTracks)>) -> Result<LoadedPlaylist, diesel::result::Error> {
    let mut unsorted = r.iter()
        .map(get_ordering)
        .collect::<Vec<(i32, &Track)>>();
    unsorted.sort_unstable_by(|&(i, _), &(j, _)| i.cmp(&j));

    let sorted = unsorted.iter().map(only_tracks).map(|t| t.clone()).collect();
    Ok(LoadedPlaylist {id: Some(pl.id), name: pl.name.clone(), items: sorted, current_position: pl.current_position})
}

pub fn restore_playlists(pool: &DBPool) -> Result<Vec<LoadedPlaylist>, diesel::result::Error> {
    use schema::playlisttracks::dsl::*;
    use schema::playlists::dsl::*;
    use schema::tracks::dsl::*;
    use diesel::associations::HasTable;
    use diesel::{BelongingToDsl, QueryDsl, RunQueryDsl, GroupedBy, ExpressionMethods};
    use diesel;

    let db = pool.get().unwrap();
    let pls = playlists.load::<Playlist>(db.deref())?;
    pls.iter().map(|pl| {
        let t: Vec<(Track, PlaylistTracks)> = tracks.inner_join(playlisttracks)
        .filter(playlist_id.eq(pl.id))
        .load(db.deref())?;
        
        create_loaded_from_playlist(pl, &t)
    }).collect()
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
    LoadedPlaylist {id: None, name: String::from(playlistname), items: results, current_position: 0}
}

pub fn get_current_uri(p: &LoadedPlaylist) -> String {
    format!("file:////{}", p.items[p.current_position as usize].path.replace(" ", "%20"))
}