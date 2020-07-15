use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods};
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

#[derive(Clone, Debug, Serialize_repr, Deserialize_repr)]
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

fn index_to_level_slice(pql: &PartialQueryLevel) -> Vec<(usize, PartialQueryLevelEnum)> {
    let start = &pql.start;
    let vec = pql.index.clone();
    let index = vec.iter();
    match start {
        PartialQueryLevelEnum::Artist => index.zip(
            [
                PartialQueryLevelEnum::Artist,
                PartialQueryLevelEnum::Album,
                PartialQueryLevelEnum::Track,
            ]
            .iter(),
        ),
        PartialQueryLevelEnum::Album => {
            index.zip([PartialQueryLevelEnum::Album, PartialQueryLevelEnum::Track].iter())
        }
        PartialQueryLevelEnum::Track => index.zip([PartialQueryLevelEnum::Track].iter()),
    }
    .map(|(a, b)| (*a, b.clone()))
    .collect()
}

/// basic query function to model tracks with the apropiate values selected
fn basic_tree_query<'a>(
    pool: &DBPool,
    pql: &'a PartialQueryLevel,
) -> crate::schema::tracks::BoxedQuery<'a, diesel::sqlite::Sqlite> {
    use crate::schema::tracks::dsl::*;

    let mut query = tracks
        .filter(artist.is_not_null())
        .filter(artist.ne(""))
        .into_boxed();
    let vals = index_to_level_slice(pql);
    println!("running loop");
    for (index, level) in vals {
        let level_query = tracks
            .filter(artist.is_not_null())
            .filter(artist.ne(""))
            .into_boxed();
        let group = match level {
            PartialQueryLevelEnum::Artist => level_query.order_by(artist).select(artist).distinct(),
            PartialQueryLevelEnum::Album => level_query.order_by(album).select(album).distinct(),
            PartialQueryLevelEnum::Track => level_query.order_by(title).select(title).distinct(),
        };
        println!("locking db");
        let p = pool.lock().expect("Error in lock");
        println!("after locking");
        let val: String = group.offset(index as i64).first(p.deref()).unwrap();

        query = match level {
            PartialQueryLevelEnum::Artist => query.filter(artist.eq(val)),
            PartialQueryLevelEnum::Album => query.filter(album.eq(val)),
            PartialQueryLevelEnum::Track => query.filter(title.eq(val)),
        };
    }
    println!("basic query: {:?}", diesel::debug_query(&query));
    query
}

pub fn load_query(pool: &DBPool, pql: &PartialQueryLevel) -> loaded_playlist::LoadedPlaylist {
    let query = basic_tree_query(pool, pql);
    let p = pool.lock().expect("Error in lock");
    let items = query.load(p.deref()).expect("Error in loading");
    let name = "Test".to_string();

    loaded_playlist::LoadedPlaylist {
        id: -1,
        name,
        items,
        current_position: 0,
    }
}

/// Queries the tree but only returns not filled in results, i.e., children might be unpopulated
pub fn query_partial_tree(pool: &DBPool, pql: &PartialQueryLevel) -> Vec<String> {
    use crate::schema::tracks::dsl::*;
    let query = basic_tree_query(pool, pql);
    let levels = index_to_level_slice(pql);
    let level = levels.last().map(|(_, lvl)| lvl).unwrap_or(&pql.start);
    let p = pool.lock().expect("Error in lock");
    match level {
        PartialQueryLevelEnum::Artist => {
            let res = query
                .select(artist)
                .distinct()
                .order_by(artist)
                .load(p.deref())
                .expect("Error in loading");
            res.into_iter().collect::<Vec<String>>()
        }
        PartialQueryLevelEnum::Album => {
            let res = query
                .select((album, year))
                .distinct()
                .order_by(year)
                .distinct()
                .load(p.deref())
                .expect("Error in loading album");
            res.into_iter()
                .map(|s: (String, Option<i32>)| s.0)
                .collect::<Vec<String>>()
        }
        PartialQueryLevelEnum::Track => {
            let res: Vec<(Option<i32>, String)> = query
                .select((tracknumber, title))
                .distinct()
                .order_by(tracknumber)
                .distinct()
                .load(p.deref())
                .expect("Error in loading album");

            res.into_iter()
                .map(|(number, t)| t)
                .collect::<Vec<String>>()
        }
    }
}
