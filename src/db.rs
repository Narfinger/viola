use diesel;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::Path;
use std::ops::Deref;
use r2d2;
use r2d2_diesel;
use taglib;
use walkdir;
use types::DBPool;
use schema::tracks;

#[derive(Identifiable,Queryable, Clone)]
pub struct Track {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub tracknumber: Option<i32>,
    pub year: Option<i32>,
    pub path: String,
    pub length: i32,
    pub albumpath: Option<String>,
}

#[derive(Insertable)]
#[table_name="tracks"]
pub struct NewTrack {
    title: String,
    artist: String,
    album: String,
    genre: String,
    tracknumber: Option<i32>,
    year: Option<i32>,
    path: String,
    length: i32,
    albumpath: Option<String>,
}

pub fn setup_db_connection() -> DBPool {
    let manager = r2d2_diesel::ConnectionManager::<diesel::SqliteConnection>::new("./music.db");
    r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
}

fn check_file(s: &Result<walkdir::DirEntry, walkdir::Error>) -> bool {
    if let Ok(ref sp) = *s {
        if sp.file_type().is_file() {
            Some(true) == sp.path().extension().map(|ex| vec!["ogg", "flac", "mp3", "wma", "aac", "opus"].contains(&ex.to_str().unwrap()))
        } else {
            false
        }
    } else {
        false
    }
}

/// gets a number and returns None if the number is zero, otherwise the number converted to i32 
fn number_zero_to_option(i: u32) -> Option<i32> {
    if i == 0 {
        None
    } else {
        Some(i as i32)
    }
}

fn get_album_file(s: &str) -> Option<String> {
    Path::new(s)
        .parent()
        .and_then(|p| {
            let jpg = p.with_file_name("cover.jpg");
            let png = p.with_file_name("cover.png");
            if jpg.exists() {
                Some(jpg)
            } else if png.exists() {
                Some(png)
            } else {
                None
            }})
        .and_then(|s| s.to_str().map(String::from))    
}

fn construct_track_from_path(s: String) -> Result<NewTrack, String> {
    let taglibfile = taglib::File::new(&s);
    if let Ok(ataglib) = taglibfile {
        let tags = ataglib.tag()
            .unwrap_or_else(|_| panic!(format!("Could not read tags for: {}", s)));
        let properties = ataglib
            .audioproperties().unwrap_or_else(|_| panic!(format!("Could not find audio properties for: {}", s)));
        let album = get_album_file(&s);
        //tracknumber and year return 0 if none set
        Ok(NewTrack {
            title: tags.title(),
            artist: tags.artist(),
            album: tags.album(),
            genre: tags.genre(),
            tracknumber: number_zero_to_option(tags.track()),
            year: number_zero_to_option(tags.year()),
            path: s,
            length: properties.length() as i32,
            albumpath: album,
        })
    } else {
        panic!(format!("Taglib could not open file: {}", s));
    }
}

fn insert_track(s: String, pool: &DBPool) -> Result<(), String> {
    use schema::tracks;
    use diesel::RunQueryDsl;

    let db = pool.get().unwrap();
    let track = construct_track_from_path(s)?;
    diesel::replace_into(tracks::table)
        .values(&track)
        .execute(db.deref())
        .map(|_| ())
        .map_err(|_| "Insertion Error".into())
}

pub fn build_db(path: &str, pool: &DBPool) -> Result<(), String> {
    let files = walkdir::WalkDir::new(&path)
                .into_iter()
                .filter(check_file)
                .map(|i| String::from(i.unwrap().path().to_str().unwrap()));
    
    let file_count = walkdir::WalkDir::new(&path)
                .into_iter()
                .filter(check_file)
                .map(|i| String::from(i.unwrap().path().to_str().unwrap()))
                .count();
    /// TODO switch this to par_iter or something
    let pb = ProgressBar::new(file_count as u64);
    pb.set_message("Updating files");
    pb.set_style(ProgressStyle::default_bar()
                          .template("[{elapsed_precise}] {msg} {spinner:.green} {bar:100.green/blue} {pos:>7}/{len:7} ({percent}%)")
                          .progress_chars("#>-"));
    let res = pb.wrap_iter(files.into_iter().map(|s| insert_track(s, pool)))
        .collect::<Result<(), String>>();
    pb.finish_with_message("Done Updating");
    res
}