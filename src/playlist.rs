use gtk::prelude::*;
use std::ops::Deref;

use db::Track;
use types::DBPool;

pub struct Playlist {
    pub items: Vec<Track>,
    pub current_position: i64,
}

pub fn playlist_from_directory(folder: &str, pool: &DBPool) -> Playlist {
    use schema::tracks::dsl::*;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;use diesel::TextExpressionMethods;


    let db = pool.get().unwrap();
    let results = tracks
                    .filter(path.like(format!("%{}%", folder)))
                    .order(path)
                    .load(db.deref())
                    .expect("Problem loading playlist");

    Playlist {items: results, current_position: 0}
}

pub fn get_current_uri(p: &Playlist) -> String {
    format!("file:////{}", p.items[p.current_position as usize].path.replace(" ", "%20"))
}