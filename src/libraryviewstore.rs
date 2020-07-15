use crate::db;
use crate::diesel::RunQueryDsl;
use crate::loaded_playlist;
use crate::types::*;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub value: String,
    pub optional: Option<i32>,
}

pub type Album = GeneralTreeViewJson<Track>;
pub type Artist = GeneralTreeViewJson<Album>;

impl From<(String, Option<i32>)> for Album {
    fn from(s: (String, Option<i32>)) -> Self {
        Album {
            value: s.0,
            children: vec![],
            optional: s.1,
        }
    }
}

impl From<String> for Artist {
    fn from(s: String) -> Self {
        Artist {
            value: s,
            optional: None,
            children: vec![],
        }
    }
}

/*#[derive(Debug, Serialize, Deserialize)]
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
}*/

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum PartialQueryLevelEnum {
    Artist = 0,
    Album = 1,
    Track = 2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartialQueryLevel {
    index: Vec<usize>,
    start: PartialQueryLevelEnum,
}

/// basic query function to model tracks with the apropiate values selected
fn basic_tree_query(
    pool: &DBPool,
    pql: &PartialQueryLevel,
) -> crate::schema::tracks::BoxedQuery<diesel::sqlite::Sqlite> {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, TextExpressionMethods};

    let mut query = tracks
        .filter(artist.is_not_null())
        .filter(artist.ne(""))
        .into_boxed();

    //if !pql.search.is_empty() {
    //    let s = String::from("%") + &pql.search + "%";
    //    query = query
    //        .filter(artist.like(s.clone()))
    //        .or_filter(album.like(s.clone()))
    //        .or_filter(title.like(s));
    //}

    let vals = match pql.start {
        PartialQueryLevelEnum::Artist => pql.index.iter().zip(
            [
                PartialQueryLevelEnum::Artist,
                PartialQueryLevelEnum::Album,
                PartialQueryLevelEnum::Track,
            ]
            .iter(),
        ),
        PartialQueryLevelEnum::Album => pql
            .index
            .iter()
            .zip([PartialQueryLevelEnum::Album, PartialQueryLevelEnum::Track].iter()),
        PartialQueryLevelEnum::Track => pql.index.iter().zip([PartialQueryLevelEnum::Track].iter()),
    };

    for (index, level) in vals {
        let mut level_query = tracks
            .filter(artist.is_not_null())
            .filter(artist.ne(""))
            .into_boxed();
        let p = pool.lock().expect("Error in lock");
        let group = match level {
            PartialQueryLevelEnum::Artist => level_query.select(artist).distinct(),
            PartialQueryLevelEnum::Album => level_query.select(album).distinct(),
            PartialQueryLevelEnum::Track => level_query.select(title).distinct(),
        };
        let val: String = group.offset(index.clone() as i64).first(p.deref()).unwrap();

        query = match level {
            PartialQueryLevelEnum::Artist => query.filter(artist.eq(&val)),
            PartialQueryLevelEnum::Album => query.filter(album.eq(&val)),
            PartialQueryLevelEnum::Track => query.filter(title.eq(&val)),
        };
    }
    query
}

pub fn load_query(pool: &DBPool, pql: &PartialQueryLevel) -> loaded_playlist::LoadedPlaylist {
    use diesel::RunQueryDsl;
    let p = pool.lock().expect("Error in lock");
    let items = basic_tree_query(pool, pql)
        .load(p.deref())
        .expect("Error in loading");
    //let name = match &pql.lvl {
    //    PartialQueryLevelEnum::Artist(x) => x.first(),
    //    PartialQueryLevelEnum::Album(x) => x.first(),
    //    PartialQueryLevelEnum::Track(x) => x.first(),
    //}
    //.cloned()
    //.unwrap_or_else(|| "Default".to_string());
    let name = "Test".to_string();

    loaded_playlist::LoadedPlaylist {
        id: -1,
        name,
        items,
        current_position: 0,
    }
}

/// Queries the tree but only returns not filled in results, i.e., children might be unpopulated
pub fn query_partial_tree(pool: &DBPool, pql: &PartialQueryLevel) -> Vec<Artist> {
    use crate::schema::tracks::dsl::*;
    use diesel::{QueryDsl, RunQueryDsl};
    let p = pool.lock().expect("Error in lock");
    let level = &pql.pql.first().unwrap().query_type;
    let query = basic_tree_query(pql);

    let sql = diesel::debug_query(&query).to_string();
    info!("query: {}", sql);

    match level {
        PartialQueryLevelEnum::Artist => {
            let res = query
                .select(artist)
                .distinct()
                .order_by(artist)
                .load(p.deref())
                .expect("Error in loading");
            res.into_iter()
                .map(|s: String| s.into())
                .collect::<Vec<Artist>>()
        }
        PartialQueryLevelEnum::Album => {
            let res = query
                .select((album, year))
                .distinct()
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
        PartialQueryLevelEnum::Track => {
            let res: Vec<(Option<i32>, String)> = query
                .select((tracknumber, title))
                .distinct()
                .order_by(tracknumber)
                .distinct()
                .load(p.deref())
                .expect("Error in loading album");

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
        }
    }
}

/// Queries the tree with the matching parameters, does not give us partials
pub fn query_tree(pool: &DBPool, pql: &PartialQueryLevel) -> Vec<Artist> {
    use diesel::RunQueryDsl;
    let p = pool.lock().expect("Error in lock");
    let query = basic_tree_query(pql);

    let mut q_tracks: Vec<db::Track> = query.load(p.deref()).expect("Error in DB");
}

// TODO This could be much more general by having the fill_fn in general
// TODO Try to make this iterator stuff, at the moment it doesn't need json
// TODO we need to not have the empty strings for stuff around
/*
pub fn get_artist_trees(pool: &DBPool) -> Vec<Artist> {
    use crate::schema::tracks::dsl::*;
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
