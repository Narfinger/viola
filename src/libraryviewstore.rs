use gdk;
use gtk;
use gtk::prelude::*;
use std::string::String;
use std::rc::Rc;
use std::cell::{Cell, RefCell, RefMut};
use std::ops::Deref;
use loaded_playlist::LoadedPlaylist;

use maingui::MainGuiPtrExt;
use types::*;

pub struct LibraryView {
    
}

const ARTIST_TYPE: i32 = 1;
const ALBUM_TYPE: i32 = 2;
const TRACK_TYPE: i32 = 3;

enum LibraryLoadType {
    Artist,
    Album,
    Track,
    Invalid,
}

impl From<i32> for LibraryLoadType {
    fn from(i: i32) -> Self {
        match i {
            1 => LibraryLoadType::Artist,
            2 => LibraryLoadType::Album,
            3 => LibraryLoadType::Track,
            _ => LibraryLoadType::Invalid,
        }
    }
}

fn idle_fill<'a, I>(pool: DBPool, ats: &Rc<RefCell<I>>, model: &gtk::TreeStore, libview: &gtk::TreeView, gui: MainGuiPtr) -> gtk::Continue 
    where I: Iterator<Item = String> {
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
    use schema::tracks::dsl::*;

    //panic!("does not work yet, iterator gets not changes");
    if let Some(a) = ats.borrow_mut().next() {
        let db = pool.get().unwrap();
        let st: String = a.chars().take(20).collect::<String>() + "..";
        {
            let albums: Vec<String> = tracks
                .select(album)
                .order(year)
                .filter(artist.like(String::from("%") + &a + "%"))
                .group_by(album)
                .load(db.deref())
                .expect("Error in db connection");
            let artist_node = model.insert_with_values(None, None, &[0, 1, 2], &[&st, &a, &ARTIST_TYPE]);
            for ab in albums {
                let album_node = model.insert_with_values(Some(&artist_node), None, &[0, 1, 2], &[&ab, &ab, &ALBUM_TYPE]);
                {
                    let ts: Vec<String> = tracks
                    .select(title)
                    .order(tracknumber)
                    .filter(artist.like(String::from("%") + &a + "%"))
                    .filter(album.eq(ab))
                    .load(db.deref())
                    .expect("Error in db connection");
                    for t in ts {
                        model.insert_with_values(Some(&album_node), None, &[0, 1, 2], &[&t, &t, &TRACK_TYPE]);
                    }
                }
            }
        }
        gtk::Continue(true)
    } else {
        println!("Done");
        libview.connect_event_after(move |s,e| { signalhandler(&pool.clone(), &gui.clone(), s, e) });
        gtk::Continue(false)
    }
}

pub fn new(pool: DBPool, builder: &BuilderPtr, gui: MainGuiPtr) {
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
    use schema::tracks::dsl::*;
    
    let libview: gtk::TreeView = builder.read().unwrap().get_object("libraryview").unwrap();

    //the model contains first a abbreviated string and in second column the whole string to construct the playlist
    let model = gtk::TreeStore::new(&[String::static_type(), String::static_type(), i32::static_type()]);
 
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    libview.append_column(&column);

    libview.set_model(Some(&model));

    let db = pool.get().unwrap();
    let artists: Vec<String> = tracks
        .select(artist)
        .order(artist)
        .group_by(artist)
        .filter(artist.not_like(String::from("%") + "feat" + "%"))
        .load(db.deref())
        .expect("Error in db connection");

    let refcell = Rc::new(RefCell::new(artists.into_iter()));
    gtk::idle_add(move || {
        idle_fill(pool.clone(), &refcell, &model, &libview, gui.clone()) 
    });
}

fn signalhandler(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView, event: &gdk::Event) {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use schema::tracks::dsl::*;

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
