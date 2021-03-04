use crate::types::{DBPool, APP_INFO};
use app_dirs::*;
use diesel::{Connection, SqliteConnection};
use indicatif::ParallelProgressIterator;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::ops::Deref;
use std::path::Path;
use std::{thread, time};
use viola_common::schema::tracks;
use viola_common::Track;
use walkdir::DirEntry;

static PROGRESSBAR_STYLE: &str =
    "[{elapsed_precise}] {msg} {spinner:.green} {bar:.green/blue} {pos:>7}/{len:7} ({percent}%)";

static PROGRESSBAR_UNKNOWN_STYLE: &str =
    "{msg} {spinner:.green} | Elapsed: {elapsed} | Files/sec: {per_sec} | Pos: {pos}";

pub trait UpdatePlayCount {
    fn update_playcount(&mut self, _: DBPool);
}

impl UpdatePlayCount for Track {
    fn update_playcount(&mut self, pool: DBPool) {
        use crate::rand::RngCore;
        use diesel::{QueryDsl, RunQueryDsl, SaveChangesDsl};
        use viola_common::schema::tracks::dsl::*;

        //wait a random time
        let mut rng = rand::thread_rng();
        std::thread::sleep(std::time::Duration::new(0, rng.next_u32()));
        let db = pool.lock().expect("Error in locking db");

        let db_track: Result<Track, diesel::result::Error> = tracks.find(self.id).first(db.deref());
        if let Ok(mut track) = db_track {
            track.playcount = Some(1 + track.playcount.unwrap_or(0));
            if track.save_changes::<Track>(db.deref()).is_err() {
                error!("Some problem with updating play status (cannot update)");
            }
        } else {
            error!("Some problem with updating play status (gettin track)");
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "tracks"]
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

embed_migrations!("migrations/");
pub fn setup_db_connection() -> Result<diesel::SqliteConnection, String> {
    let mut db_file = get_app_root(AppDataType::UserConfig, &APP_INFO)
        .map_err(|_| String::from("Could not get app root"))?;
    if !db_file.exists() {
        return Err(String::from("Dir does not exists"));
    }
    db_file.push("music.db");
    SqliteConnection::establish(&db_file.to_str().unwrap())
        .map_err(|_| String::from("DB Connection error"))
}

pub fn create_db() {
    let db_dir = get_app_root(AppDataType::UserConfig, &APP_INFO)
        .map_err(|_| String::from("Could not get app root"))
        .expect("Error getting app dir");
    if !db_dir.exists() {
        std::fs::create_dir(get_app_root(AppDataType::UserConfig, &APP_INFO).unwrap())
            .expect("We could not create app dir");
    }
    let mut db_file =
        get_app_root(AppDataType::UserConfig, &APP_INFO).expect("Could not get app root");
    db_file.push("music.db");
    let connection =
        SqliteConnection::establish(&db_file.to_str().unwrap()).expect("Something wrong");
    embedded_migrations::run(&connection).expect("Could not run migration");
}

fn is_valid_file(s: &Result<DirEntry, walkdir::Error>) -> bool {
    if let Ok(ref sp) = *s {
        if sp.metadata().unwrap().file_type().is_file() {
            Some(true)
                == sp.path().extension().map(|ex| {
                    vec!["ogg", "flac", "mp3", "wma", "aac", "opus", "m4a"]
                        .contains(&ex.to_str().unwrap())
                })
        } else {
            false
        }
    } else {
        false
    }
}

fn get_album_file(s: &str) -> Option<String> {
    let p = Path::new(s);
    let jpg = p.with_file_name("cover.jpg");
    let png = p.with_file_name("cover.png");
    if jpg.exists() {
        Some(jpg)
    } else if png.exists() {
        Some(png)
    } else {
        None
    }
    .and_then(|s| s.to_str().map(String::from))
}

fn convert_to_i32_option(u: Option<u32>) -> Option<i32> {
    if let Some(i) = u {
        Some(i as i32)
    } else {
        None
    }
}

fn construct_track_from_path(s: &str) -> NewTrack {
    let taglibfile = taglib::File::new(&s);
    if let Ok(ataglib) = taglibfile {
        let tags = ataglib
            .tag()
            .unwrap_or_else(|e| panic!(format!("Could not read tags for: {}. {:?}", s, e)));
        let properties = ataglib
            .audioproperties()
            .unwrap_or_else(|_| panic!(format!("Could not find audio properties for: {}", s)));
        let album = get_album_file(&s);
        //tracknumber and year return 0 if none set
        NewTrack {
            title: tags.title().unwrap_or_default(),
            artist: tags.artist().unwrap_or_default(),
            album: tags.album().unwrap_or_default(),
            genre: tags.genre().unwrap_or_default(),
            tracknumber: convert_to_i32_option(tags.track()),
            year: convert_to_i32_option(tags.year()),
            path: s.to_string(),
            length: properties.length() as i32,
            albumpath: album,
        }
    } else {
        panic!(format!("Taglib could not open file: {}", s));
    }
}

fn tags_equal(nt: &NewTrack, ot: &Track) -> bool {
    nt.title == ot.title
        && nt.artist == ot.artist
        && nt.album == ot.album
        && nt.genre == ot.genre
        && nt.tracknumber == ot.tracknumber
        && nt.year == ot.year
        && nt.length == ot.length
        && nt.albumpath == ot.albumpath
}

fn insert_track_with_error_retries(s: &str, db: &DBPool) -> Result<(), String> {
    let mut i = 2;
    let mut res = Err("retry".into());
    while res.is_err() {
        res = insert_track(s, db);
        i -= 1;
        if res.is_ok() {
            return res;
        } else if i > 0 {
            let ten_millis = time::Duration::from_millis(10);
            thread::sleep(ten_millis);
        } else if i <= 0 {
            return res;
        }
    }
    res
}

fn insert_track(s: &str, db: &DBPool) -> Result<(), String> {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SaveChangesDsl};
    use viola_common::schema::tracks::dsl::*;

    let new_track = construct_track_from_path(s);
    let old_track_perhaps = tracks
        .filter(path.eq(&new_track.path))
        .get_result::<Track>(db.lock().expect("DB Error").deref());

    if let Ok(mut old_track) = old_track_perhaps {
        if tags_equal(&new_track, &old_track) {
            Ok(())
        } else {
            old_track.title = new_track.title;
            old_track.artist = new_track.artist;
            old_track.album = new_track.album;
            old_track.genre = new_track.genre;
            old_track.tracknumber = new_track.tracknumber;
            old_track.year = new_track.year;
            old_track.length = new_track.length;
            old_track.albumpath = new_track.albumpath;

            old_track
                .save_changes::<Track>(db.lock().expect("DB Error").deref())
                .map(|_| ())
                .map_err(|err| format!("Error in updateing for track {}, See full: {:?}", s, err))
        }
    } else {
        diesel::insert_into(tracks)
            .values(&new_track)
            .execute(db.lock().expect("DB Error").deref())
            .map(|_| ())
            .map_err(|err| format!("Insertion Error for track {}, See full: {:?}", s, err))
    }
}

/// Tested on 01-06-2019 with jwalk and walkdir. walkdir was faster on my machine
pub fn build_db(p: &str, db: &DBPool, fast_delete: bool) -> Result<(), String> {
    info!("Building database, getting walkdir iterator");
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template(PROGRESSBAR_UNKNOWN_STYLE));
    pb.set_message("Collecting files");
    let files = pb
        .wrap_iter(walkdir::WalkDir::new(&p).into_iter())
        .filter(is_valid_file)
        .map(|i| String::from(i.unwrap().path().to_str().unwrap()))
        .collect::<HashSet<String>>();
    pb.finish_with_message("Done Updating");

    let file_count = files.len();

    {
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods};
        use viola_common::schema::tracks::dsl::*;
        let old_files: HashSet<String> = HashSet::from_iter(if fast_delete {
            tracks
                .select(path)
                .load(db.lock().expect("DB Error").deref())
                .expect("Error in loading old files")
        } else {
            tracks
                .select(path)
                //ignore files that are not in the path
                .filter(path.like(String::from("%") + p + "%"))
                .load(db.lock().expect("DB Error").deref())
                .expect("Error in loading old files")
        });

        {
            let pb = ProgressBar::new(file_count as u64);
            pb.set_message("Updating tags");
            pb.set_style(ProgressStyle::default_bar().template(PROGRESSBAR_STYLE));
            let res = files
                .par_iter()
                .progress_with(pb)
                .map(|s| insert_track_with_error_retries(s, db))
                .collect::<Result<(), String>>();

            if let Err(err) = res {
                error!("Error in updating database");
                error!("{}", err);
                panic!("Aborting");
            }
        }

        {
            info!("Deleting old database entries");
            let pb = ProgressBar::new_spinner();
            pb.set_message("Computing Difference to old database");
            let to_delete: Vec<&String> = old_files.difference(&files).collect();
            pb.finish();

            let pb2 = ProgressBar::new(to_delete.len() as u64);
            pb2.set_style(
                ProgressStyle::default_bar()
                    .template(PROGRESSBAR_STYLE)
                    .progress_chars("#>-"),
            );
            pb2.set_message("Deleting old unused entries");
            for i in pb2.wrap_iter(to_delete.iter()) {
                //println!("to delete: {}", i);
                diesel::delete(tracks)
                    .filter(path.eq(i))
                    .execute(db.lock().expect("DB Error").deref())
                    .unwrap_or_else(|_| {
                        panic!("Error in deleting outdated database entries: {}", &i)
                    });
            }
            pb.finish_with_message("Done removing old entries");
        }
    }

    Ok(())
}

// returns an id for a newly created playlist. Returns 0 if no playlists yet in db
pub fn get_new_playlist_id(db: &DBPool) -> i32 {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use viola_common::schema::playlists::dsl::*;
    playlists
        .select(viola_common::schema::playlists::id)
        .order(viola_common::schema::playlists::id.desc())
        .load(db.lock().expect("DB Error").deref())
        .ok()
        .and_then(|v: Vec<i32>| v.first().cloned())
        .map_or(0, |i| i + 1)
}
