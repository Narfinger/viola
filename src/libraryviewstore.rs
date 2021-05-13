use crate::types::*;
use crate::{
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    loaded_playlist::LoadedPlaylist,
};
use diesel::TextExpressionMethods;
use itertools::{izip, Itertools};
use std::ops::Deref;
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
        tracks
            .filter(artist.like(String::from("%") + &search_string + "%"))
            .or_filter(album.like(String::from("%") + &search_string + "%"))
            .or_filter(title.like(String::from("%") + &search_string + "%"))
            .load::<viola_common::Track>(db.lock().deref())
            .unwrap()
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

    current_tracks
}

fn sort_key_from_treetype<'a>(
    ttype: &'a Option<&'a TreeType>,
    t: &'a viola_common::Track,
) -> &'a String {
    match ttype {
        Some(&TreeType::Artist) => &t.artist,
        Some(&TreeType::Album) => &t.album,
        Some(&TreeType::Genre) => &t.genre,
        Some(&TreeType::Track) => &t.title,
        None => &t.artist,
    }
}

/// sorts the tracks according to the treeviewquery we have
/// TODO: This has the problem that we rarely want to sort albums by name but mostly by year.
/// But sometimes by name
fn sort_tracks(query: &TreeViewQuery, t: &mut [viola_common::Track]) {
    let indexed = query.get_indexed_ttypes();
    t.sort_unstable_by(|x, y| {
        // We build a map of Ordering that compares all the keys in indexed.
        // Then we fold over this to use Ordering::Then to get the correct evaluation
        let ordering = std::cmp::Ordering::Equal;
        indexed
            .iter()
            .map(|ttype| {
                sort_key_from_treetype(&Some(&ttype), x)
                    .cmp(sort_key_from_treetype(&Some(&ttype), y))
            })
            .fold(ordering, |acc, x| acc.then(x))
    });

    /*
    if query.indices.is_empty() {
        let ttype = query.types.get(0);
        t.sort_by(|x, y| sort_key_from_treetype(&ttype, x).cmp(sort_key_from_treetype(&ttype, y)));
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
    } else {
        let last = query.get_after_last_ttype();
        t.sort_by(|x, y| sort_key_from_treetype(&last, x).cmp(sort_key_from_treetype(&last, y)));
    }*/
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
