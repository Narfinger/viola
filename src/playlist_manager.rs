use gdk;
use gtk;
use gtk::prelude::*;
use std::string::String;
use std::ops::Deref;
use loaded_playlist::LoadedPlaylist;

use gui::GuiPtrExt;
use types::*;

pub struct PlaylistManager {
    
}

pub fn new(pool: DBPool, builder: &BuilderPtr, gui: GuiPtr) {
    let plview: gtk::TreeView = builder.read().unwrap().get_object("playlistmanagerview").unwrap();

    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    plview.append_column(&column);

    let model = gtk::TreeStore::new(&[String::static_type(), i32::static_type()]);
    model.insert_with_values(None, None, &[0,1], &[&"Full Collection", &0]);

    plview.set_model(Some(&model));
    plview.connect_event_after(move |s,e| { signalhandler(&pool.clone(), &gui.clone(), s, e) });
}

fn signalhandler(pool: &DBPool, gui: &GuiPtr, tv: &gtk::TreeView, event: &gdk::Event) {
    if event.get_event_type() == gdk::EventType::DoubleButtonPress {
        if let Ok(b) = event.clone().downcast::<gdk::EventButton>() {
            if b.get_button() == 1 {
                let (model, iter) = tv.get_selection().get_selected().unwrap();
                let index = model.get_value(&iter, 1).get::<i32>().unwrap();
               
                gui.add_page(add_playlist(pool, index));
            }
        }
    }
}

fn add_playlist(pool: &DBPool, index: i32) -> LoadedPlaylist {
    use diesel::{QueryDsl, RunQueryDsl};
    use schema::tracks::dsl::*;
    
    println!("You selected index: {}", index);
    if index == 0 {
        let db = pool.get().expect("DB problem");
        let results = tracks
            .order(path)
            .load(db.deref())
            .expect("Problem loading playlist");

        LoadedPlaylist {
            id: None,
            name: String::from("Full Collection"),
            items: results,
            current_position: 0,
        }
    } else {
        LoadedPlaylist {
            id: None,
            name: String::from("Error"),
            items: vec![],
            current_position: 0
        }
    }
}