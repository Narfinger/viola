use crate::types::*;
use crate::{
    diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl},
    loaded_playlist::LoadedPlaylist,
};
use diesel::{query_builder::AsQuery, TextExpressionMethods};
use itertools::{izip, Itertools};
use std::ops::Deref;
use std::{any::Any, convert::TryInto};
use viola_common::{schema::tracks, TreeViewQuery};
use viola_common::{schema::tracks::dsl::*, TreeType};

/// produces a simple query that gives for one type a query that selects on it

/*
fn match_and_select_simple<'a>(
    base_query: viola_common::schema::tracks::BoxedQuery<'a, diesel::sqlite::Sqlite>,
    ttype: &'a viola_common::TreeType,
) -> diesel::query_builder::BoxedSelectStatement<
    'a,
    diesel::sql_types::Text,
    viola_common::schema::tracks::table,
    diesel::sqlite::Sqlite,
> {
    match ttype {
        viola_common::TreeType::Artist => base_query
            .select(artist)
            .filter(artist.not_like("%feat%"))
            .group_by(artist)
            .distinct()
            .order_by(artist),
        viola_common::TreeType::Album => base_query
            .select(album)
            .group_by(album)
            .distinct()
            .order_by(album),
        viola_common::TreeType::Track => base_query
            .select(title)
            .group_by(title)
            .distinct()
            .order_by(title),
        viola_common::TreeType::Genre => base_query
            .select(genre)
            .group_by(genre)
            .distinct()
            .order_by(genre),
    }
}

/// Produces the string that we should filter on if we are deeper in the tree
fn get_filter_string(
    base_query: viola_common::schema::tracks::BoxedQuery<diesel::sqlite::Sqlite>,
    db: &DBPool,
    ttype: viola_common::TreeType,
    index: &usize,
    search: &Option<String>,
) -> String {
    let select_query = match_and_select_simple(base_query, &ttype);
    let select_query = if let Some(ref search_string) = search {
        select_query
            .filter(artist.like(String::from("%") + &search_string + "%"))
            .or_filter(album.like(String::from("%") + &search_string + "%"))
            .or_filter(title.like(String::from("%") + &search_string + "%"))
    } else {
        select_query
    };
    let loaded_query: Vec<String> = select_query
        .offset(index.clone().try_into().unwrap())
        .limit(1)
        .load(db.lock().unwrap().deref())
        .expect("Error in query");
    loaded_query.first().expect("Error in stuff").to_string()
}

/// Generral Query to get the tree
fn treeview_query<'a>(
    db: &'a DBPool,
    query: &'a TreeViewQuery,
) -> viola_common::schema::tracks::BoxedQuery<'a, diesel::sqlite::Sqlite> {
    let mut filter_strings = Vec::new();
    // for first one
    if let Some(i) = query.indices.get(0) {
        let base_query: viola_common::schema::tracks::BoxedQuery<diesel::sqlite::Sqlite> =
            tracks.into_boxed();
        filter_strings.push(get_filter_string(
            base_query,
            db,
            query.types[0],
            i,
            &query.search,
        ));
    }
    println!("filter strings: {:?}", filter_strings);
    println!("search {:?}", query.search);
    // for second one
    if let Some(i) = query.indices.get(1) {
        let base_query = match query.types[0] {
            viola_common::TreeType::Artist => tracks
                .filter(artist.like(filter_strings[0].to_owned() + "%"))
                .into_boxed(),
            viola_common::TreeType::Album => tracks
                .filter(album.eq(filter_strings[0].clone()))
                .into_boxed(),
            viola_common::TreeType::Track => tracks
                .filter(title.eq(filter_strings[0].clone()))
                .into_boxed(),
            viola_common::TreeType::Genre => tracks
                .filter(genre.eq(filter_strings[0].clone()))
                .into_boxed(),
        };
        filter_strings.push(get_filter_string(
            base_query,
            db,
            query.types[1],
            i,
            &query.search,
        ));
    }

    // for third one
    if let Some(i) = query.indices.get(2) {
        panic!("Not yet implemented");
    }

    let mut db_query = tracks.into_boxed::<diesel::sqlite::Sqlite>();
    for (layer, filter_string) in filter_strings.iter().enumerate() {
        db_query = match query.types[layer] {
            viola_common::TreeType::Artist => db_query.filter(artist.eq(filter_string.clone())),
            viola_common::TreeType::Album => db_query.filter(album.eq(filter_string.clone())),
            viola_common::TreeType::Track => db_query.filter(title.eq(filter_string.clone())),
            viola_common::TreeType::Genre => db_query.filter(genre.eq(filter_string.clone())),
        };
    }
    db_query
}

/// Produces a partial query, i.e., the Vector of Strings that we show in the treeview
pub(crate) fn partial_query(db: &DBPool, query: &TreeViewQuery) -> Vec<String> {
    let base_query = treeview_query(db, query);
    info!("Query: {:?}", query);
    let query_type = match query.indices.len() {
        0 => query.types.get(0),
        1 => query.types.get(1),
        2 => query.types.get(2),
        _ => query.types.last(),
    }
    .expect("Error in index stuff");

    //let mut final_query = match_and_select_simple(base_query, query_type);
    let mut final_query = base_query;

    if let Some(ref search_string) = query.search {
        final_query = final_query
            .filter(artist.like(String::from("%") + &search_string + "%"))
            .or_filter(album.like(String::from("%") + &search_string + "%"))
            .or_filter(title.like(String::from("%") + &search_string + "%"));
    }

    if query.indices.len() == 1
        && query.types.get(0) == Some(&TreeType::Artist)
        && query.types.get(1) == Some(&TreeType::Album)
    {
        let result = final_query
            .select((album, year))
            .group_by((album, year))
            .order_by(year)
            .load::<(String, Option<i32>)>(db.lock().unwrap().deref())
            .expect("Error in query");
        result
            .iter()
            .map(|(album_t, year_t)| {
                year_t.map_or(String::from(""), |t| t.to_string()) + "-" + album_t
            })
            .collect()
    } else if query.indices.len() == 2
        && query.types.get(0) == Some(&TreeType::Artist)
        && query.types.get(1) == Some(&TreeType::Album)
        && query.types.get(2) == Some(&TreeType::Track)
    {
        let result = final_query
            .select((title, tracknumber))
            .group_by((title, tracknumber))
            .order_by(tracknumber)
            .load::<(String, Option<i32>)>(db.lock().unwrap().deref())
            .expect("Error in query");
        result
            .iter()
            .map(|(album_t, tracknumber_t)| {
                tracknumber_t.map_or(String::from(""), |t| t.to_string()) + "-" + album_t
            })
            .collect()
    } else {
        match_and_select_simple(final_query, query_type)
            .load(db.lock().unwrap().deref())
            .expect("Error on query")
    }
}
*/

/// produces the filter string, for sorting reasons we need the type_vec to be the first n of the types in the original query
/// where n is the current iteration depth
fn get_filter_string(
    new_bunch: &Vec<viola_common::Track>,
    current_ttype: &TreeType,
    index: &usize,
    type_vec: Vec<TreeType>,
) -> String {
    let mut new: Vec<viola_common::Track> = new_bunch.iter().map(|t| (*t).clone()).collect();
    let new_indices = (0..type_vec.len()).collect();
    //println!("we sorted to {:?}", &type_vec);
    let query = TreeViewQuery {
        types: type_vec,
        indices: new_indices,
        search: None,
    };
    sort_tracks(&query, &mut new);
    //println!("sorted {:?}", new.iter().map(|t| t.album.clone()));

    panic!(
        "there is something wrong with this because the empty string is vanishing somewhere here"
    );
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
    println!("full unique {:?}", &full_unique);
    let st = full_unique.get(*index).unwrap().clone().clone();
    st
}

fn basic_get_tracks(db: &DBPool, query: &TreeViewQuery) -> Vec<viola_common::Track> {
    //this function is currently to difficult to implement in diesel as we cannot clone boxed ty pes and otherwise we can cyclic type error
    let mut current_tracks = if let Some(ref search_string) = query.search {
        tracks
            .filter(artist.like(String::from("%") + &search_string + "%"))
            .or_filter(album.like(String::from("%") + &search_string + "%"))
            .or_filter(title.like(String::from("%") + &search_string + "%"))
            .load::<viola_common::Track>(db.lock().unwrap().deref())
            .unwrap()
    } else {
        tracks
            .filter(artist.ne(""))
            .load::<viola_common::Track>(db.lock().unwrap().deref())
            .unwrap()
    };

    for (recursion_depth, (index, current_ttype)) in
        izip!(query.indices.iter(), query.types.iter(),).enumerate()
    {
        let filter_query_types: Vec<TreeType> = query
            .types
            .iter()
            .take(recursion_depth + 1)
            .cloned()
            .collect();
        //println!(
        //    "recdepth {}, filter_query_types {:?}",
        //    recursion_depth, &filter_query_types
        //);

        let filter_value =
            get_filter_string(&current_tracks, &current_ttype, index, filter_query_types);
        //println!("current iter {:?}", &current_tracks);
        //println!(
        //    "filter value {}, filter_by {:?}",
        //    filter_value, current_ttype
        //);

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
    sort_tracks(query, &mut current_tracks);

    //println!("final tracks {:?}", current_tracks);
    current_tracks
}

/// sorts the tracks according to the treeviewquery we have
fn sort_tracks(query: &TreeViewQuery, t: &mut [viola_common::Track]) {
    if query.indices.len() == 0 {
        match query.types.get(0) {
            Some(&TreeType::Artist) => t.sort_by_cached_key(|t| t.artist.to_owned()),
            Some(&TreeType::Album) => t.sort_by_cached_key(|t| t.album.to_owned()),
            Some(&TreeType::Genre) => t.sort_by_cached_key(|t| t.genre.to_owned()),
            Some(&TreeType::Track) => t.sort_by_cached_key(|t| t.title.to_owned()),
            None => t.sort_by_cached_key(|t| t.artist.to_owned()),
        }
    } else if query.indices.len() == 1
        && query.types.get(0) == Some(&TreeType::Artist)
        && query.types.get(1) == Some(&TreeType::Album)
    {
        t.sort_unstable_by_key(|t| t.year);
    } else if query.indices.len() == 2
        && query.types.get(0) == Some(&viola_common::TreeType::Artist)
        && query.types.get(1) == Some(&viola_common::TreeType::Album)
        && query.types.get(2) == Some(&viola_common::TreeType::Track)
    {
        t.sort_unstable_by_key(|t| t.tracknumber);
    }
}

/// custom strings that appear in the partial query view
fn track_to_partial_string(query: &TreeViewQuery, t: viola_common::Track) -> String {
    if query.indices.len() == 0 {
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
        let last = query
            .indices
            .iter()
            .zip(query.types.iter())
            .last()
            .unwrap()
            .1;
        match last {
            &TreeType::Artist => t.artist,
            &TreeType::Album => t.album,
            &TreeType::Track => t.title,
            &TreeType::Genre => t.genre,
        }
    }
}

pub(crate) fn partial_query(db: &DBPool, query: &TreeViewQuery) -> Vec<String> {
    let t = basic_get_tracks(db, query);
    println!("query: {:?}", &query);
    let strings = t
        .into_iter()
        .map(|t| track_to_partial_string(query, t))
        .unique()
        .collect();
    strings
}

/// produces a LoadedPlaylist frrom a treeviewquery
pub(crate) fn load_query(db: &DBPool, query: &TreeViewQuery) -> LoadedPlaylist {
    let t = basic_get_tracks(db, query);
    LoadedPlaylist {
        id: -1,
        name: "Foo".to_string(),
        current_position: 0,
        items: t,
    }

    /*
    let mut q = treeview_query(db, query);

    info!("query types: {:?}", query.types);
    //custom sorting
    if query.types.get(0) == Some(&viola_common::TreeType::Artist)
        && query.types.get(1) == Some(&viola_common::TreeType::Album)
        && query.types.get(2) == Some(&viola_common::TreeType::Track)
    {
        q = q.order((year, tracknumber));
    }

    let name = if query.search.is_none() || query.search.as_ref().unwrap().is_empty() {
        "Foo".to_string()
    } else {
        query.search.to_owned().unwrap()
    };
    let t = q.load(db.lock().unwrap().deref()).expect("Error in Query");
    LoadedPlaylist {
        id: -1,
        name,
        current_position: 0,
        items: t,
    }
    */
}
