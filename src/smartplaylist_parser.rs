use toml;
use std;
use std::ops::Deref;
use std::fs;
use std::rc::Rc;
use std::collections::HashMap;
use diesel;
use diesel::{QueryDsl, RunQueryDsl};
use diesel::sqlite::Sqlite;
use schema;
use schema::tracks::dsl::*;

use loaded_playlist::LoadedPlaylist;
use types::*;

pub struct SmartPlaylist<'a> {
    pub name: String,
    pub query: schema::tracks::BoxedQuery<'a, Sqlite>,
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
enum Tag {
    DirExclude,
    DirInclude,
    ArtistInclude,
    GenreInclude,
}

fn build_iter(t: Tag, v: Option<Vec<String>>) -> impl Iterator<Item = (Tag, String)> {
    v.unwrap_or(vec![]).into_iter().map(move |v| (t.clone(), v))
}

impl IntoIterator for SmartPlaylistParsed {
    type Item = (Tag, String);
    type IntoIter = Box<Iterator<Item = (Tag, String)>>;
    fn into_iter(self) -> Self::IntoIter {
        Box::new(
            build_iter(Tag::DirExclude, self.dir_exclude)
            .chain(
                build_iter(Tag::DirInclude, self.dir_include)
                .chain(
                    build_iter(Tag::ArtistInclude, self.artist_include)
                    .chain(
                        build_iter(Tag::GenreInclude, self.genre_include)
                    )
                )
            )
        )
    }
}

pub trait LoadSmartPlaylist {
    fn load(&self, &DBPool) -> LoadedPlaylist;
}

impl<'a> LoadSmartPlaylist for SmartPlaylist<'a> {
    fn load(&self, pool: &DBPool) -> LoadedPlaylist {
        let db = pool.get().unwrap();
        let res = self.query.load(db.deref()).unwrap();
        LoadedPlaylist {
            id: None,
            name: self.name.clone(),
            items: res,
            current_position: 0,
        }
    }
}

fn read_smartplaylist<'a>(sm: SmartPlaylistParsed) -> SmartPlaylist<'a> {
    use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, TextExpressionMethods};
    
    let mut s = tracks.into_boxed::<Sqlite>();
    let name = sm.name.clone();
    
    for (k,v) in sm {
        match k {
            //"name" => { name = Some(v)},
            Tag::ArtistInclude => { s = s.filter(artist.eq(v)); },
            Tag::DirInclude => { s = s.filter(path.like(String::from("%") + &v + "%")); },
            Tag::DirExclude => { s = s.filter(path.not_like(String::from("%") + &v + "%")); },
            Tag::GenreInclude => { s = s.filter(genre.eq(v)); },
            v => { panic!("We found a weird tag, we could not quite figure out: {:?}", v); },
        };
    }

    SmartPlaylist {
        name: name,
        query: s,
    }

    

    //if let Some(n) = name {
    //    (n, s)
    //} else {
    //    panic!("Did not find file");
    //}
}

fn read_file(file: &str) -> Vec<SmartPlaylist> {
    let string = fs::read_to_string(file).unwrap();
    let s = toml::from_str::<SmartPlaylistConfig>(&string).expect("Could not parse");

    s.smartplaylist.into_iter().map(read_smartplaylist).collect()
}

pub fn construct_smartplaylists_from_config<'a>() -> Vec<SmartPlaylist<'a>> {
    panic!("not yet implemented");
    vec![]
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