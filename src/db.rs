use crate::types::DBPool;
use diesel::Insertable;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::MigrationHarness;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use indicatif::ParallelProgressIterator;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use std::iter::FromIterator;
use std::ops::DerefMut;
use std::path::Path;
use std::{collections::HashSet, path::PathBuf};
use std::{thread, time};
use viola_common::schema::tracks;
use viola_common::Track;
use walkdir::DirEntry;

static PROGRESSBAR_STYLE: &str =
    "[{elapsed_precise}] {msg} {spinner:.green} {bar:.green/blue} {pos:>7}/{len:7} ({percent}%)";

static PROGRESSBAR_UNKNOWN_STYLE: &str =
    "{msg} {spinner:.green} | Elapsed: {elapsed} | Files/sec: {per_sec}";

pub(crate) trait UpdatePlayCount {
    fn update_playcount(&mut self, _: DBPool);
}

impl UpdatePlayCount for Track {
    fn update_playcount(&mut self, pool: DBPool) {
        use diesel::{QueryDsl, RunQueryDsl, SaveChangesDsl};
        use rand::RngCore;
        use viola_common::schema::tracks::dsl::*;

        //wait a random time
        let mut rng = rand::thread_rng();
        std::thread::sleep(std::time::Duration::new(0, rng.next_u32()));
        let mut db = pool.lock();

        let db_track: Result<Track, diesel::result::Error> =
            tracks.find(self.id).first(db.deref_mut());
        if let Ok(mut track) = db_track {
            track.playcount = Some(1 + track.playcount.unwrap_or(0));
            if track.save_changes::<Track>(db.deref_mut()).is_err() {
                error!("Some problem with updating play status (cannot update)");
            }
        } else {
            error!("Some problem with updating play status (gettin track)");
        }
    }
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = tracks)]
pub(crate) struct NewTrack {
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

/// the migrations we run
pub(crate) const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

/// Opens the db connection and returns it
pub(crate) fn setup_db_connection() -> Result<diesel::SqliteConnection, String> {
    let mut db_file =
        crate::utils::get_config_dir().map_err(|_| String::from("Could not get app root"))?;
    if !db_file.exists() {
        return Err(String::from("Dir does not exists"));
    }
    db_file.push("music.db");
    SqliteConnection::establish(db_file.to_str().unwrap())
        .map_err(|_| String::from("DB Connection error"))
}

/// create the db file
pub(crate) fn create_db() {
    let db_dir = crate::utils::get_config_dir()
        .map_err(|_| String::from("Could not get app root"))
        .expect("Error getting app dir");
    if !db_dir.exists() {
        crate::utils::get_config_dir().expect("We could not create app dir");
    }
    let mut db_file = crate::utils::get_config_dir().unwrap();
    db_file.push("music.db");
    let mut connection =
        SqliteConnection::establish(db_file.to_str().unwrap()).expect("Something wrong");
    connection.run_pending_migrations(MIGRATIONS).unwrap();
}

/// is this a valid file, i.e., has the correct extension
fn is_valid_file(s: &Result<DirEntry, walkdir::Error>) -> bool {
    if let Ok(ref sp) = *s {
        if sp.metadata().unwrap().file_type().is_file() {
            Some(true)
                == sp.path().extension().map(|ex| {
                    ["ogg", "flac", "mp3", "wma", "aac", "opus", "m4a"]
                        .contains(&ex.to_str().unwrap())
                })
        } else {
            false
        }
    } else {
        false
    }
}

/// constants that tell me which names are ok
const ALBUM_NAMES: [&str; 3] = ["cover.jpg", "cover.png", "cover.webp"];
/// for a given path, tries to find cover.jpg, cover.png and also check the parent directory for it.
/// Returns Option of the string it found
fn get_album_file(s: &str) -> Option<String> {
    let cur_path = Path::new(s);
    let mut covers = ALBUM_NAMES
        .iter()
        .map(|v| cur_path.with_file_name(v))
        .filter(|p| p.exists());

    let mut covers_parent = ALBUM_NAMES
        .iter()
        .filter_map(|v| cur_path.parent().map(|p| p.with_file_name(v)))
        .filter(|p| p.exists());

    covers
        .next()
        .or_else(|| covers_parent.next())
        .and_then(|p: PathBuf| p.to_str().map(String::from))
}

/// construct a `NewTrack` from a path pointing to a file
fn construct_track_from_path(s: &str) -> NewTrack {
    let taglibfile = taglib::File::new(s);
    if let Ok(ataglib) = taglibfile {
        let tags = ataglib
            .tag()
            .unwrap_or_else(|e| panic!("Could not read tags for: {}. {:?}", s, e));
        let properties = ataglib
            .audioproperties()
            .unwrap_or_else(|_| panic!("Could not find audio properties for: {}", s));
        let album = get_album_file(s);
        //tracknumber and year return 0 if none set
        NewTrack {
            title: tags.title().unwrap_or_default(),
            artist: tags.artist().unwrap_or_default(),
            album: tags.album().unwrap_or_default(),
            genre: tags.genre().unwrap_or_default(),
            tracknumber: tags.track().map(|i| i as i32),
            year: tags.year().map(|i| i as i32),
            path: s.to_string(),
            length: properties.length() as i32,
            albumpath: album,
        }
    } else {
        panic!("Taglib could not open file: {}", s);
    }
}

/// are the tags, not tracks equal?
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

/// insert tracks but retry
fn insert_track_with_error_retries(s: &str, db: &DBPool) -> Result<(), String> {
    for i in 1..3 {
        let res = insert_track(s, db);
        if res.is_ok() {
            return res;
        } else if i > 0 {
            thread::sleep(time::Duration::from_secs(2));
        } else if i <= 0 {
            return res;
        }
    }
    Err(String::from("Could not insert"))
}

/// insert a track into a db given by the filepath `s``
fn insert_track(s: &str, db: &DBPool) -> Result<(), String> {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SaveChangesDsl};
    use viola_common::schema::tracks::dsl::*;

    let new_track = construct_track_from_path(s);
    let old_track_perhaps = tracks
        .filter(path.eq(&new_track.path))
        .get_result::<Track>(&mut *db.lock());

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
                .save_changes::<Track>(&mut *db.lock())
                .map(|_| ())
                .map_err(|err| format!("Error in updateing for track {}, See full: {:?}", s, err))
        }
    } else {
        diesel::insert_into(tracks)
            .values(&new_track)
            .execute(&mut *db.lock())
            .map(|_| ())
            .map_err(|err| format!("Insertion Error for track {}, See full: {:?}", s, err))
    }
}

/// Tested on 01-06-2019 with jwalk and walkdir. walkdir was faster on my machine
pub(crate) fn build_db(p: &str, db: &DBPool, fast_delete: bool) -> Result<(), String> {
    info!("Building database, getting walkdir iterator");
    let pb = ProgressBar::new_spinner();
    let style = ProgressStyle::default_spinner()
        .template(PROGRESSBAR_UNKNOWN_STYLE)
        .map_err(|_| String::from("Error in progressstyle"))?;
    pb.set_style(style);
    pb.set_message("Collecting files");
    let files = pb
        .wrap_iter(walkdir::WalkDir::new(p).into_iter())
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
                .load(&mut *db.lock())
                .expect("Error in loading old files")
        } else {
            tracks
                .select(path)
                //ignore files that are not in the path
                .filter(path.like(String::from("%") + p + "%"))
                .load(&mut *db.lock())
                .expect("Error in loading old files")
        });

        {
            let pb = ProgressBar::new(file_count as u64);
            pb.set_message("Updating tags");
            let style = ProgressStyle::default_spinner()
                .template(PROGRESSBAR_STYLE)
                .map_err(|_| String::from("Error in progressstyle"))?;
            pb.set_style(style);
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
            let style = ProgressStyle::default_spinner()
                .template(PROGRESSBAR_UNKNOWN_STYLE)
                .map_err(|_| String::from("Error in progressstyle"))?;
            pb2.set_style(style.progress_chars("#>-"));
            pb2.set_message("Deleting old unused entries");
            for i in pb2.wrap_iter(to_delete.iter()) {
                //println!("to delete: {}", i);
                diesel::delete(tracks)
                    .filter(path.eq(i))
                    .execute(&mut *db.lock())
                    .unwrap_or_else(|_| {
                        panic!("Error in deleting outdated database entries: {}", &i)
                    });
            }
            pb.finish_with_message("Done removing old entries");
        }
    }

    Ok(())
}

/// returns an id for a newly created playlist. Returns 0 if no playlists yet in db
#[must_use]
pub(crate) fn get_new_playlist_id(db: &DBPool) -> i32 {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use viola_common::schema::playlists::dsl::*;
    playlists
        .select(viola_common::schema::playlists::id)
        .order(viola_common::schema::playlists::id.desc())
        .load(&mut *db.lock())
        .ok()
        .and_then(|v: Vec<i32>| v.first().copied())
        .map_or(0, |i| i + 1)
}
