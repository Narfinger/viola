use crate::schema::{playlists, playlisttracks};
use diesel;
use std::cell::Cell;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::db::Track;
use crate::loaded_playlist::LoadedPlaylist;
use crate::types::DBPool;

#[derive(Identifiable, Queryable, Associations)]
struct Playlist {
    id: i32,
    name: String,
    current_position: i32,
}

#[derive(Insertable)]
#[table_name = "playlists"]
struct NewPlaylist {
    id: i32,
    name: String,
    current_position: i32,
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
struct NewPlaylistTrack {
    playlist_id: i32,
    track_id: i32,
    playlist_order: i32,
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

pub fn update_playlist(db: &DBPool, pl: &LoadedPlaylist) -> Result<(), diesel::result::Error> {
    use crate::schema::playlists::dsl::*;
    use crate::schema::playlisttracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    info!("playlist id {:?}", pl.id);

    let exists = diesel::select(diesel::dsl::exists(
        playlists.filter(crate::schema::playlists::id.eq(pl.id)),
    ))
    .get_result(db.lock().expect("DB Error").deref())
    .expect("Error in db");

    if exists {
        // the playlist is already in the database
        diesel::update(playlists.find(pl.id))
            .set(current_position.eq(pl.current_position as i32))
            .execute(db.lock().expect("DB Error").deref())?;
    }

    let playlist: Playlist = if exists {
        playlists
            .find(pl.id)
            .first::<Playlist>(db.lock().expect("DB Error").deref())?
    } else {
        let t = vec![NewPlaylist {
            id: pl.id,
            name: pl.name.clone(),
            current_position: pl.current_position as i32,
        }];
        diesel::insert_into(playlists)
            .values(&t)
            .execute(db.lock().expect("DB Error").deref())?;
        playlists
            .filter(name.eq(&pl.name))
            .first(db.lock().expect("DB Error").deref())?
    };

    //deleting old tracks
    diesel::delete(playlisttracks)
        .filter(playlist_id.eq(playlist.id))
        .execute(db.lock().expect("DB Error").deref())?;

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
        .execute(db.lock().expect("DB Error").deref())?;

    info!("done");

    Ok(())
}

pub fn clear_all(db: &DBPool) {
    use crate::schema::playlists::dsl::*;
    use crate::schema::playlisttracks::dsl::*;
    use diesel::RunQueryDsl;

    diesel::delete(playlists)
        .execute(db.lock().expect("DB Error").deref())
        .expect("Error in cleaning playlists");
    diesel::delete(playlisttracks)
        .execute(db.lock().expect("DB Error").deref())
        .expect("Error in cleaning playlisttracks");
}

pub fn delete_with_id(db: &DBPool, index: i32) {
    use crate::schema;
    use crate::schema::playlists::dsl::*;
    use crate::schema::playlisttracks::dsl::*;
    use diesel::{ExpressionMethods, RunQueryDsl};

    info!("index for deleting: {}", index);

    diesel::delete(playlists)
        .filter(schema::playlists::dsl::id.eq(index))
        .execute(db.lock().expect("DB Error").deref())
        .expect("Error in database deletion");

    diesel::delete(playlisttracks)
        .filter(playlist_id.eq(index))
        .execute(db.lock().expect("DB Error").deref())
        .expect("Error in database deletion");
}

/*
pub fn load_playlist_from_directory(folder: &str, pool: &DBPool) -> LoadedPlaylist {
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use diesel::TextExpressionMethods;
    use schema::tracks::dsl::*;

    let db = pool.get().unwrap();
    let results = tracks
        .filter(path.like(format!("%{}%", folder)))
        .order(path)
        .load(db.lock().expect("DB Error").deref())
        .expect("Problem loading playlist");

    let playlistname = &folder[&folder.len() - 10..];
    LoadedPlaylist {
        id: None,
        name: String::from(playlistname),
        items: results,
        current_position: 0,
    }
}
*/
