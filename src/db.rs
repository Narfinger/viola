use app_dirs::*;
use diesel;
use diesel::r2d2;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use schema::tracks;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::ops::Deref;
use std::path::Path;
use taglib;
use types::{DBPool, APP_INFO};
use walkdir;

#[derive(AsChangeset, Debug, Identifiable, Queryable, Clone)]
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
    let manager = diesel::r2d2::ConnectionManager::<diesel::SqliteConnection>::new(
        db_file.to_str().expect("Error in converting string"),
    );
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

fn check_file(s: &Result<walkdir::DirEntry, walkdir::Error>) -> bool {
    if let Ok(ref sp) = *s {
        if sp.file_type().is_file() {
            Some(true) == sp.path().extension().map(|ex| {
                vec!["ogg", "flac", "mp3", "wma", "aac", "opus"].contains(&ex.to_str().unwrap())
            })
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
    let p = Path::new(s);
    let jpg = p.with_file_name("cover.jpg");
    let png = p.with_file_name("cover.png");
    if jpg.exists() {
        Some(jpg)
    } else if png.exists() {
        Some(png)
    } else {
        None
    }.and_then(|s| s.to_str().map(String::from))
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
            title: tags.title(),
            artist: tags.artist(),
            album: tags.album(),
            genre: tags.genre(),
            tracknumber: number_zero_to_option(tags.track()),
            year: number_zero_to_option(tags.year()),
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

fn insert_track(s: &str, pool: &DBPool) -> Result<(), String> {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SaveChangesDsl};
    use schema::tracks::dsl::*;

    let db = pool.get().unwrap();

    let new_track = construct_track_from_path(s)?;
    let old_track_perhaps = tracks
        .filter(path.eq(&new_track.path))
        .get_result::<Track>(db.deref());

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
                .save_changes::<Track>(db.deref())
                .map(|_| ())
                .map_err(|err| format!("Error in updateing for track {}, See full: {:?}", s, err))
        }
    } else {
        diesel::insert_into(tracks)
            .values(&new_track)
            .execute(db.deref())
            .map(|_| ())
            .map_err(|err| format!("Insertion Error for track {}, See full: {:?}", s, err))
    }
}

pub fn build_db(path: &str, pool: &DBPool) -> Result<(), String> {
    let files = walkdir::WalkDir::new(&path)
        .into_iter()
        .filter(check_file)
        .map(|i| String::from(i.unwrap().path().to_str().unwrap()))
        .collect::<HashSet<String>>();

    let file_count = walkdir::WalkDir::new(&path)
        .into_iter()
        .filter(check_file)
        .map(|i| String::from(i.unwrap().path().to_str().unwrap()))
        .count();

    let db = pool.get().unwrap();
    {
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        use schema::tracks::dsl::*;
        let old_files: HashSet<String> = HashSet::from_iter(
            tracks
                .select(path)
                .load(db.deref())
                .expect("Error in loading old files"),
        );

        /// TODO switch this to par_iter or something
        {
            let pb = ProgressBar::new(file_count as u64);
            pb.set_message("Updating files");
            pb.set_style(ProgressStyle::default_bar()
                                  .template("[{elapsed_precise}] {msg} {spinner:.green} {bar:100.green/blue} {pos:>7}/{len:7} ({percent}%)")
                                  .progress_chars("#>-"));
            let res = pb
                .wrap_iter(files.iter().map(|s| insert_track(s, pool)))
                .collect::<Result<(), String>>();
            pb.finish_with_message("Done Updating");

            if let Err(err) = res {
                println!("Error in updating database");
                println!("{}", err);
                panic!("Aborting");
            }
        }

        {
            println!("Deleting old database entries");
            let pb = ProgressBar::new_spinner();
            pb.set_message("Computing Difference to old database");
            let to_delete = old_files.difference(&files);
            pb.finish();

            let pb = ProgressBar::new_spinner();
            pb.set_message("Deleting old unused entries");
            for i in to_delete {
                println!("to delete: {}", i);
                diesel::delete(tracks)
                    .filter(path.eq(&i))
                    .execute(db.deref())
                    .unwrap_or_else(|_| {
                        panic!("Error in deleting outdated database entries: {}", &i)
                    });
            }
            pb.finish_with_message("Done removing old entries");
        }
    }

    Ok(())
}
