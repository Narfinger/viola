use toml;
use std;
use std::fs;
use std::collections::HashMap;
use diesel;
use diesel::QueryDsl;
use diesel::sqlite::Sqlite;
use schema::tracks::dsl::*;

#[derive(Deserialize, Debug)]
struct SmartPlaylistConfig {
    test: String,
    smartplaylist: Vec<SmartPlaylist>,
}

#[derive(Debug, Deserialize)]
struct SmartPlaylist {
    name: String,
    dir_exclude: Option<Vec<String>>,
    dir_include: Option<Vec<String>>,
    artist_include: Option<Vec<String>>,
    genre_include: Option<Vec<String>>,
}

enum Tag {
    DirExclude,
    DirInclude,
    ArtistInclude,
    GenreInclude,
}

fn build_iter(t: Tag, v: Option<Vec<String>>) -> impl Iterator {
    v.unwrap_or(vec![]).into_iter().map(|v| (t, v))
}

impl IntoIterator for SmartPlaylist {
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

fn read_smartplaylist(sm: SmartPlaylist) -> (String, impl QueryDsl) {
    use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, TextExpressionMethods};
    
    let mut s = tracks.into_boxed::<Sqlite>();
    let mut name = None;
    /*for (k,v) in s {
        match k {
            //"name" => { name = Some(v)},
            Tag::ArtistInclude => { s = s.filter(artist.eq(v)); },
            Tag::DirInclude => { s = s.filter(path.like(String::from("%") + &v + "%")); },
            Tag::DirExclude => { s = s.filter(path.not_like(String::from("%") + &v + "%")); },
            Tag::GenreInclude => { s = s.filter(genre.eq(v)); },
            v => { panic!("We found a weird tag, we could not quite figure out: {}", v); },
        };
    }*/
    if let Some(n) = name {
        (n, s)
    } else {
        panic!("Did not find file");
    }
}

fn read_file(file: &str) -> Vec<(String, impl QueryDsl)> {
    let string = fs::read_to_string(file).unwrap();
    let s = toml::from_str::<SmartPlaylistConfig>(&string).expect("Could not parse");

    s.smartplaylist.into_iter().map(read_smartplaylist).collect()
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