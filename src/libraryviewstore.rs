use gdk;
use gtk;
use gtk::prelude::*;
use std::string::String;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::time::{Duration, Instant};
use loaded_playlist::LoadedPlaylist;
use db::Track;

use maingui::{MainGuiExt, MainGuiPtrExt};
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

fn idle_fill<I>(pool: &DBPool, ats: &Rc<RefCell<I>>, model: &gtk::TreeStore) -> gtk::Continue 
    where I: Iterator<Item = String> {
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
    use schema::tracks::dsl::*;

    ///TODO replace this with const fn
    let DEFAULT_VISIBILITY: &gtk::Value = &true.to_value();

    //panic!("does not work yet, iterator gets not changes");
    if let Some(a) = ats.borrow_mut().next() {
        let db = pool.get().unwrap();
        let st: String = a.chars().take(20).collect::<String>() + "..";
        {
            let albums: Vec<(String, Option<i32>)> = tracks
                .select((album,year))
                .order(year)
                .filter(artist.like(String::from("%") + &a + "%"))
                .group_by(album)
                .load(db.deref())
                .expect("Error in db connection");
            let artist_node = model.insert_with_values(None, None, &[0, 1, 2, 3], &[&st, &a, &ARTIST_TYPE, DEFAULT_VISIBILITY]);

            for (ab, y) in albums {
                //add the year if it exists to the string we insert
                let abstring = if let Some(yp) = y {
                    yp.to_string() + " - " + &ab
                } else {
                    ab.to_string()
                };

                let album_node = model.insert_with_values(Some(&artist_node), None, &[0, 1, 2, 3], &[&abstring, &ab, &ALBUM_TYPE, DEFAULT_VISIBILITY]);
                {
                    let ts: Vec<String> = tracks
                    .select(title)
                    .order(tracknumber)
                    .filter(artist.like(String::from("%") + &a + "%"))
                    .filter(album.eq(ab))
                    .load(db.deref())
                    .expect("Error in db connection");
                    for t in ts {
                        model.insert_with_values(Some(&album_node), None, &[0, 1, 2, 3], &[&t, &t, &TRACK_TYPE, DEFAULT_VISIBILITY]);
                    }
                }
            }
        }
        gtk::Continue(true)
    } else {
        println!("Done");
        gtk::Continue(false)
    }
}

pub fn new(pool: DBPool, builder: &BuilderPtr, gui: MainGuiPtr) {
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
    use schema::tracks::dsl::*;
    
    let libview: gtk::TreeView = builder.read().unwrap().get_object("libraryview").unwrap();

    //the model contains first a abbreviated string and in second column the whole string to construct the playlist
    let model = gtk::TreeStore::new(&[String::static_type(), String::static_type(), i32::static_type(), bool::static_type()]);
    let fmodel = gtk::TreeModelFilter::new(&model, None);
    fmodel.set_visible_column(3);

    let searchfield: gtk::SearchEntry = builder.read().unwrap().get_object("collectionsearch").unwrap();
    {
        let bc = builder.clone();
        searchfield.connect_search_changed(move |s| search_changed(s, &bc));
    }
    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    libview.append_column(&column);

    libview.set_model(Some(&fmodel));

    let db = pool.get().unwrap();
    let artists: Vec<String> = tracks
        .select(artist)
        .order(artist)
        .group_by(artist)
        .filter(artist.not_like(String::from("%") + "feat" + "%"))
        .filter(artist.ne(""))
        .load(db.deref())
        .expect("Error in db connection");
        


    {
        let pc = pool.clone();
        let guic = gui.clone();
        libview.connect_event_after(move |s,e| { signalhandler(&pc, &guic, s, e) });
    }
    let refcell = Rc::new(RefCell::new(artists.into_iter()));
    {
        let pc = pool.clone();
        let guic = gui.clone();
        gtk::idle_add(move || {
            idle_fill(&pc, &refcell, &model) 
        });
    }
}


/// This function will be called on idle if the search changed.
/// Because there could be multiple functions added, we need to have the following safeguard
/// s is the string the search started with
/// field is the current field, if both are not the same, we abort this thread as somebody else should be running
fn idle_search_changed(s: Rc<String>, field: Rc<gtk::SearchEntry>, treeiter: Rc<gtk::TreeIter>, fmodel: Rc<gtk::TreeModelFilter>, model: Rc<gtk::TreeStore>) -> gtk::Continue {
    let visible: &gtk::Value = &true.to_value();
    let invisible: &gtk::Value = &false.to_value();

    // abort if another thread is running
    if *s != field.get_text().unwrap().to_lowercase() {
        println!("Killing this search thread because {:?}, {:?}", *s, field.get_text().unwrap());
        return gtk::Continue(false);
    }
        
    if !s.is_empty() {
        ///we need to go into children
        //panic!("we need to go into children");
        let val = model.get_value(&treeiter, 1).get::<String>().map(|v| v.to_lowercase().contains(&*s));
        println!("Looking at: {:?}, {:?}, {:?}", model.get_value(&treeiter, 1).get::<String>(), val, s);
        if val == Some(true) {
            model.set_value(&treeiter, 3, visible);
            
            // set parents to visible
            {
                let mut it = (*treeiter).clone();
                let mut valid = true;
                while valid {
                    let val = model.iter_parent(&it);
                    model.set_value(&it, 3, visible);

                    let v = model.get_value(&it, 1).get::<String>().unwrap();
                    println!("Doing parents {}", v);

                    if let Some(v) = val {
                        valid = true;
                        it = v;
                    } else {
                        valid = false;
                    }
                }
            }
        } else {
            model.set_value(&treeiter, 3, invisible);
        }
    } else {
        model.set_value(&treeiter, 3, visible);
    }

    // check the children if they exist
    {
        let it = (*treeiter).clone();
        if let Some(child_iter) = model.iter_children(Some(&it)) {
            let ci = Rc::new(child_iter);
            let sc = s.clone();
            let fc = field.clone();
            let fmc = fmodel.clone();
            let mc = model.clone();
            gtk::idle_add(move || {
                idle_search_changed(sc.clone(), fc.clone(), ci.clone(), fmc.clone(), mc.clone()) 
            });
        }
    }


    // model.iter_next can return false, if that we do not spawn a new thread
    let val = model.iter_next(&treeiter);
    
    if val {
        gtk::idle_add(move || {
            idle_search_changed(s.clone(), field.clone(), treeiter.clone(), fmodel.clone(), model.clone()) 
        });
    } else {
        //fmodel.refilter();
    }
    gtk::Continue(false)
} 

fn search_changed(s: &gtk::SearchEntry, builder: &BuilderPtr) {
    
    let libview: gtk::TreeView = builder.read().unwrap().get_object("libraryview").unwrap();

    
    let fmodel = Rc::new(libview.get_model().unwrap().downcast::<gtk::TreeModelFilter>().unwrap());
    let model = Rc::new(fmodel.get_model().unwrap().downcast::<gtk::TreeStore>().unwrap());
 
    let text = Rc::new(s.get_text().unwrap().to_lowercase());
    let sc = Rc::new(s.clone());

    gtk::idle_add(move || {
        idle_search_changed(text.clone(), sc.clone(), Rc::new(model.get_iter_first().unwrap()), fmodel.clone(), model.clone()) 
    });
    println!("added search_changed");
    
    /*
    
    if let Some(text) = s.get_text() {
        let mut treeiter = model.get_iter_first();
        let mut valid = treeiter.is_some();
        while valid {
            if let Some(ref t) = treeiter {
                ///we need to go into children
                panic!("we need to go into children");
                let val = model.get_value(&t, 0).get::<String>().map(|s| s.contains(&text));
                if val == Some(true) {
                    model.set_value(&t, 3, visible);
                } else {
                    model.set_value(&t, 3, invisible);
                }
                valid = model.iter_next(&t);
            } else {
                panic!("Invalid... why is it?");
                println!("this is invalid?");
            }
        }
    } else {
        let mut treeiter = model.get_iter_first();
        while let Some(ref t) = treeiter {
            model.set_value(&t, 3, visible);
            model.iter_next(&t);
        }
    }
    fmodel.refilter();
    */
}

fn get_tracks_for_selection(pool: &DBPool, tv: &gtk::TreeView) -> Option<(String, Vec<Track>)> {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods};
    use schema::tracks::dsl::*;
    
    let (model, iter) = tv.get_selection().get_selected().unwrap();
    let m = model.downcast::<gtk::TreeStore>().expect("Error in downcast");
    let artist_name = m.get_value(&iter, 1).get::<String>().unwrap();
    let db = pool.get().expect("DB problem");
    let query = tracks
        .filter(artist.like(String::from("%") + &artist_name + "%"))
        .order(path)
        .into_boxed();
    if m.iter_depth(&iter) == 0 {
        Some((artist_name, query.load(db.deref()).expect("Error in query")))
    } else if m.iter_depth(&iter) == 1 {
        let a = m.get_value(&iter, 1).get::<String>().unwrap();
        Some((a.clone(), query.filter(album.eq(a)).load(db.deref()).expect("Error in query")))
    } else if m.iter_depth(&iter) == 2 {
        let a = m.get_value(&iter, 1).get::<String>().unwrap();
        let t = m.get_value(&iter, 2).get::<String>().unwrap();
        Some((t.clone(), query.filter(album.eq(a)).filter(title.eq(t)).load(db.deref()).expect("Error in query")))
    } else {
        None
    }
}

fn do_new(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView) {
    let (name, res) = get_tracks_for_selection(pool, tv).expect("Error in getting tracks");
    let pl =LoadedPlaylist {
        id: None,
        name: name,
        items: res,
        current_position: 0,
    };
    gui.add_page(pl);
}

fn do_append(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView) {
    let (_, res) = get_tracks_for_selection(pool, tv).expect("Error in getting tracks");
    gui.append_to_playlist(res);
}

fn do_replace(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView) {
    panic!("not yet implemented");
}


fn signalhandler(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView, event: &gdk::Event) {
    if event.get_event_type() == gdk::EventType::ButtonPress {
        println!("button press");
        if let Ok(b) = event.clone().downcast::<gdk::EventButton>() {
            println!("the button: {}", b.get_button());
            if b.get_button() == 3 {
                let mut menu = gtk::Menu::new();
                {
                    let menuitem = gtk::MenuItem::new_with_label("New");
                    let pc = pool.clone();
                    let gc = gui.clone();
                    let tvc = tv.clone();
                    menuitem.connect_activate(move |_| do_new(&pc,&gc, &tvc));
                    menu.append(&menuitem);
                }
                {
                    let menuitem = gtk::MenuItem::new_with_label("Replace");
                    let pc = pool.clone();
                    let gc = gui.clone();
                    let tvc = tv.clone();
                    menuitem.connect_activate(move |_| do_replace(&pc,&gc, &tvc));
                    menu.append(&menuitem);
                }
                {
                    let menuitem = gtk::MenuItem::new_with_label("Append");
                    let pc = pool.clone();
                    let gc = gui.clone();
                    let tvc = tv.clone();
                    menuitem.connect_activate(move |_| do_append(&pc,&gc, &tvc));
                    menu.append(&menuitem);
                }
                menu.show_all();
                gtk::MenuExt::popup_at_pointer(&menu, event);
            }
        }
    }
}
