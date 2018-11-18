use db::Track;
use gdk;
use gtk;
use gtk::prelude::*;
use loaded_playlist::LoadedPlaylist;
use serde_json;
use std::cell::RefCell;
use std::rc::Rc;
use std::string::String;

use maingui::{MainGuiExt, MainGuiPtrExt};
use types::*;

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

fn idle_fill<I>(pool: &DBPool, ats: &Rc<RefCell<I>>, model: &gtk::TreeStore) -> gtk::Continue
where
    I: Iterator<Item = String>,
{
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
                .select((album, year))
                .order(year)
                .filter(artist.like(String::from("%") + &a + "%"))
                .group_by(album)
                .load(&db)
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
                        .load(&db)
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

pub fn new(pool: &DBPool, builder: &BuilderPtr, gui: &MainGuiPtr) {
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl, TextExpressionMethods};
    use schema::tracks::dsl::*;

    let libview: gtk::TreeView = builder.read().unwrap().get_object("libraryview").unwrap();
    // setup drag drop
    {
        let pc = pool.clone();
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
            let (_, t) = get_tracks_for_selection(&pc, &w).expect("Could not get tracks");
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

    let db = pool.get().unwrap();
    let artists: Vec<String> = tracks
        .select(artist)
        .order(artist)
        .group_by(artist)
        .filter(artist.not_like(String::from("%") + "feat" + "%"))
        .filter(artist.ne(""))
        .load(&db)
        .expect("Error in db connection");

    {
        let pc = pool.clone();
        let guic = gui.clone();
        libview.connect_event_after(move |s, e| signalhandler(&pc, &guic, s, e));
    }
    let refcell = Rc::new(RefCell::new(artists.into_iter()));
    {
        let pc = pool.clone();
        gtk::idle_add(move || idle_fill(&pc, &refcell, &model));
    }
}

fn idle_make_parents_visible_from_search(s: Rc<String>,
    field: Rc<gtk::SearchEntry>,
    treeiter: Rc<gtk::TreeIter>,
    model: Rc<gtk::TreeStore>,
) -> gtk::Continue {
    let visible: &gtk::Value = &true.to_value();
    //let invisible: &gtk::Value = &false.to_value();

    // abort if another thread is running
    if *s != field.get_text().unwrap().to_lowercase() {
        //println!(
        //    "Killing this search thread because {:?}, {:?}",
        //    *s,
        //    field.get_text().unwrap()
        //);
        return gtk::Continue(false);
    }
    //Setting parents visible after we gone through it all.
    //This needs to be after because it interfers otherwise with the already visible set parents
    if model.get_value(&treeiter, 3).get::<bool>().unwrap() {
        let mut itt = (*treeiter).clone();  //make a new iterator because we will modify it
        let mut check_more_parents = true;
        while check_more_parents {
            let parent = model.iter_parent(&itt);
            model.set_value(&itt, 3, visible);
            //let v = model.get_value(&it, 1).get::<String>().unwrap();
            //println!("Doing parents {}", v);
            if let Some(parent) = parent {
                check_more_parents = true;
                itt = parent;
            } else {
                check_more_parents = false;
            }
        }
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
        //println!("Looking at: {:?}, {:?}, {:?}", model.get_value(&treeiter, 1).get::<String>(), val, s);
        let parent_visible = {
            let it = (*treeiter).clone();
            
            //is parent visible?
            let parent = model
                        .iter_parent(&it)
                        .and_then(|pit| model.get_value(&pit, 3).get::<bool>());
            //is parent of parent visible?
            let pparent = model
                        .iter_parent(&it)
                        .and_then(|pit| model
                                        .iter_parent(&pit)
                                        .and_then(|ppit| model.get_value(&ppit,3).get::<bool>()));

            (parent == Some(true)) | (pparent == Some(true))
        };

        if val == Some(true) {
            model.set_value(&treeiter, 3, visible);
        } else if parent_visible {
            model.set_value(&treeiter, 3, visible);
        } else {
            model.set_value(&treeiter, 3, invisible);
        }

    } else {
        model.set_value(&treeiter, 3, visible);
    }

    // check the children if they exist
    {
        let olditer = Rc::new((*treeiter).clone());
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
        } else {
            //now we need to enable the parents we might have made invisible
            //if we do not find a valid next, this means we are at the end of the iterator and gone through all the elements
            let sc = s.clone();
            let fc = field.clone();
            let mc = model.clone();
            let tc = olditer.clone();
            gtk::idle_add(move || {
                idle_make_parents_visible_from_search(
                    sc.clone(),
                    fc.clone(),
                    tc.clone(),
                    mc.clone(),
                )
            });
        }
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
    pool: &DBPool,
    tv: &gtk::TreeView,
) -> Result<(String, Vec<Track>), String> {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods};
    use schema::tracks::dsl::*;

    let (m, iter) = get_model_and_iter_for_selection(tv);

    info!("Iter depth: {}", m.iter_depth(&iter));

    let db = pool.get().expect("DB problem");
    let query = tracks.order(path).into_boxed();
    if m.iter_depth(&iter) == 0 {
        let artist_name = m.get_value(&iter, 1).get::<String>().unwrap();
        info!("artist: {}", artist_name);
        Ok((
            artist_name.clone(),
            query
                .filter(artist.like(String::from("%") + &artist_name + "%"))
                .load(&db)
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
                .load(&db)
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
                .load(&db)
                .expect("Error in query"),
        ))
    } else {
        Err(format!("Found iter depth: {}", m.iter_depth(&iter)))
    }
}

fn do_new(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView) {
    let (name, res) = get_tracks_for_selection(pool, tv).expect("Error in getting tracks");
    let pl = LoadedPlaylist {
        id: None,
        name,
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
    let (_, res) = get_tracks_for_selection(pool, tv).expect("Error in getting tracks");
    gui.replace_playlist(res);
}

fn signalhandler(pool: &DBPool, gui: &MainGuiPtr, tv: &gtk::TreeView, event: &gdk::Event) {
    if event.get_event_type() == gdk::EventType::ButtonPress {
        info!("button press");
        if let Ok(b) = event.clone().downcast::<gdk::EventButton>() {
            info!("the button: {}", b.get_button());
            if b.get_button() == 3 {
                let mut menu = gtk::Menu::new();
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
                gtk::MenuExt::popup_at_pointer(&menu, event);
            }
        }
    }
}
