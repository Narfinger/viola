use crate::diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl};
use crate::types::*;
use std::convert::TryInto;
use std::ops::Deref;
use viola_common::schema::tracks::dsl::*;
use viola_common::{Track, TreeViewQuery};

fn match_and_select<'a>(
    base_query: viola_common::schema::tracks::BoxedQuery<'a, diesel::sqlite::Sqlite>,
    ttype: &'a viola_common::TreeType,
) -> diesel::query_builder::BoxedSelectStatement<
    'a,
    diesel::sql_types::Text,
    viola_common::schema::tracks::table,
    diesel::sqlite::Sqlite,
> {
    let select_query = match ttype {
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
    };
    select_query
}

fn get_filter_string(
    base_query: viola_common::schema::tracks::BoxedQuery<diesel::sqlite::Sqlite>,
    db: &DBPool,
    ttype: viola_common::TreeType,
    index: &usize,
) -> String {
    let select_query = match_and_select(base_query, &ttype);
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
        filter_strings.push(get_filter_string(base_query, db, query.types[0], i));
    }
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
        };
        filter_strings.push(get_filter_string(base_query, db, query.types[1], i));
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
        };
    }
    db_query
}

pub(crate) fn partial_query(db: &DBPool, query: &TreeViewQuery) -> Vec<String> {
    let base_query = treeview_query(db, query);
    let query_type = match query.indices.len() {
        0 => query.types.get(0),
        1 => query.types.get(1),
        2 => query.types.get(2),
        _ => query.types.last(),
    }
    .expect("Error in index stuff");
    let final_query = match_and_select(base_query, query_type);
    final_query
        .load(db.lock().unwrap().deref())
        .expect("Error in query")
}
