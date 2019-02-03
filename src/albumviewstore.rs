use crate::db::Track;
use gdk;
use gtk;
use gtk::prelude::*;
use crate::loaded_playlist::LoadedPlaylist;
use serde_json;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::string::String;
use std::ops::Deref;

use crate::maingui::{MainGuiExt, MainGuiPtrExt};
use crate::types::*;

pub fn new(db: &DBPool, builder: &BuilderPtr, gui: &MainGuiPtr) {
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
    use crate::schema::tracks::dsl::*;

    let albumview: gtk::TreeView = builder.read().unwrap().get_object("albumview").unwrap();

    //the model contains first a abbreviated string and in second column the whole string to construct the playlist
    let model = gtk::ListStore::new(&[String::static_type(), String::static_type(), bool::static_type()]);
    let fmodel = gtk::TreeModelFilter::new(&model, None);
    fmodel.set_visible_column(2);

    //let searchfield: gtk::SearchEntry = builder
    //    .read()
    //    .unwrap()
    //    .get_object("albumsearch")
    //    .unwrap();
    {
        let bc = builder.clone();
        //searchfield.connect_search_changed(move |s| search_changed(s, &bc));
    }
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    albumview.append_column(&column);

    albumview.set_model(Some(&fmodel));

    let albums: Vec<String> = tracks
        .select(album)
        .order(album)
        .filter(artist.ne(""))
        .distinct()
        .load(db.deref())
        .expect("Error in db connection");


    fmodel.refilter();
    {
        let refcell = Rc::new(RefCell::new(albums.into_iter()));
        gtk::idle_add(move || idle_fill(&refcell, &model));
    }

    {
        let dbc = db.clone();
        let guic = gui.clone();
        //albumview.connect_event_after(move |s, e| signalhandler(&dbc, &guic, s, e));
    }
}

fn idle_fill<I>(ats: &Rc<RefCell<I>>, model: &gtk::ListStore) -> gtk::Continue 
    where  I: Iterator<Item = String> {

    if let Some(a) = ats.borrow_mut().next() {
        let st: String = a.chars().take(20).collect::<String>() + "..";
        model.insert_with_values(
            None,
            &[0,1,2],
            &[&st, &a, &true]
        );
        Continue(true)
    } else {
        Continue(false)
    }
}
