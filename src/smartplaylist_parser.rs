use toml;
use std::ops::Deref;
use std::fs;
use diesel::{QueryDsl, RunQueryDsl};
use diesel::sqlite::Sqlite;
use schema::tracks::dsl::*;
use preferences::prefs_base_dir;

use loaded_playlist::LoadedPlaylist;
use types::*;

pub struct SmartPlaylist {
    pub name: String,
    query: Vec<(Tag, String)>,
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

#[derive(Clone, Debug)]
pub enum Tag {
    DirExclude,
    DirInclude,
    ArtistInclude,
    GenreInclude,
}

fn build_iter(t: Tag, v: Option<Vec<String>>) -> impl Iterator<Item = (Tag, String)> {
    v.unwrap_or(vec![]).into_iter().map(move |v| (t.clone(), v))
}

fn construct_smartplaylist(smp: SmartPlaylistParsed) -> SmartPlaylist {
    let query = build_iter(Tag::DirExclude, smp.dir_exclude)
        .chain(
            build_iter(Tag::DirInclude, smp.dir_include)
            .chain(
                build_iter(Tag::ArtistInclude, smp.artist_include)
                .chain(
                    build_iter(Tag::GenreInclude, smp.genre_include)
                )
            )
        ).collect::<Vec<(Tag, String)>>();

    SmartPlaylist {name: smp.name, query: query }
}

pub trait LoadSmartPlaylist {
    fn load(&self, &DBPool) -> LoadedPlaylist;
}

impl LoadSmartPlaylist for SmartPlaylist {
    fn load(&self, pool: &DBPool) -> LoadedPlaylist {
        panic!("the order of operation is not quite good, filter would filter multiple things while we need some or and some and");
        use diesel::{ExpressionMethods, TextExpressionMethods};

        let db = pool.get().unwrap();     
        let mut s = tracks.into_boxed::<Sqlite>();
    
        for (k,v) in self.query.iter() {
            match k {
                //"name" => { name = Some(v)},
                Tag::ArtistInclude => { s = s.filter(artist.eq(v)); },
                Tag::DirInclude => { s = s.filter(path.like(String::from("%") + &v + "%")); },
                Tag::DirExclude => { s = s.filter(path.not_like(String::from("%") + &v + "%")); },
                Tag::GenreInclude => { s = s.filter(genre.eq(v)); },
            };
        }

        let res = s.load(db.deref()).expect("Error in loading smart playlist");

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