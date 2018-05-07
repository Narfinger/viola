use gdk;
use gtk;
use gtk::prelude::*;
use std::string::String;
use std::ops::Deref;
use loaded_playlist::LoadedPlaylist;

use gui::GuiPtrExt;
use types::*;

pub struct LibraryView {
    
}


pub fn new(pool: DBPool, builder: &BuilderPtr, gui: GuiPtr) {
    use diesel::{GroupByDsl, QueryDsl, RunQueryDsl};
    use schema::tracks::dsl::*;

    let libview: gtk::TreeView = builder.read().unwrap().get_object("libraryview").unwrap();

    //the model contains first a abbreviated string and in second column the whole string to construct the playlist
    let model = gtk::TreeStore::new(&[String::static_type(), String::static_type()]);
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
    libview.append_column(&column);

    println!("Running");
    for i in res {
        let st: String = i.chars().take(20).collect::<String>() + "..";
        model.insert_with_values(None, None, &[0, 1], &[&st, &i]);
    }

    libview.set_model(Some(&model));
    println!("Stopped");

    libview.connect_event_after(move |s,e| { signalhandler(&pool.clone(), &gui.clone(), s, e) });
}

fn signalhandler(pool: &DBPool, gui: &GuiPtr, tv: &gtk::TreeView, event: &gdk::Event) {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, debug_query};
    use schema::tracks::dsl::*;
    use diesel;

    if event.get_event_type() == gdk::EventType::DoubleButtonPress {
        if let Ok(b) = event.clone().downcast::<gdk::EventButton>() {
            if b.get_button() == 1 {
                let (model, iter) = tv.get_selection().get_selected().unwrap();
                let artist_name = model.get_value(&iter, 1).get::<String>().unwrap();
                let db = pool.get().expect("DB problem");
                let results = tracks
                    .filter(artist.eq(&artist_name))
                    .order(path)
                    .load(db.deref())
                    .expect("Problem loading playlist");
/* 
                {
                    let dquery = tracks.filter(artist.eq(&artist_name));
                    let debug = debug_query::<diesel::sqlite::Sqlite, _>(&dquery);
                    println!("loading playlist from query: {:?}", debug);
                } */

                let pl = LoadedPlaylist {
                    id: None,
                    name: artist_name,
                    items: results,
                    current_position: 0,
                };
                gui.add_page(pl);
            }
        }
    }
}
