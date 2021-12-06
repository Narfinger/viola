use diesel::sqlite::Sqlite;
use diesel::{QueryDsl, RunQueryDsl};
use preferences::prefs_base_dir;
use rand::prelude::*;
use std::fs;
use std::ops::Deref;
use viola_common::schema::tracks::dsl::*;

use crate::db;
use crate::loaded_playlist::LoadedPlaylist;
use crate::types::*;
use viola_common::Track;

#[derive(Debug)]
pub struct SmartPlaylist {
    pub name: String,
    random: bool,
    include_query: Vec<IncludeTag>,
    exclude_query: Vec<ExcludeTag>,
}

#[derive(Deserialize, Debug)]
struct SmartPlaylistConfig {
    _test: String,
    smartplaylist: Vec<SmartPlaylistParsed>,
}

#[derive(Debug, Deserialize)]
struct SmartPlaylistParsed {
    name: String,
    random: Option<bool>,
    dir_exclude: Option<Vec<String>>,
    dir_include: Option<Vec<String>>,
    album_include: Option<Vec<String>>,
    artist_include: Option<Vec<String>>,
    genre_include: Option<Vec<String>>,
    play_count_least_include: Option<i32>,
    play_count_exact_include: Option<i32>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum IncludeTag {
    Dir(Vec<String>),
    Album(Vec<String>),
    Artist(Vec<String>),
    Genre(Vec<String>),
    PlayCountLeast(i32),
    PlayCountExact(i32),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExcludeTag {
    Dir(Vec<String>),
}

fn construct_smartplaylist(smp: SmartPlaylistParsed) -> SmartPlaylist {
    /// returns None if the option is none or the vector in it empty
    fn insert_vec_value(v: Option<Vec<String>>) -> Option<Vec<String>> {
        if let Some(value) = v {
            if !value.is_empty() {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Inserts `vec` into `pushto` with the tag `value`
    macro_rules! vec_option_insert {
        ($value: expr, $vec: expr, $pushto: expr) => {
            if let Some(v) = insert_vec_value($vec) {
                $pushto.push($value(v));
            }
        };
    }

    let random = smp.random.unwrap_or(false);
    let mut include_query = Vec::new();

    vec_option_insert!(IncludeTag::Dir, smp.dir_include, include_query);
    vec_option_insert!(IncludeTag::Artist, smp.artist_include, include_query);
    vec_option_insert!(IncludeTag::Album, smp.album_include, include_query);
    vec_option_insert!(IncludeTag::Genre, smp.genre_include, include_query);

    if let Some(v) = smp.play_count_least_include {
        include_query.push(IncludeTag::PlayCountLeast(v));
    }

    if let Some(v) = smp.play_count_exact_include {
        include_query.push(IncludeTag::PlayCountExact(v));
    }

    let mut exclude_query = Vec::new();
    vec_option_insert!(ExcludeTag::Dir, smp.dir_exclude, exclude_query);

    SmartPlaylist {
        name: smp.name,
        random,
        include_query,
        exclude_query,
    }
}

pub trait LoadSmartPlaylist {
    fn load(&self, _: &DBPool) -> LoadedPlaylist;
}

fn matched_with_exclude(t: &Track, h: &[ExcludeTag]) -> bool {
    h.iter().any(|k| match k {
        ExcludeTag::Dir(v) => v.iter().any(|value| t.path.contains(value)),
    })
}

impl LoadSmartPlaylist for SmartPlaylist {
    /// This is kind of weird because we need to construct the vector instead of the query.
    /// I would love to use union of queries but it doesn't seem to work in diesel
    fn load(&self, db: &DBPool) -> LoadedPlaylist {
        use diesel::{ExpressionMethods, TextExpressionMethods};

        let basic: Vec<Track> = if self.include_query.is_empty() {
            tracks
                .load(db.lock().deref())
                .expect("Error in loading smart playlist")
        } else {
            self.include_query
                .iter()
                .map(|k| match k {
                    IncludeTag::Album(v) => {
                        let mut s = tracks.into_boxed::<Sqlite>();
                        for value in v {
                            s = s.or_filter(album.eq(value))
                        }
                        s.load(db.lock().deref())
                            .expect("Error in loading smart playlist")
                    }
                    IncludeTag::Artist(v) => {
                        let mut s = tracks.into_boxed::<Sqlite>();
                        for value in v {
                            s = s.or_filter(artist.eq(value));
                        }
                        //println!("Query ArtistInclude: {:?}", debug_query(&s));
                        s.load(db.lock().deref())
                            .expect("Error in loading smart playlist")
                    }
                    IncludeTag::Dir(v) => {
                        let mut s = tracks.into_boxed::<Sqlite>();
                        for value in v {
                            s = s.or_filter(path.like(String::from("%") + value + "%"));
                        }
                        //println!("Query DirInclude: {:?}", debug_query(&s));
                        s.load(db.lock().deref())
                            .expect("Error in loading smart playlist")
                    }
                    IncludeTag::Genre(v) => {
                        let mut s = tracks.into_boxed::<Sqlite>();
                        for value in v {
                            s = s.or_filter(genre.eq(value));
                        }
                        //println!("Query GenreInclude: {:?}", debug_query(&s));
                        s.load(db.lock().deref())
                            .expect("Error in loading smart playlist")
                    }
                    IncludeTag::PlayCountLeast(v) => {
                        let mut s = tracks.into_boxed::<Sqlite>();
                        s = s.or_filter(playcount.ge(v));

                        s.load(db.lock().deref())
                            .expect("Error in loading smart playlist")
                    }
                    IncludeTag::PlayCountExact(v) => {
                        let mut s = tracks.into_boxed::<Sqlite>();
                        s = s.or_filter(playcount.eq(v));

                        s.load(db.lock().deref())
                            .expect("Error in loading smart playlist")
                    }
                })
                .flat_map(std::iter::IntoIterator::into_iter)
                .collect::<Vec<Track>>()
        };

        let mut filtered = basic
            .iter()
            .filter(|t| {
                // remember this keeps elements with true and removes other elements
                self.exclude_query.is_empty() | !matched_with_exclude(t, &self.exclude_query)
            })
            .cloned()
            .collect::<Vec<Track>>();

        filtered.dedup();

        if self.random {
            let mut rng = thread_rng();
            filtered.shuffle(&mut rng);
        } else {
            filtered.sort_unstable_by(|u, v| u.path.cmp(&v.path));
        }

        LoadedPlaylist {
            id: db::get_new_playlist_id(db),
            name: self.name.clone(),
            items: filtered,
            current_position: 0,
        }
    }
}

fn read_file(file: &str) -> Vec<SmartPlaylist> {
    let string = fs::read_to_string(file).unwrap();
    let s = toml::from_str::<SmartPlaylistConfig>(&string).expect("Could not parse");

    s.smartplaylist
        .into_iter()
        .map(construct_smartplaylist)
        .collect()
}

pub fn construct_smartplaylists_from_config() -> Vec<SmartPlaylist> {
    let mut p = prefs_base_dir().expect("Could not find base dir");
    p.push("viola");
    p.push("smartplaylists.toml");
    if p.exists() {
        let st = p.to_str().expect("Could not convert");
        read_file(st)
    } else {
        vec![]
    }
}

#[test]
fn test_query_output() {
    let string = fs::read_to_string("tests/playlists.toml").unwrap();
    println!("{}\n\n", string);
    let s = toml::from_str::<SmartPlaylistConfig>(&string).unwrap();
    println!("{:?}", s);
    /*
    let res = read_file("tests/playlists.toml");
    assert!(res.len() == 2, "Did not read all playlists");
    let pl1 = &res[0];
    let pl2 = &res[1];
    assert!(false, "Playlist 1 did not parse correctly");
    assert!(false, "Playlist 2 did not parse correctly");
    */
}
