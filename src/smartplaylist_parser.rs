use toml;
use std::ops::Deref;
use std::fs;
use std::collections::HashMap;
use diesel::{QueryDsl, RunQueryDsl};
use diesel::sqlite::Sqlite;
use schema::tracks::dsl::*;
use preferences::prefs_base_dir;

use loaded_playlist::LoadedPlaylist;
use types::*;

pub struct SmartPlaylist {
    pub name: String,
    query: HashMap<Tag, Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct SmartPlaylistConfig {
    test: String,
    smartplaylist: Vec<SmartPlaylistParsed>,
}

#[derive(Debug, Deserialize)]
struct SmartPlaylistParsed {
    name: String,
    dir_exclude: Option<Vec<String>>,
    dir_include: Option<Vec<String>>,
    artist_include: Option<Vec<String>>,
    genre_include: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Tag {
    DirExclude,
    DirInclude,
    ArtistInclude,
    GenreInclude,
}

fn construct_smartplaylist(smp: SmartPlaylistParsed) -> SmartPlaylist {
    fn query_insert(v: Option<Vec<String>>, tag: Tag, m: &mut HashMap<Tag, Vec<String>>) {
    if let Some(p) = v {
        if !p.is_empty() {
            m.insert(tag, p);
        }
    } 
} 
    let mut query = HashMap::new();
    query_insert(smp.dir_exclude,   Tag::DirExclude, &mut query);
    query_insert(smp.dir_include,   Tag::DirInclude, &mut query);
    query_insert(smp.genre_include, Tag::GenreInclude, &mut query);
    SmartPlaylist {name: smp.name, query: query }
}

pub trait LoadSmartPlaylist {
    fn load(&self, &DBPool) -> LoadedPlaylist;
}

use diesel::debug_query;
impl LoadSmartPlaylist for SmartPlaylist {
    /// This is kind of weird because we need to construct the vector instead of the query.
    /// I would love to use union of queries but it doesn't seem to work in diesel
    fn load(&self, pool: &DBPool) -> LoadedPlaylist {
        use db::Track;
        use diesel::{ExpressionMethods, TextExpressionMethods};

        let res = self.query.iter().map(|(k,v)| {
            match k {
                Tag::ArtistInclude => {
                    let mut s = tracks.into_boxed::<Sqlite>();
                    for value in v {
                        s = s.or_filter(artist.eq(value));
                    }
                    let db = pool.get().expect("DB Error");
                    println!("Query ArtistInclude: {:?}", debug_query(&s));
                    s.load(db.deref()).expect("Error in loading smart playlist")
                },
                Tag::DirInclude => {
                    let mut s = tracks.into_boxed::<Sqlite>();
                    for value in v {
                        s = s.or_filter(path.like(String::from("%") + &value + "%"));
                    }
                    let db = pool.get().expect("DB Error");
                    println!("Query DirInclude: {:?}", debug_query(&s));
                    s.load(db.deref()).expect("Error in loading smart playlist")
                },
                Tag::DirExclude => {
                    let mut s = tracks.into_boxed::<Sqlite>();
                    for value in v {
                        s = s.or_filter(path.not_like(String::from("%") + &value + "%"));
                    }
                    let db = pool.get().expect("DB Error");
                    println!("Query DirExclude: {:?}", debug_query(&s));
                    s.load(db.deref()).expect("Error in loading smart playlist")
                },
                Tag::GenreInclude => {
                    let mut s = tracks.into_boxed::<Sqlite>();
                    for value in v {
                        s = s.or_filter(genre.eq(value));
                    }
                    let db = pool.get().expect("DB Error");
                    println!("Query GenreInclude: {:?}", debug_query(&s));
                    s.load(db.deref()).expect("Error in loading smart playlist")
                },
            }
        })
        .flat_map(|v| v.into_iter())
        .collect::<Vec<Track>>();

        LoadedPlaylist {
            id: None,
            name: self.name.clone(),
            items: res,
            current_position: 0,
        }
    }
}

fn read_file(file: &str) -> Vec<SmartPlaylist> {
    let string = fs::read_to_string(file).unwrap();
    let s = toml::from_str::<SmartPlaylistConfig>(&string).expect("Could not parse");

    s.smartplaylist.into_iter().map(construct_smartplaylist).collect()
}

pub fn construct_smartplaylists_from_config<'a>() -> Vec<SmartPlaylist> {
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
