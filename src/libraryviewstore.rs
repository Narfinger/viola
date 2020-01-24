/*

use crate::db::Track;
use crate::loaded_playlist::LoadedPlaylist;
use gdk;
use gtk;
use gtk::prelude::*;
use serde_json;
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::rc::Rc;
use std::string::String;

//use crate::maingui::{MainGuiExt, MainGuiPtrExt};
use crate::types::*;

pub struct LibraryView {}

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

fn idle_fill<I>(db: &DBPool, ats: &Rc<RefCell<I>>, model: &gtk::TreeStore) -> gtk::Continue
where
    I: Iterator<Item = String>,
{
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};

    ///TODO replace this with const fn
    let DEFAULT_VISIBILITY: &gtk::Value = &true.to_value();

    //panic!("does not work yet, iterator gets not changes");
    if let Some(a) = ats.borrow_mut().next() {
        let st: String = a.chars().take(20).collect::<String>() + "..";
        {
            let albums: Vec<(String, Option<i32>)> = tracks
                .select((album, year))
                .order(year)
                .filter(artist.like(String::from("%") + &a + "%"))
                .group_by(album)
                .load(db.lock().expect("DB Error").deref())
                .expect("Error in db connection");
            let artist_node = model.insert_with_values(
                None,
                None,
                &[0, 1, 2, 3],
                &[&st, &a, &ARTIST_TYPE, DEFAULT_VISIBILITY],
            );

            for (ab, y) in albums {
                //add the year if it exists to the string we insert
                let abstring = if let Some(yp) = y {
                    yp.to_string() + " - " + &ab
                } else {
                    ab.to_string()
                };

                let album_node = model.insert_with_values(
                    Some(&artist_node),
                    None,
                    &[0, 1, 2, 3],
                    &[&abstring, &ab, &ALBUM_TYPE, DEFAULT_VISIBILITY],
                );
                {
                    let ts: Vec<String> = tracks
                        .select(title)
                        .order(tracknumber)
                        .filter(artist.like(String::from("%") + &a + "%"))
                        .filter(album.eq(ab))
                        .load(db.lock().expect("DB Error").deref())
                        .expect("Error in db connection");
                    for t in ts {
                        model.insert_with_values(
                            Some(&album_node),
                            None,
                            &[0, 1, 2, 3],
                            &[&t, &t, &TRACK_TYPE, DEFAULT_VISIBILITY],
                        );
                    }
                }
            }
        }
        gtk::Continue(true)
    } else {
        info!("Done");
        gtk::Continue(false)
    }
}

pub fn new(db: &DBPool, builder: &BuilderPtr, gui: &MainGuiPtr) {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};

    let libview: gtk::TreeView = builder.read().unwrap().get_object("libraryview").unwrap();
    // setup drag drop
    {
        let dbc = db.clone();
        let targets = vec![gtk::TargetEntry::new(
            "text/plain",
            gtk::TargetFlags::SAME_APP,
            0,
        )];
        libview.drag_source_set(
            gdk::ModifierType::MODIFIER_MASK,
            &targets,
            gdk::DragAction::COPY,
        );
        libview.connect_drag_data_get(move |w, _, s, _, _| {
            let (_, t) = get_tracks_for_selection(&dbc, &w).expect("Could not get tracks");
            let data = serde_json::to_string(&t).expect("Error in formating drop data");
            s.set_text(&data);
        });
    }

    //the model contains first a abbreviated string and in second column the whole string to construct the playlist
    let model = gtk::TreeStore::new(&[
        String::static_type(),
        String::static_type(),
        i32::static_type(),
        bool::static_type(),
    ]);
    let fmodel = gtk::TreeModelFilter::new(&model, None);
    fmodel.set_visible_column(3);

    let searchfield: gtk::SearchEntry = builder
        .read()
        .unwrap()
        .get_object("collectionsearch")
        .unwrap();
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

    let artists: Vec<String> = tracks
        .select(artist)
        .order(artist)
        .group_by(artist)
        .filter(artist.not_like(String::from("%") + "feat" + "%"))
        .filter(artist.ne(""))
        .load(db.lock().expect("DB Error").deref())
        .expect("Error in db connection");

    {
        let dbc = db.clone();
        let guic = gui.clone();
        libview.connect_event_after(move |s, e| signalhandler(&dbc, &guic, s, e));
    }
    let refcell = Rc::new(RefCell::new(artists.into_iter()));
    {
        let dbc = db.clone();
        gtk::idle_add(move || idle_fill(&dbc, &refcell, &model));
    }
}

fn idle_make_parents_visible_from_search(
    s: &Rc<String>,
    field: &Rc<gtk::SearchEntry>,
    treeiter: &Rc<gtk::TreeIter>,
    model: &Rc<gtk::TreeStore>,
) -> gtk::Continue {
    let visible: &gtk::Value = &true.to_value();
    //let invisible: &gtk::Value = &false.to_value();

    // abort if another thread is running
    if s.deref() != &field.get_text().unwrap().to_lowercase() {
        return gtk::Continue(false);
    }
    let mut parent = model.iter_parent(&treeiter); //make a new iterator because we will modify it
    while let Some(p) = parent {
        model.set_value(&p, 3, visible);
        parent = model.iter_parent(&p);
    }
    gtk::Continue(false)
}

fn idle_make_children_visible_from_search(
    s: &Rc<String>,
    field: &Rc<gtk::SearchEntry>,
    treeiter: &Rc<gtk::TreeIter>,
    model: &Rc<gtk::TreeStore>,
) -> gtk::Continue {
    let visible: &gtk::Value = &true.to_value();
    //let invisible: &gtk::Value = &false.to_value();

    // abort if another thread is running
    if s.deref() != &field.get_text().unwrap().to_lowercase() {
        return gtk::Continue(false);
    }

    let current_tree = (*treeiter.deref()).clone(); //make a new iterator because we will modify it
    let mut child_iterator = model.iter_children(Some(&current_tree));
    while let Some(child) = child_iterator {
        model.set_value(&child, 3, visible);

        //recursively
        {
            let childcopy = Rc::new(child.clone());
            let sc = s.clone();
            let fc = field.clone();
            let mc = model.clone();
            gtk::idle_add(move || {
                idle_make_children_visible_from_search(&sc, &fc, &childcopy, &mc)
            });
        }
        child_iterator = if model.iter_next(&child) {
            Some(child)
        } else {
            None
        };
    }
    gtk::Continue(false)
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
    model: Rc<gtk::TreeStore>,
) -> gtk::Continue {
    let visible: &gtk::Value = &true.to_value();
    let invisible: &gtk::Value = &false.to_value();

    // abort if another thread is running
    if *s != field.get_text().unwrap().to_lowercase() {
        //println!(
        //    "Killing this search thread because {:?}, {:?}",
        //    *s,
        //    field.get_text().unwrap()
        //);
        return gtk::Continue(false);
    }

    if !s.is_empty() {
        let val = model
            .get_value(&treeiter, 1)
            .get::<String>()
            .map(|v| v.to_lowercase().contains(&*s));

        if val == Some(true) {
            model.set_value(&treeiter, 3, visible);
            {
                //parents visible
                let sc = s.clone();
                let fc = field.clone();
                let mc = model.clone();
                let tc = Rc::new((*treeiter).clone());
                gtk::idle_add(move || {
                    idle_make_parents_visible_from_search(
                        &sc.clone(),
                        &fc.clone(),
                        &tc.clone(),
                        &mc.clone(),
                    )
                });
            }
            {
                //childrens visible
                let sc = s.clone();
                let fc = field.clone();
                let mc = model.clone();
                let tc = Rc::new((*treeiter).clone());
                gtk::idle_add(move || idle_make_children_visible_from_search(&sc, &fc, &tc, &mc));
            }
        } else {
            //if we are not matched now, we might match a child, so go into children
            model.set_value(&treeiter, 3, invisible);
            let itt = (*treeiter).clone();
            if let Some(c) = model.iter_children(Some(&itt)) {
                let cc = Rc::new(c);
                let sc = s.clone();
                let fc = field.clone();
                let fmc = fmodel.clone();
                let mc = model.clone();
                gtk::idle_add(move || {
                    idle_search_changed(sc.clone(), fc.clone(), cc.clone(), fmc.clone(), mc.clone())
                });
            }
        }
    } else {
        model.set_value(&treeiter, 3, visible);

        //set everything to visible
        let it = Rc::new((*treeiter).clone());
        let sc = s.clone();
        let fc = field.clone();
        let mc = model.clone();
        gtk::idle_add(move || idle_make_children_visible_from_search(&sc, &fc, &it, &mc));
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

fn search_changed(s: &gtk::SearchEntry, builder: &BuilderPtr) {
    let libview: gtk::TreeView = builder.read().unwrap().get_object("libraryview").unwrap();

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
            .downcast::<gtk::TreeStore>()
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

fn get_model_and_iter_for_selection(tv: &gtk::TreeView) -> (gtk::TreeStore, gtk::TreeIter) {
    let (model, iter) = tv.get_selection().get_selected().unwrap();
    let filtermodel = model
        .downcast::<gtk::TreeModelFilter>()
        .expect("Error in TreeModelFilter downcast");
    let m = filtermodel
        .get_model()
        .expect("No base model for TreeModelFilter")
        .downcast::<gtk::TreeStore>()
        .expect("Error in TreeStore downcast");
    let realiter = filtermodel.convert_iter_to_child_iter(&iter);

    (m, realiter)
}

fn get_tracks_for_selection(
    db: &DBPool,
    tv: &gtk::TreeView,
) -> Result<(String, Vec<Track>), String> {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods};

    let (m, iter) = get_model_and_iter_for_selection(tv);

    info!("Iter depth: {}", m.iter_depth(&iter));

    let query = tracks.order(path).into_boxed();
    if m.iter_depth(&iter) == 0 {
        let artist_name = m.get_value(&iter, 1).get::<String>().unwrap();
        info!("artist: {}", artist_name);
        Ok((
            artist_name.clone(),
            query
                .filter(artist.like(String::from("%") + &artist_name + "%"))
                .load(db.lock().expect("DB Error").deref())
                .expect("Error in query"),
        ))
    } else if m.iter_depth(&iter) == 1 {
        let parent_artist = m
            .iter_parent(&iter)
            .expect("We do not have a parent, this is strange");
        let artist_name = m.get_value(&parent_artist, 1).get::<String>().unwrap();
        let album_name = m.get_value(&iter, 1).get::<String>().unwrap();
        info!(
            "doing with artist {}, album \"{}\"",
            artist_name, album_name
        );
        Ok((
            album_name.clone(),
            query
                .filter(artist.like(String::from("%") + &artist_name + "%"))
                .filter(album.eq(album_name))
                .load(db.lock().expect("DB Error").deref())
                .expect("Error in query"),
        ))
    } else if m.iter_depth(&iter) == 2 {
        let parent_album = m
            .iter_parent(&iter)
            .expect("We do not have a parent, this is strange");
        let parent_artist = m
            .iter_parent(&parent_album)
            .expect("We do not have a parent, this is strange");

        let artist_name = m.get_value(&parent_artist, 1).get::<String>().unwrap();
        let album_name = m.get_value(&parent_album, 1).get::<String>().unwrap();
        let track_name = m.get_value(&iter, 1).get::<String>().unwrap();
        Ok((
            track_name.clone(),
            query
                .filter(artist.like(String::from("%") + &artist_name + "%"))
                .filter(album.eq(album_name))
                .filter(title.eq(track_name))
                .load(db.lock().expect("DB Error").deref())
                .expect("Error in query"),
        ))
    } else {
        Err(format!("Found iter depth: {}", m.iter_depth(&iter)))
    }
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
*/

use crate::db;
use crate::types::*;
use std::collections::HashMap;
use std::ops::Deref;

pub type Track = String;
pub type Album = GeneralTreeViewJson<Track>;
pub type Artist = GeneralTreeViewJson<Album>;

impl From<String> for Album {
    fn from(s: String) -> Self {
        Album {
            name: s,
            children: vec![],
        }
    }
}

impl From<String> for Artist {
    fn from(s: String) -> Self {
        Artist {
            name: s,
            children: vec![],
        }
    }
}

fn get_track_vec(
    db: &diesel::SqliteConnection,
    artist_name: Option<&str>,
    album_name: Option<&str>,
) -> Vec<Track> {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    let mut query = tracks.into_boxed();
    if let Some(a) = artist_name {
        query = query
            .filter(artist.eq(a))
            .filter(artist.is_not_null())
            .filter(artist.is_not_null())
            .filter(artist.ne(""));
    }
    if let Some(a) = album_name {
        query = query
            .filter(album.eq(album_name.unwrap()))
            .filter(album.ne(""))
    }
    query
        .order_by(tracknumber)
        .load(db)
        .expect("Error in loading DB")
        .into_iter()
        .map(|t: db::Track| t.title.to_owned())
        .collect()
}

pub fn get_album_subtree(db: &diesel::SqliteConnection, artist_name: Option<&str>) -> Vec<Album> {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    let mut query = tracks.into_boxed();

    if let Some(a) = artist_name {
        query = query
            .filter(artist.eq(a))
            .filter(artist.is_not_null())
            .filter(album.is_not_null())
            .filter(artist.ne(""))
            .filter(album.ne(""));
    } else {
        query = query
            .filter(artist.is_not_null())
            .filter(album.is_not_null());
    }
    query
        .select((album, artist))
        .distinct()
        .order_by(album)
        .load(db)
        .expect("Error in loading DB")
        .into_iter()
        .map(|t: (String, String)| Album {
            name: t.0,
            children: get_track_vec(db, Some(&t.1), artist_name),
        })
        .collect()
}

pub fn get_tracks(pool: &DBPool) -> Vec<Track> {
    let db = pool.lock().expect("Error locking DB");
    get_track_vec(db.deref(), None, None)
}

pub fn get_album_trees(pool: &DBPool) -> Vec<Album> {
    let db = pool.lock().expect("Error locking DB");
    get_album_subtree(db.deref(), None)
}

fn track_to_artist(t: String, db: &diesel::SqliteConnection) -> Artist {
    println!("Doing artist: {:?}", t);
    Artist {
        name: t.chars().take(20).collect::<String>(),
        children: Vec::new(),
        //children: get_album_subtree(db, &Some(t)),
    }
}

/// basic query function to model tracks with the apropiate values selected
fn basic_tree_query<'a>(
    pool: &'a DBPool,
    artist_opt: &'a Option<String>,
    album_opt: &'a Option<String>,
    track_opt: &'a Option<String>,
) -> crate::schema::tracks::BoxedQuery<'a, diesel::sqlite::Sqlite> {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl};
    //let p = pool.lock().expect("Error in lock");
    let mut query = tracks
        .filter(artist.is_not_null())
        .filter(artist.ne(""))
        .distinct()
        .order_by(artist)
        .into_boxed();

    if let Some(a) = artist_opt {
        query = query.filter(artist.eq(a));
    }
    if let Some(a) = album_opt {
        query = query.filter(album.eq(a));
    }
    if let Some(t) = track_opt {
        query = query.filter(title.eq(t));
    }
    query
}

/// Queries the tree but only returns not filled in results, i.e., children might be unpopulated
pub fn query_partial_tree(
    pool: &DBPool,
    artist_opt: &Option<String>,
    album_opt: &Option<String>,
    track_opt: &Option<String>,
) -> Vec<Artist> {
    use crate::schema::tracks::dsl::*;
    use diesel::{GroupByDsl, QueryDsl, RunQueryDsl};
    let p = pool.lock().expect("Error in lock");
    let mut query = basic_tree_query(pool, artist_opt, album_opt, track_opt);
    if let Some(ref t) = track_opt {
        let res = query
            .select(title)
            .group_by(title)
            .load(p.deref())
            .expect("Error in loading");

        vec![Artist {
            name: artist_opt.to_owned().unwrap_or("Default".to_string()),
            children: vec![Album {
                name: album_opt.to_owned().unwrap_or("Default".to_string()),
                children: res,
            }],
        }]
    } else if let Some(ref ab) = album_opt {
        let res = query
            .select(album)
            .group_by(album)
            .load(p.deref())
            .expect("Error in Loading");

        vec![Artist {
            name: artist_opt.to_owned().unwrap_or("Default".to_string()),
            children: res.into_iter().map(|s: String| s.into()).collect(),
        }]
    } else if let Some(ref a) = artist_opt {
        let res = query
            .select(artist)
            .group_by(artist)
            .load(p.deref())
            .expect("Error in loading");

        res.into_iter().map(|s: String| s.into()).collect()
    } else {
        vec![]
    }
}

/// Queries the tree with the matching parameters, does not give us partials
pub fn query_tree(
    pool: &DBPool,
    artist_opt: &Option<String>,
    album_opt: &Option<String>,
    track_opt: &Option<String>,
) -> Vec<Artist> {
    use diesel::RunQueryDsl;
    let p = pool.lock().expect("Error in lock");
    let mut query = basic_tree_query(pool, artist_opt, album_opt, track_opt);

    let mut q_tracks: Vec<db::Track> = query.load(p.deref()).expect("Error in DB");

    //Artist, Album, Track
    let mut hashmap: HashMap<String, HashMap<String, Vec<db::Track>>> = HashMap::new();
    for t in q_tracks.drain(0..) {
        if let Some(ref mut artist_hash) = hashmap.get_mut(&t.artist) {
            if let Some(ref mut album_vec) = artist_hash.get_mut(&t.album) {
                album_vec.push(t);
            } else {
                let k = t.artist.clone();
                let v = vec![t];
                artist_hash.insert(k, v);
            }
        } else {
            let mut v = HashMap::new();
            let kb = t.album.clone();
            let ka = t.artist.clone();
            let v2 = vec![t];
            v.insert(kb, v2);
            hashmap.insert(ka, v);
        }
    }

    hashmap
        .drain()
        .map(|(k, mut m): (String, HashMap<String, Vec<db::Track>>)| {
            let mut children = m
                .drain()
                .map(|(k2, v): (String, Vec<db::Track>)| Album {
                    name: k2.clone(),
                    children: v.iter().map(|v| v.title.clone()).collect(),
                })
                .collect();
            Artist {
                name: k.clone(),
                children,
            }
        })
        .collect::<Vec<Artist>>()
}

// TODO This could be much more general by having the fill_fn in general
// TODO Try to make this iterator stuff, at the moment it doesn't need json
// TODO we need to not have the empty strings for stuff around
/*
pub fn get_artist_trees(pool: &DBPool) -> Vec<Artist> {
    use crate::schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
    let p = pool.lock().expect("Error in lock");
    tracks
        .select(artist)
        .filter(artist.is_not_null())
        .filter(artist.ne(""))
        .distinct()
        .order_by(artist)
        .load(p.deref())
        .expect("Error in DB")
        .into_iter()
        .map(move |t| track_to_artist(t, p.deref()))
        .take(10)
        .collect()
}
*/
