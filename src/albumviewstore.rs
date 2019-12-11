use crate::db::Track;
use crate::loaded_playlist::LoadedPlaylist;
use gdk;
use gtk;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::string::String;

use crate::maingui::{MainGuiExt, MainGuiPtrExt};
use crate::types::*;

pub fn new(db: &DBPool, builder: &BuilderPtr, gui: &MainGuiPtr) {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    let albumview: gtk::TreeView = builder.read().unwrap().get_object("albumview").unwrap();

    //the model contains first a abbreviated string and in second column the whole string to construct the playlist
    let model = gtk::ListStore::new(&[String::static_type(), bool::static_type()]);
    let fmodel = gtk::TreeModelFilter::new(&model, None);
    fmodel.set_visible_column(1);

    let searchfield: gtk::SearchEntry = builder.read().unwrap().get_object("albumsearch").unwrap();
    {
        let bc = builder.clone();
        searchfield.connect_search_changed(move |s| search_changed(s, &bc));
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
        .filter(album.ne(""))
        .distinct()
        .load(db.lock().expect("DB Error").deref())
        .expect("Error in db connection");

    fmodel.refilter();
    {
        let refcell = Rc::new(RefCell::new(albums.into_iter()));
        gtk::idle_add(move || idle_fill(&refcell, &model));
    }

    {
        let dbc = db.clone();
        let guic = gui.clone();
        albumview.connect_event_after(move |s, e| signalhandler(&dbc, &guic, s, e));
    }
}

fn idle_fill<I>(ats: &Rc<RefCell<I>>, model: &gtk::ListStore) -> gtk::Continue
where
    I: Iterator<Item = String>,
{
    if let Some(a) = ats.borrow_mut().next() {
        model.insert_with_values(None, &[0, 1], &[&a, &true]);
        Continue(true)
    } else {
        Continue(false)
    }
}

/// This function will be called on idle if the search changed.
/// Because there could be multiple functions added, we need to have the following safeguard
/// s is the string the search started with
/// field is the current field, if both are not the same, we abort this thread as somebody else should be running
fn idle_search_changed(
    s: Rc<String>,
    field: Rc<gtk::SearchEntry>,
    treeiter: Rc<gtk::TreeIter>,
    fmodel: Rc<gtk::TreeModelFilter>,
    model: Rc<gtk::ListStore>,
) -> gtk::Continue {
    let visible: &gtk::Value = &true.to_value();
    let invisible: &gtk::Value = &false.to_value();

    // abort if another thread is running
    if *s != field.get_text().unwrap().to_lowercase() {
        return gtk::Continue(false);
    }

    if !s.is_empty() {
        let val = model
            .get_value(&treeiter, 0)
            .get::<String>()
            .map(|v| v.to_lowercase().contains(&*s));

        if val == Some(true) {
            model.set_value(&treeiter, 1, visible);
        } else {
            model.set_value(&treeiter, 1, invisible);
        }
    } else {
        model.set_value(&treeiter, 1, visible);
    }

    // model.iter_next can return false, if that we do not spawn a new thread
    let val = model.iter_next(&treeiter);

    if val {
        gtk::idle_add(move || {
            idle_search_changed(
                s.clone(),
                field.clone(),
                treeiter.clone(),
                fmodel.clone(),
                model.clone(),
            )
        });
    } else {
        //fmodel.refilter();
    }
    gtk::Continue(false)
}

fn get_model_and_iter_for_selection(tv: &gtk::TreeView) -> (gtk::ListStore, gtk::TreeIter) {
    let (model, iter) = tv.get_selection().get_selected().unwrap();
    let filtermodel = model
        .downcast::<gtk::TreeModelFilter>()
        .expect("Error in TreeModelFilter downcast");
    let m = filtermodel
        .get_model()
        .expect("No base model for TreeModelFilter")
        .downcast::<gtk::ListStore>()
        .expect("Error in TreeStore downcast");
    let realiter = filtermodel.convert_iter_to_child_iter(&iter);

    (m, realiter)
}

fn search_changed(s: &gtk::SearchEntry, builder: &BuilderPtr) {
    let libview: gtk::TreeView = builder.read().unwrap().get_object("albumview").unwrap();

    let fmodel = Rc::new(
        libview
            .get_model()
            .unwrap()
            .downcast::<gtk::TreeModelFilter>()
            .unwrap(),
    );
    let model = Rc::new(
        fmodel
            .get_model()
            .unwrap()
            .downcast::<gtk::ListStore>()
            .unwrap(),
    );

    let text = Rc::new(s.get_text().unwrap().to_lowercase());
    let sc = Rc::new(s.clone());

    gtk::idle_add(move || {
        idle_search_changed(
            text.clone(),
            sc.clone(),
            Rc::new(model.get_iter_first().unwrap()),
            fmodel.clone(),
            model.clone(),
        )
    });
}

fn get_tracks_for_selection(
    db: &DBPool,
    tv: &gtk::TreeView,
) -> Result<(String, Vec<Track>), String> {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    let (m, iter) = get_model_and_iter_for_selection(tv);
    let album_name = m.get_value(&iter, 0).get::<String>().unwrap();
    let query = tracks.order(path);

    Ok((
        album_name.to_owned(),
        query
            .filter(album.eq(&album_name))
            .load(db.lock().expect("Lock Error").deref())
            .expect("Error in query"),
    ))
}

fn do_new(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView) {
    /*
        let (name, res) = get_tracks_for_selection(pool, tv).expect("Error in getting tracks");
        let pl = LoadedPlaylist {
            id: Cell::new(None),
            name,
            items: res,
            current_position: 0,
        };
        gui.add_page(pl);
    */
}

fn do_append(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView) {
    let (_, res) = get_tracks_for_selection(pool, tv).expect("Error in getting tracks");
    gui.append_to_playlist(res);
}

fn do_replace(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView) {
    let (_, res) = get_tracks_for_selection(pool, tv).expect("Error in getting tracks");
    gui.replace_playlist(res);
}

fn signalhandler(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView, event: &gdk::Event) {
    if event.get_event_type() == gdk::EventType::ButtonPress {
        info!("button press");
        if let Ok(b) = event.clone().downcast::<gdk::EventButton>() {
            info!("the button: {}", b.get_button());
            if b.get_button() == 3 {
                let menu = gtk::Menu::new();
                {
                    let menuitem = gtk::MenuItem::new_with_label("New");
                    let pc = pool.clone();
                    let gc = gui.clone();
                    let tvc = tv.clone();
                    menuitem.connect_activate(move |_| do_new(&pc, &gc, &tvc));
                    menu.append(&menuitem);
                }
                {
                    let menuitem = gtk::MenuItem::new_with_label("Replace");
                    let pc = pool.clone();
                    let gc = gui.clone();
                    let tvc = tv.clone();
                    menuitem.connect_activate(move |_| do_replace(&pc, &gc, &tvc));
                    menu.append(&menuitem);
                }
                {
                    let menuitem = gtk::MenuItem::new_with_label("Append");
                    let pc = pool.clone();
                    let gc = gui.clone();
                    let tvc = tv.clone();
                    menuitem.connect_activate(move |_| do_append(&pc, &gc, &tvc));
                    menu.append(&menuitem);
                }
                menu.show_all();
                gtk::GtkMenuExt::popup_at_pointer(&menu, Some(event));
            }
        }
    }
}
