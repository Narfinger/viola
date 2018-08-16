use diesel::sqlite::Sqlite;
use diesel::{QueryDsl, RunQueryDsl};
use preferences::prefs_base_dir;
use rand::{thread_rng, Rng};
use schema::tracks::dsl::*;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::hash::Hash;
use toml;

use db::Track;
use loaded_playlist::LoadedPlaylist;
use types::*;

#[derive(Debug)]
pub struct SmartPlaylist {
    pub name: String,
    random: bool,
    include_query: HashMap<IncludeTag, Vec<String>>,
    exclude_query: HashMap<ExcludeTag, Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct SmartPlaylistConfig {
    test: String,
    smartplaylist: Vec<SmartPlaylistParsed>,
}

#[derive(Debug, Deserialize)]
struct SmartPlaylistParsed {
    name: String,
    random: Option<bool>,
    dir_exclude: Option<Vec<String>>,
    dir_include: Option<Vec<String>>,
    artist_include: Option<Vec<String>>,
    genre_include: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum IncludeTag {
    Dir,
    Artist,
    Genre,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExcludeTag {
    Dir,
}

fn construct_smartplaylist(smp: SmartPlaylistParsed) -> SmartPlaylist {
    fn query_insert<T: Eq + Hash>(v: Option<Vec<String>>, tag: T, m: &mut HashMap<T, Vec<String>>) {
        if let Some(p) = v {
            if !p.is_empty() {
                m.insert(tag, p);
            }
        }
    }

    let random = smp.random.unwrap_or(false);
    
    let mut include_query = HashMap::new();
    query_insert(smp.dir_include, IncludeTag::Dir, &mut include_query);
    query_insert(smp.genre_include, IncludeTag::Genre, &mut include_query);

    let mut exclude_query = HashMap::new();
    query_insert(smp.dir_exclude, ExcludeTag::Dir, &mut exclude_query);

    SmartPlaylist {
        name: smp.name,
        random,
        include_query,
        exclude_query,
    }
}

pub trait LoadSmartPlaylist {
    fn load(&self, &DBPool) -> LoadedPlaylist;
}

fn matched_with_exclude(t: &Track, h: &HashMap<ExcludeTag, Vec<String>>) -> bool {
    h
    .iter()
    .any(|(k,v)| match k {
        ExcludeTag::Dir => {
            v.iter()
                .any(|value| {
                    t.path.contains(value)
                })
        }
    })
}

use diesel::debug_query;
impl LoadSmartPlaylist for SmartPlaylist {
    /// This is kind of weird because we need to construct the vector instead of the query.
    /// I would love to use union of queries but it doesn't seem to work in diesel
    fn load(&self, pool: &DBPool) -> LoadedPlaylist {
        use db::Track;
        use diesel::{ExpressionMethods, TextExpressionMethods};

        let mut filtered = self
            .include_query
            .iter()
            .map(|(k, v)| match k {
                IncludeTag::Artist => {
                    let mut s = tracks.into_boxed::<Sqlite>();
                    for value in v {
                        s = s.or_filter(artist.eq(value));
                    }
                    let db = pool.get().expect("DB Error");
                    //println!("Query ArtistInclude: {:?}", debug_query(&s));
                    s.load(db.deref()).expect("Error in loading smart playlist")
                }
                IncludeTag::Dir => {
                    let mut s = tracks.into_boxed::<Sqlite>();
                    for value in v {
                        s = s.or_filter(path.like(String::from("%") + &value + "%"));
                    }
                    let db = pool.get().expect("DB Error");
                    //println!("Query DirInclude: {:?}", debug_query(&s));
                    s.load(db.deref()).expect("Error in loading smart playlist")
                }
                IncludeTag::Genre => {
                    let mut s = tracks.into_boxed::<Sqlite>();
                    for value in v {
                        s = s.or_filter(genre.eq(value));
                    }
                    let db = pool.get().expect("DB Error");
                    //println!("Query GenreInclude: {:?}", debug_query(&s));
                    s.load(db.deref()).expect("Error in loading smart playlist")
                }
            }).flat_map(|v| v.into_iter())
            .filter(|t| {  // remember this keeps elements with true and removes other elements
                self.exclude_query.is_empty() | 
                !matched_with_exclude(t, &self.exclude_query)
            })
            .collect::<Vec<Track>>();

        if self.random {
            let mut rng = thread_rng();
            rng.shuffle(&mut filtered);
        } else {
            filtered.sort_unstable_by(|u, v| u.path.cmp(&v.path));
        }

        LoadedPlaylist {
            id: None,
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
