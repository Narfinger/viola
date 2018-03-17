use diesel;
use rayon::prelude::*;
use std;
use std::ops::Deref;
use r2d2;
use r2d2_diesel;
use taglib;
use walkdir;
use types::DBPool;
use schema::tracks;

#[derive(Queryable)]
pub struct Track {
    pub id: i32,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub path: String,
    pub duration: Option<i32>,
    pub albumpath: Option<String>,
}

#[derive(Insertable)]
#[table_name="tracks"]
pub struct NewTrack {
    title: String,
    artist: String,
    album: String,
    year: i32,
    path: String,
    duration: i32,
    albumpath: Option<String>,
}

pub fn setup_db_connection() -> DBPool {
    let manager = r2d2_diesel::ConnectionManager::<diesel::SqliteConnection>::new("./music.db");
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}

fn check_dir(s: &Result<walkdir::DirEntry, walkdir::Error>) -> bool {
    if let &Ok(ref sp) = s {
        sp.file_type().is_file()
    } else {
        false
    }
}

fn construct_track_from_path(s: String) -> Result<NewTrack, String> {
    let taglibfile = taglib::File::new(&s);
    if let Err(e) = taglibfile {
        Err("taglib couldn't open the file".into())
    } else {
        let ataglib = taglibfile.unwrap();
        let tags = ataglib.tag().unwrap();

        Ok(NewTrack {
            title: tags.title(),
            artist: tags.artist(),
            album: tags.album(),
            year: tags.year() as i32,
            path: s,
            duration: 0,
            albumpath: None,
        })
    }
}

fn insert_track(s: String, pool: DBPool) -> Result<(), String> {
    return Err(String::from("Not yet checking for new version"));
    use schema::tracks;
    use diesel::RunQueryDsl;

    let db = pool.get().unwrap();
    let track = construct_track_from_path(s)?;
    diesel::insert_into(tracks::table)
        .values(&track)
        .execute(db.deref())
        .map(|_| ())
        .map_err(|e| "Insertion Error".into())
}

pub fn build_db(path: String, pool: DBPool) -> Result<(), String> {
    let db = pool.get();
    let files = walkdir::WalkDir::new(path)
                .into_iter()
                .filter(check_dir)
                .map(|i| String::from(i.unwrap().path().to_str().unwrap()));
    
    /// TODO switch this to par_iter or something
    files.into_iter().map(|s| insert_track(s, pool.clone())).collect::<Result<(), String>>()
}