use gtk;
use gtk::prelude::*;
use std::ops::Deref;
use types::*;

pub fn connect(pool: DBPool, tv: &gtk::TreeView) {
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl};
    use schema::tracks::dsl::*;

    let model = gtk::TreeStore::new(&[String::static_type()]);
    let db = pool.get().unwrap();
    let res: Vec<String> = tracks
        .select(artist)
        .order(artist)
        .group_by(artist)
        .load(db.deref())
        .expect("Error in db connection");

    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    tv.append_column(&column);

    println!("Running");
    for i in res {
        let st: String = i.chars().take(20).collect();
        model.insert_with_values(None, None, &[0], &[&st]);
    }

    tv.set_model(Some(&model));
    println!("Stopped");
}
