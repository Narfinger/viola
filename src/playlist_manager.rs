use crate::loaded_playlist::LoadedPlaylist;
use crate::smartplaylist_parser;
use crate::smartplaylist_parser::{LoadSmartPlaylist, SmartPlaylist};
use gdk;
use gtk;
use gtk::prelude::*;
use std::cell::Cell;
use std::ops::Deref;
use std::string::String;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use crate::db::get_new_playlist_id;
use crate::maingui::MainGuiPtrExt;
use crate::types::*;

pub struct PlaylistManager {}

pub fn new(pool: DBPool, builder: &BuilderPtr, gui: MainGuiPtr) -> PlaylistManager {
    let plview: gtk::TreeView = builder
        .read()
        .unwrap()
        .get_object("playlistmanagerview")
        .unwrap();

    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    plview.append_column(&column);

    let model = gtk::TreeStore::new(&[String::static_type(), i32::static_type()]);
    model.insert_with_values(None, None, &[0, 1], &[&"Full Collection", &0]);

    let sm = smartplaylist_parser::construct_smartplaylists_from_config();
    for (i, v) in sm.iter().enumerate() {
        let index = ((i as usize) + 1) as i32;
        model.insert_with_values(None, None, &[0, 1], &[&v.name, &index]);
    }

    plview.set_model(Some(&model));
    plview.connect_event_after(move |s, e| signalhandler(&pool, &gui, &sm, s, e));

    PlaylistManager {}
}

fn signalhandler(
    pool: &DBPool,
    gui: &MainGuiPtr,
    sm: &[SmartPlaylist],
    tv: &gtk::TreeView,
    event: &gdk::Event,
) {
    if event.get_event_type() == gdk::EventType::DoubleButtonPress {
        if let Ok(b) = event.clone().downcast::<gdk::EventButton>() {
            if b.get_button() == 1 {
                let (model, iter) = tv.get_selection().get_selected().unwrap();
                let index = model.get_value(&iter, 1).get::<i32>().unwrap();
                gui.add_page(add_playlist(pool, sm, index));
            }
        }
    }
}

fn add_playlist(db: &DBPool, sm: &[SmartPlaylist], index: i32) -> LoadedPlaylist {
    use crate::schema::playlists::dsl::*;
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    info!("You selected index: {}", index);
    if index == 0 {
        let results = tracks
            .order(path)
            .load(db.lock().expect("DB Error").deref())
            .expect("Problem loading playlist");

        let new_id = get_new_playlist_id(db);

        LoadedPlaylist {
            id: new_id,
            name: String::from("Full Collection"),
            items: results,
            current_position: Arc::new(AtomicUsize::new(0)),
        }
    } else {
        let i = index - 1 as i32;
        sm[i as usize].load(db)
    }
}
