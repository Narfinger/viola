use toml;
use std::fs;
use std::collections::HashMap;
use diesel::QueryDsl;

fn read_smartplaylist(h: HashMap<String, String>) -> (String, impl QueryDsl) {
    use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, TextExpressionMethods};
    use schema::tracks::dsl::*;
    use diesel::sqlite::Sqlite;
    
    let mut s = tracks.into_boxed::<Sqlite>();
    let mut name = None;
    for (k,v) in h {
        match k.as_str() {
            "name" => { name = Some(v)},
            "artist_include" => { s = s.filter(artist.eq(v)); },
            "dir_include" => { s = s.filter(path.like(String::from("%") + &v + "%")); },
            "dir_exclude" => { s = s.filter(path.not_like(String::from("%") + &v + "%")); },
            "genre_include" => { s = s.filter(genre.eq(v)); },
            v => { panic!("We found a weird tag, we could not quite figure out: {}", v); },
        };
    }
    if let Some(n) = name {
        (n, s)
    } else {
        panic!("Did not find file");
    }
}

fn read_file(file: &str) -> Vec<(String,impl QueryDsl)> {
    let string = fs::read_to_string(file).unwrap();
    let s = toml::from_str::<Vec<HashMap<String,String>>>(&string).expect("Could not parse");

    s.into_iter().into_iter().map(read_smartplaylist).collect()
}

#[test]
fn test_query_output() {
    let res = read_file("tests/playlists.toml");
    assert!(res.len() == 2, "Did not read all playlists");
    let pl1 = &res[0];
    let pl2 = &res[1];
    assert!(false, "Playlist 1 did not parse correctly");
    assert!(false, "Playlist 2 did not parse correctly");
}