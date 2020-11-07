use crate::db;
use crate::loaded_playlist;
use crate::types::*;
use std::collections::HashMap;
use std::ops::Deref;
use viola_common::{Album, Artist, GeneralTreeViewJson, Track};

//#[derive(Debug, Serialize, Deserialize)]
//pub struct Track {
//    pub value: String,
//    pub optional: Option<i32>,
//}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialQueryLevel {
    pub lvl: PartialQueryLevelEnum,
    pub search: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum PartialQueryLevelEnum {
    /// We want to get all the possible artists
    Artist(Vec<String>),
    /// We want to get all the possible albums. If Some(x), only albums of artist x
    Album(Vec<String>),
    /// We want to get all the possible tracks. If Some((x,y)), only tracks in album y or artist x
    Track(Vec<String>),
}

/// basic query function to model tracks with the apropiate values selected
fn basic_tree_query(
    pql: &PartialQueryLevel,
) -> viola_common::schema::tracks::BoxedQuery<diesel::sqlite::Sqlite> {
    use diesel::{ExpressionMethods, QueryDsl, TextExpressionMethods};
    use viola_common::schema::tracks::dsl::*;

    let level = &pql.lvl;

    let mut query = tracks
        .filter(artist.is_not_null())
        .filter(artist.ne(""))
        .into_boxed();
    if !pql.search.is_empty() {
        let s = String::from("%") + &pql.search + "%";
        query = query
            .filter(artist.like(s.clone()))
            .or_filter(album.like(s.clone()))
            .or_filter(title.like(s));
    }

    match level {
        PartialQueryLevelEnum::Artist(_) => query.order_by(artist).distinct(),
        PartialQueryLevelEnum::Album(artist_value) => {
            if let [a] = artist_value.as_slice() {
                query.order_by(path).filter(artist.eq(a)).distinct()
            } else {
                query.order_by(album)
            }
        }
        PartialQueryLevelEnum::Track(artist_and_album) => {
            if let [artist_value, album_value] = artist_and_album.as_slice() {
                query
                    .order_by(path)
                    .filter(artist.eq(artist_value))
                    .filter(album.eq(album_value))
            } else if let [artist_value, album_value, title_value] = artist_and_album.as_slice() {
                query
                    .order_by(path)
                    .filter(artist.eq(artist_value))
                    .filter(album.eq(album_value))
                    .filter(title.eq(title_value))
            } else {
                query.order_by(title)
            }
        }
    }
}

pub fn load_query(pool: &DBPool, pql: &PartialQueryLevel) -> loaded_playlist::LoadedPlaylist {
    use diesel::RunQueryDsl;
    let p = pool.lock().expect("Error in lock");
    let items = basic_tree_query(pql)
        .load(p.deref())
        .expect("Error in loading");
    let name = match &pql.lvl {
        PartialQueryLevelEnum::Artist(x) => x.first(),
        PartialQueryLevelEnum::Album(x) => x.first(),
        PartialQueryLevelEnum::Track(x) => x.first(),
    }
    .cloned()
    .unwrap_or_else(|| "Default".to_string());

    loaded_playlist::LoadedPlaylist {
        id: -1,
        name,
        items,
        current_position: 0,
    }
}

/// Queries the tree but only returns not filled in results, i.e., children might be unpopulated
pub fn query_partial_tree(pool: &DBPool, pql: &PartialQueryLevel) -> Vec<Artist> {
    use diesel::{QueryDsl, RunQueryDsl};
    use viola_common::schema::tracks::dsl::*;
    let p = pool.lock().expect("Error in lock");
    let level = &pql.lvl;
    let query = basic_tree_query(pql);

    let sql = diesel::debug_query(&query).to_string();
    info!("query: {}", sql);

    match level {
        PartialQueryLevelEnum::Artist(_) => {
            let res = query
                .select(artist)
                .order_by(artist)
                .load(p.deref())
                .expect("Error in loading");
            res.into_iter()
                .map(|s: String| s.into())
                .collect::<Vec<Artist>>()
        }
        PartialQueryLevelEnum::Album(x) => {
            let res = query
                .select((album, year))
                .order_by(year)
                .distinct()
                .load(p.deref())
                .expect("Error in loading album");
            vec![Artist {
                value: "Default".to_string(),
                optional: None,
                children: res
                    .into_iter()
                    .map(|s: (String, Option<i32>)| s.into())
                    .collect::<Vec<Album>>(),
            }]
        }
        PartialQueryLevelEnum::Track(x) => {
            let res: Vec<(Option<i32>, String)> = query
                .select((tracknumber, title))
                .order_by(tracknumber)
                .distinct()
                .load(p.deref())
                .expect("Error in loading album");
            vec![]
            /*
            vec![Artist {
                value: "Default".to_string(),
                optional: None,
                children: vec![Album {
                    value: "Default".to_string(),
                    optional: None,
                    children: res
                        .into_iter()
                        .map(|(number, t)| Track {
                            value: t,
                            optional: number,
                        })
                        .collect::<Vec<Track>>(),
                }],
            }]
            */
        }
    }
}

/// Queries the tree with the matching parameters, does not give us partials
pub fn query_tree(pool: &DBPool, pql: &PartialQueryLevel) -> Vec<Artist> {
    use diesel::RunQueryDsl;
    let p = pool.lock().expect("Error in lock");
    let query = basic_tree_query(pql);

    let mut q_tracks: Vec<viola_common::Track> = query.load(p.deref()).expect("Error in DB");

    //Artist, Album, Track
    let mut hashmap: HashMap<String, HashMap<String, Vec<viola_common::Track>>> = HashMap::new();
    for t in q_tracks.drain(0..) {
        if let Some(ref mut artist_hash) = hashmap.get_mut(&t.artist) {
            if let Some(ref mut album_vec) = artist_hash.get_mut(&t.album) {
                album_vec.push(t);
            } else {
                let k = t.artist.clone();
                let v = vec![t];
                artist_hash.insert(k, v);
            }
        } else {
            let mut v = HashMap::new();
            let kb = t.album.clone();
            let ka = t.artist.clone();
            let v2 = vec![t];
            v.insert(kb, v2);
            hashmap.insert(ka, v);
        }
    }

    vec![]
    /*
    hashmap
        .drain()
        .map(
            |(k, mut m): (String, HashMap<String, Vec<viola_common::Track>>)| {
                let children = m
                    .drain()
                    .map(|(k2, v): (String, Vec<viola_common::Track>)| Album {
                        value: k2,
                        optional: None,
                        children: v
                            .into_iter()
                            .map(|v| Track {
                                value: v.title,
                                optional: v.tracknumber,
                            })
                            .collect(),
                    })
                    .collect();
                Artist {
                    value: k,
                    optional: None,
                    children,
                }
            },
        )
        .collect::<Vec<Artist>>()
        */
}

// TODO This could be much more general by having the fill_fn in general
// TODO Try to make this iterator stuff, at the moment it doesn't need json
// TODO we need to not have the empty strings for stuff around
/*
pub fn get_artist_trees(pool: &DBPool) -> Vec<Artist> {
    use viola_common::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
    let p = pool.lock().expect("Error in lock");
    tracks
        .select(artist)
        .filter(artist.is_not_null())
        .filter(artist.ne(""))
        .distinct()
        .order_by(artist)
        .load(p.deref())
        .expect("Error in DB")
        .into_iter()
        .map(move |t| track_to_artist(t, p.deref()))
        .take(10)
        .collect()
}
*/
