use crate::types::*;
use crate::{
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    loaded_playlist::LoadedPlaylist,
};
use diesel::TextExpressionMethods;
use itertools::{izip, Itertools};
use std::{collections::HashSet, ops::Deref};
use viola_common::TreeViewQuery;
use viola_common::{schema::tracks::dsl::*, TreeType};

/// produces the filter string, for sorting reasons we need the type_vec to be the first n of the types in the original query
/// where n is the current iteration depth
fn get_filter_string(
    new_bunch: &[viola_common::Track],
    current_ttype: &TreeType,
    index: &usize,
    recursion_depth: usize,
    type_vec: Vec<TreeType>,
) -> String {
    let mut new: Vec<viola_common::Track> = new_bunch.iter().map(|t| (*t).clone()).collect();
    let new_indices = (0..recursion_depth).collect();
    let query = TreeViewQuery {
        types: type_vec,
        indices: new_indices,
        search: None,
    };
    sort_tracks(&query, &mut new);

    let full_unique: Vec<&String> = new
        .iter()
        .map(|t| match current_ttype {
            TreeType::Artist => &t.artist,
            TreeType::Album => &t.album,
            TreeType::Track => &t.title,
            TreeType::Genre => &t.genre,
        })
        .unique()
        .collect();
    //println!("full unique {:?}", &full_unique);

    let st = (*full_unique.get(*index).unwrap()).to_owned();
    //let st = full_unique.get(*index).unwrap().clone().clone();

    st
}

fn basic_get_tracks(db: &DBPool, query: &TreeViewQuery) -> Vec<viola_common::Track> {
    //this function is currently to difficult to implement in diesel as we cannot clone boxed ty pes and otherwise we can cyclic type error
    let mut current_tracks = if let Some(ref search_string) = query.search {
        let mut track_set: HashSet<viola_common::Track> = HashSet::new();
        for val in &query.types {
            let new_tracks = match val {
                TreeType::Artist => tracks
                    .filter(artist.like(String::from("%") + &search_string + "%"))
                    .load::<viola_common::Track>(db.lock().deref())
                    .unwrap(),
                TreeType::Album => tracks
                    .filter(album.like(String::from("%") + &search_string + "%"))
                    .load::<viola_common::Track>(db.lock().deref())
                    .unwrap(),
                TreeType::Track => tracks
                    .filter(title.like(String::from("%") + &search_string + "%"))
                    .load::<viola_common::Track>(db.lock().deref())
                    .unwrap(),
                TreeType::Genre => tracks
                    .filter(genre.like(String::from("%") + &search_string + "%"))
                    .load::<viola_common::Track>(db.lock().deref())
                    .unwrap(),
            }
            .into_iter();
            // yes union is so weird to use that I don't know how to use it.
            for i in new_tracks {
                track_set.insert(i);
            }
        }
        track_set.into_iter().collect::<Vec<viola_common::Track>>()
    } else {
        tracks
            .filter(artist.ne(""))
            .load::<viola_common::Track>(db.lock().deref())
            .unwrap()
    };

    for (recursion_depth, (index, current_ttype)) in
        izip!(query.indices.iter(), query.types.iter(),).enumerate()
    {
        let filter_value = get_filter_string(
            &current_tracks,
            &current_ttype,
            index,
            recursion_depth,
            query.types.clone(),
        );
        info!(
            "recursion depth {}, index {}, current_ttype {:?}",
            &recursion_depth, &index, &current_ttype
        );
        info!("Filter value {}", &filter_value);
        current_tracks = match current_ttype {
            TreeType::Artist => current_tracks
                .into_iter()
                .filter(|t| t.artist == filter_value)
                .collect(),
            TreeType::Album => current_tracks
                .into_iter()
                .filter(|t| t.album == filter_value)
                .collect(),
            TreeType::Track => current_tracks
                .into_iter()
                .filter(|t| t.title == filter_value)
                .collect(),
            TreeType::Genre => current_tracks
                .into_iter()
                .filter(|t| t.genre == filter_value)
                .collect(),
        };
    }
    info!("Sorting tracks now");
    sort_tracks(query, &mut current_tracks);

    current_tracks
}

/// Returns a projection of `t` for which we sort our stuff, dependend on ttype and level
/// I would love to have this return a reference but because of the options inside it is unclear how to do it
fn sort_key_from_treetype<'a>(
    ttype: &'a Option<&'a TreeType>,
    t: &'a viola_common::Track,
    level: usize,
) -> String {
    match ttype {
        Some(&TreeType::Artist) => t.artist.to_owned(),
        Some(&TreeType::Album) => {
            if level == 0 {
                t.album.to_owned()
            } else {
                t.year.unwrap_or_default().to_string()
            }
        }
        Some(&TreeType::Genre) => t.genre.to_owned(),
        Some(&TreeType::Track) => {
            if level == 0 {
                t.title.to_owned()
            } else {
                t.path.to_string()
            }
        }
        None => t.artist.to_owned(),
    }
}

/// sorts the tracks according to the treeviewquery we have
/// TODO: This has the problem that we rarely want to sort albums by name but mostly by year.
/// But sometimes by name
fn sort_tracks(query: &TreeViewQuery, t: &mut [viola_common::Track]) {
    if query.indices.len() != 1 {
        let indexed = query.get_indexed_ttypes();
        t.sort_unstable_by(|x, y| {
            // We build a map of Ordering that compares all the keys in indexed.
            // Then we fold over this to use Ordering::Then to get the correct valuation
            let ordering = std::cmp::Ordering::Equal;
            indexed
                .iter()
                .enumerate()
                .map(|(level, ttype)| {
                    let xkey = sort_key_from_treetype(&Some(&ttype), x, level);
                    let ykey = sort_key_from_treetype(&Some(&ttype), y, level);
                    xkey.cmp(&ykey)
                })
                .fold(ordering, |acc, x| acc.then(x))
        });
    } else {
        t.sort_unstable_by_key(|t| t.path.clone());
    }

    //let ttype = query.get_after_last_ttype();
    //println!("ttype {:?}", ttype);
    //t.sort_by_cached_key(|x| sort_key_from_treetype(&ttype, &x));
}

/// custom strings that appear in the partial query view
fn track_to_partial_string(query: &TreeViewQuery, t: viola_common::Track) -> String {
    if query.indices.is_empty() {
        match query.types.get(0) {
            Some(TreeType::Artist) => t.artist,
            Some(TreeType::Album) => t.album,
            Some(TreeType::Track) => t.title,
            Some(TreeType::Genre) => t.genre,
            None => "None".to_string(),
        }
    } else if query.indices.len() == 1
        && query.types.get(0) == Some(&viola_common::TreeType::Artist)
        && query.types.get(1) == Some(&viola_common::TreeType::Album)
    {
        format!("{}-{}", t.year.unwrap_or(0), t.album)
    } else if query.indices.len() == 2
        && query.types.get(0) == Some(&viola_common::TreeType::Artist)
        && query.types.get(1) == Some(&viola_common::TreeType::Album)
        && query.types.get(2) == Some(&viola_common::TreeType::Track)
    {
        format!("{}-{}", t.tracknumber.unwrap_or(0), t.title)
    } else {
        let last = query.get_after_last_ttype();
        match last {
            Some(TreeType::Artist) => t.artist,
            Some(TreeType::Album) => t.album,
            Some(TreeType::Track) => t.title,
            Some(TreeType::Genre) => t.genre,
            None => t.title,
        }
    }
}

/// extracts a playlistname from the query
fn get_playlist_name(query: &TreeViewQuery, t: &[viola_common::Track]) -> String {
    let mut res = if let Some(ref search) = query.search {
        search.to_owned()
    } else {
        let last = query.get_after_last_ttype();
        let first_track = t.get(0);
        match last {
            Some(TreeType::Artist) => first_track.map(|t| t.artist.to_owned()),
            Some(TreeType::Album) => first_track.map(|t| t.album.to_owned()),
            Some(TreeType::Genre) => first_track.map(|t| t.genre.to_owned()),
            Some(TreeType::Track) => first_track.map(|t| t.title.to_owned()),
            None => None,
        }
        .unwrap_or_else(|| "Foo".to_owned())
    };
    res.truncate(10);
    res
}

pub(crate) fn partial_query(db: &DBPool, query: &TreeViewQuery) -> Vec<String> {
    let t = basic_get_tracks(db, query);
    t.into_iter()
        .map(|t| track_to_partial_string(query, t))
        .unique()
        .collect()
}

/// produces a LoadedPlaylist frrom a treeviewquery
pub(crate) fn load_query(db: &DBPool, query: &TreeViewQuery) -> LoadedPlaylist {
    let t = basic_get_tracks(db, query);
    LoadedPlaylist {
        id: -1,
        name: get_playlist_name(query, &t),
        current_position: 0,
        items: t,
    }
}

#[cfg(test)]
mod test {
    use std::{fs, sync::Arc};

    use parking_lot::Mutex;

    use super::*;
    use crate::db::{self, NewTrack};

    embed_migrations!("migrations/");
    fn fill_db(db: &diesel::SqliteConnection) {
        #[derive(Deserialize)]
        struct Obj {
            newtracks: Vec<NewTrack>,
        }

        let string = fs::read_to_string("tests/tracks.toml").unwrap();
        let val = toml::from_str::<Obj>(&string).expect("Could not parse");

        diesel::insert_into(tracks)
            .values(&val.newtracks)
            .execute(db)
            .unwrap();
    }

    fn setup_db_connection() -> diesel::SqliteConnection {
        let conn = <diesel::SqliteConnection as diesel::Connection>::establish(":memory:")
            .map_err(|_| String::from("DB Connection error"))
            .unwrap();
        embedded_migrations::run(&conn).expect("Could not run migration");
        fill_db(&conn);
        conn
    }

    #[test]
    fn test_partial_strings_depth0() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Artist, TreeType::Album, TreeType::Track],
            indices: vec![],
            search: None,
        };
        let res = partial_query(&db, &query);
        assert_eq!(res[0], "2Cellos");
    }

    #[test]
    fn test_partial_strings_depth0_alt() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Artist, TreeType::Album, TreeType::Track],
            indices: vec![],
            search: None,
        };
        let res = partial_query(&db, &query);
        println!("res {:?}", res);
        assert_eq!(res[1], "Apocalyptica");
    }

    #[test]
    fn test_partial_strings_depth0_search() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Artist, TreeType::Album, TreeType::Track],
            indices: vec![],
            search: Some("2Cel".to_string()),
        };
        let res = partial_query(&db, &query);
        assert_eq!(res[0], "2Cellos");
    }

    #[test]
    fn test_partial_strings_depth1() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Artist, TreeType::Album, TreeType::Track],
            indices: vec![1],
            search: None,
        };
        let res = partial_query(&db, &query);
        let exp_res: Vec<String> = vec![
            "1996-Plays Metallica by Four Cellos",
            "1998-Inquisition Symphony",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        assert_eq!(res, exp_res);
    }

    #[test]
    fn test_partial_strings_depth2() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Artist, TreeType::Album, TreeType::Track],
            indices: vec![1, 0],
            search: None,
        };
        let res = partial_query(&db, &query);
        let exp_res: Vec<String> = vec![
            "1-Enter Sandman",
            "2-Master of Puppets",
            "3-Harvester of Sorrow",
            "4-The Unforgiven",
            "5-Sad But True",
            "6-Creeping Death",
            "7-Wherever I May Roam",
            "8-Welcome Home",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        assert_eq!(res, exp_res);
    }

    #[test]
    fn test_partial_strings_album_track_depth0() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Album, TreeType::Track],
            indices: vec![],
            search: None,
        };
        let res = partial_query(&db, &query);
        assert_eq!(res[3], "Plays Metallica by Four Cellos");
    }

    #[test]
    fn test_partial_strings_album_track_depth0_alt() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Album, TreeType::Track],
            indices: vec![],
            search: None,
        };
        let res = partial_query(&db, &query);
        assert_eq!(res[2], "Metallica");
    }

    #[test]
    fn test_partial_strings_album_track_depth1() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Album, TreeType::Track],
            indices: vec![3],
            search: None,
        };
        let res = partial_query(&db, &query);
        let exp_res: Vec<String> = vec![
            "Enter Sandman",
            "Master of Puppets",
            "Harvester of Sorrow",
            "The Unforgiven",
            "Sad But True",
            "Creeping Death",
            "Wherever I May Roam",
            "Welcome Home",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        assert_eq!(res, exp_res);
    }

    #[test]
    fn test_partial_strings_track_depth0() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![TreeType::Track],
            indices: vec![],
            search: None,
        };
        let res = partial_query(&db, &query);
        assert_eq!(res[5714], "Enter Sandman");
    }

    #[test]
    fn test_partial_strings_genre_depth0() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![
                TreeType::Genre,
                TreeType::Artist,
                TreeType::Album,
                TreeType::Track,
            ],
            indices: vec![],
            search: None,
        };
        let res = partial_query(&db, &query);
        assert_eq!(res[26], "Cello Rock");
    }

    #[test]
    fn test_partial_strings_genre_depth1() {
        let db = Arc::new(Mutex::new(setup_db_connection()));
        let query = TreeViewQuery {
            types: vec![
                TreeType::Genre,
                TreeType::Artist,
                TreeType::Album,
                TreeType::Track,
            ],
            indices: vec![26],
            search: None,
        };
        let res = partial_query(&db, &query);
        let exp_res: Vec<String> = vec![
            "Apocalyptica",
            "Apocalyptica feat. Tomoyasu Hotei",
            "Apocalyptica feat. Corey Taylor",
            "Apocalyptica feat. Till Lindemann",
            "Apocalyptica feat. Dave Lombardo",
            "Apocalyptica feat. Adam Gontier",
            "Apocalyptica feat. Cristina Scabbia",
            "Melora Creager",
            "Rasputina",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        assert_eq!(res, exp_res);
    }
}
