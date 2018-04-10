use diesel;
use schema::{playlists, playlisttracks};
use std::ops::Deref;

use db::Track;
use types::DBPool;

#[derive(Identifiable, Queryable, Associations)]
struct Playlist {
    id: i32,
    name: String,
    current_position: i32,
}

#[derive(Insertable)]
#[table_name = "playlists"]
struct NewPlaylist {
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

#[derive(Insertable, Associations)]
#[table_name = "playlisttracks"]
struct NewPlaylistTrack {
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

fn get_ordering(&(ref t, ref pt): &(Track, PlaylistTrack)) -> (i32, &Track) {
    (pt.playlist_order, t)
}

fn only_tracks<'a>(&(ref i, ref t): &'a (i32, &Track)) -> &'a Track {
    t
}

fn create_loaded_from_playlist(
    pl: &Playlist,
    r: &Vec<(Track, PlaylistTrack)>,
) -> Result<LoadedPlaylist, diesel::result::Error> {
    let mut unsorted = r.iter().map(get_ordering).collect::<Vec<(i32, &Track)>>();
    unsorted.sort_unstable_by(|&(i, _), &(j, _)| i.cmp(&j));

    let sorted = unsorted
        .iter()
        .map(only_tracks)
        .map(|t| t.clone())
        .collect();
    Ok(LoadedPlaylist {
        id: Some(pl.id),
        name: pl.name.clone(),
        items: sorted,
        current_position: pl.current_position,
    })
}

pub fn restore_playlists(pool: &DBPool) -> Result<Vec<LoadedPlaylist>, diesel::result::Error> {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use schema::playlists::dsl::*;
    use schema::playlisttracks::dsl::*;
    use schema::tracks::dsl::*;

    let db = pool.get().unwrap();
    let pls = playlists.load::<Playlist>(db.deref())?;
    pls.iter()
        .map(|pl| {
            let t: Vec<(Track, PlaylistTrack)> = tracks
                .inner_join(playlisttracks)
                .filter(playlist_id.eq(pl.id))
                .load(db.deref())?;

            create_loaded_from_playlist(pl, &t)
        })
        .collect()
}

pub fn update_playlist(pool: &DBPool, pl: &LoadedPlaylist) {
    use diesel;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use schema::playlists::dsl::*;
    use schema::playlisttracks::dsl::*;

    let db = pool.get().unwrap();
    if let Some(id) = pl.id {
        // the playlist is already in the database
        diesel::update(playlists.find(id))
            .set(current_position.eq(pl.current_position))
            .execute(db.deref())
            .expect("Error in playlist update");
    }
    let playlist: Playlist = if let Some(id) = pl.id {
        playlists
            .find(id)
            .first::<Playlist>(db.deref())
            .expect("DB Error")
    } else {
        let t = vec![NewPlaylist {
            name: pl.name.clone(),
            current_position: pl.current_position,
        }];
        diesel::insert_into(playlists)
            .values(&t)
            .execute(db.deref())
            .expect("Database error");
        playlists
            .filter(name.eq(&pl.name))
            .first(db.deref())
            .expect("DB Erorr")
    };
    // the playlist is not in the database
    for (index, track) in pl.items.iter().enumerate() {
        let t = vec![NewPlaylistTrack {
            playlist_id: playlist.id,
            track_id: track.id,
            playlist_order: index as i32,
        }];
        diesel::insert_into(playlisttracks)
            .values(&t)
            .execute(db.deref())
            .expect("Database error");

        /*diesel::insert_into(playlisttracks::table)
            .values(&t)
            .execute(db.deref())
            .map(|_| ())
            .map_err(|_| "Insertion Error".into());
        */
    }
    panic!("fix playlist");
}

pub fn playlist_from_directory(folder: &str, pool: &DBPool) -> LoadedPlaylist {
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use diesel::TextExpressionMethods;
    use schema::tracks::dsl::*;

    let db = pool.get().unwrap();
    let results = tracks
        .filter(path.like(format!("%{}%", folder)))
        .order(path)
        .load(db.deref())
        .expect("Problem loading playlist");

    let playlistname = &folder[&folder.len() - 10..];
    LoadedPlaylist {
        id: None,
        name: String::from(playlistname),
        items: results,
        current_position: 0,
    }
}

pub fn get_current_uri(p: &LoadedPlaylist) -> String {
    format!(
        "file:////{}",
        p.items[p.current_position as usize]
            .path
            .replace(" ", "%20")
    )
}
