use crate::schema::tracks;
use crate::types::{DBPool, APP_INFO};
use app_dirs::*;
use diesel;
use diesel::{Connection, SqliteConnection};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use taglib;
use walkdir::DirEntry;

#[derive(AsChangeset, Clone, Debug, Identifiable, Queryable, Serialize, Deserialize)]
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
    pub playcount: Option<i32>,
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

pub trait UpdatePlayCount {
    fn update_playcount(&mut self, _: DBPool);
}

impl UpdatePlayCount for Track {
    fn update_playcount(&mut self, pool: DBPool) {
        use crate::rand::RngCore;
        use crate::schema::tracks::dsl::*;
        use diesel::{QueryDsl, RunQueryDsl, SaveChangesDsl};

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

pub fn setup_db_connection() -> DBPool {
    let mut db_file =
        get_app_root(AppDataType::UserConfig, &APP_INFO).expect("Could not get app root");
    db_file.push("music.db");
    Arc::new(Mutex::new(
        SqliteConnection::establish(&db_file.to_str().unwrap()).expect("Could not open database"),
    ))
}

fn is_valid_file(s: &Result<DirEntry, walkdir::Error>) -> bool {
    if let Ok(ref sp) = *s {
        if sp.metadata().unwrap().file_type().is_file() {
            Some(true)
                == sp.path().extension().map(|ex| {
                    vec!["ogg", "flac", "mp3", "wma", "aac", "opus"].contains(&ex.to_str().unwrap())
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

fn construct_track_from_path(s: &str) -> Result<NewTrack, String> {
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
        Ok(NewTrack {
            title: tags.title().unwrap_or_default(),
            artist: tags.artist().unwrap_or_default(),
            album: tags.album().unwrap_or_default(),
            genre: tags.genre().unwrap_or_default(),
            tracknumber: convert_to_i32_option(tags.track()),
            year: convert_to_i32_option(tags.year()),
            path: s.to_string(),
            length: properties.length() as i32,
            albumpath: album,
        })
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
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SaveChangesDsl};

    let new_track = construct_track_from_path(s)?;
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
    let files = walkdir::WalkDir::new(&p)
        .into_iter()
        .filter(is_valid_file)
        .map(|i| String::from(i.unwrap().path().to_str().unwrap()))
        .collect::<HashSet<String>>();

    let file_count = files.len();

    {
        use crate::schema::tracks::dsl::*;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods};
        let old_files: HashSet<String> = HashSet::from_iter(if fast_delete {
            tracks
                .select(path)
                .load(db.lock().expect("DB Error").deref())
                .expect("Error in loading old files")
        } else {
            tracks
                .select(path)
                //ignore files that are not in the path
                .filter(path.like(String::from("%") + &p + "%"))
                .load(db.lock().expect("DB Error").deref())
                .expect("Error in loading old files")
        });

        /// TODO switch this to par_iter or something
        {
            let pb = ProgressBar::new(file_count as u64);
            pb.set_message("Updating files");
            pb.set_style(ProgressStyle::default_bar()
                                  .template("[{elapsed_precise}] {msg} {spinner:.green} {bar:100.green/blue} {pos:>7}/{len:7} ({percent}%)")
                                  .progress_chars("#>-"));
            let res = pb
                .wrap_iter(files.iter().map(|s| insert_track_with_error_retries(s, db)))
                .collect::<Result<(), String>>();
            pb.finish_with_message("Done Updating");

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
            pb2.set_style(ProgressStyle::default_bar()
                                  .template("[{elapsed_precise}] {msg} {spinner:.green} {bar:100.green/blue} {pos:>7}/{len:7} {percent}    %)")
                                  .progress_chars("#>-"));
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
    use crate::schema::playlists::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    playlists
        .select(crate::schema::playlists::id)
        .order(crate::schema::playlists::id.desc())
        .load(db.lock().expect("DB Error").deref())
        .ok()
        .and_then(|v: Vec<i32>| v.first().cloned())
        .map_or(0, |i| i + 1)
}
