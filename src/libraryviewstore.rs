use crate::types::*;
use crate::{
    diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl},
    loaded_playlist::LoadedPlaylist,
};
use diesel::TextExpressionMethods;
use std::convert::TryInto;
use std::ops::Deref;
use viola_common::schema::tracks::dsl::*;
use viola_common::TreeViewQuery;

fn match_and_select<'a>(
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

fn get_filter_string(
    base_query: viola_common::schema::tracks::BoxedQuery<diesel::sqlite::Sqlite>,
    db: &DBPool,
    ttype: viola_common::TreeType,
    index: &usize,
    search: &Option<String>,
) -> String {
    let select_query = match_and_select(base_query, &ttype);
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
                .filter(artist.eq(filter_strings[0].clone()))
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

pub(crate) fn partial_query(db: &DBPool, query: &TreeViewQuery) -> Vec<String> {
    let base_query = treeview_query(db, query);
    println!("Query: {:?}", query);
    let query_type = match query.indices.len() {
        0 => query.types.get(0),
        1 => query.types.get(1),
        2 => query.types.get(2),
        _ => query.types.last(),
    }
    .expect("Error in index stuff");
    let mut final_query = match_and_select(base_query, query_type);

    if let Some(ref search_string) = query.search {
        final_query = final_query
            .filter(artist.like(String::from("%") + &search_string + "%"))
            .or_filter(album.like(String::from("%") + &search_string + "%"))
            .or_filter(title.like(String::from("%") + &search_string + "%"));
    }

    final_query
        .load(db.lock().unwrap().deref())
        .expect("Error in query")
}

pub(crate) fn load_query(db: &DBPool, query: &TreeViewQuery) -> LoadedPlaylist {
    let mut q = treeview_query(db, query);

    info!("query types: {:?}", query.types);
    //custom sorting
    if query.types.get(0) == Some(&viola_common::TreeType::Artist)
        && query.types.get(1) == Some(&viola_common::TreeType::Album)
        && query.types.get(2) == Some(&viola_common::TreeType::Track)
    {
        q = q.order((year, tracknumber));
    }

    let t = q.load(db.lock().unwrap().deref()).expect("Error in Query");
    LoadedPlaylist {
        id: -1,
        name: "Foobar".to_string(),
        current_position: 0,
        items: t,
    }
}
