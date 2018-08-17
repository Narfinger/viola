use gdk;
use gtk;
use gtk::prelude::*;
use gtk::{ListStoreExt, ListStoreExtManual, TreeModelExt, TreeSelectionExt};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use db;
use loaded_playlist::{LoadedPlaylist, LoadedPlaylistExt, PlaylistControls};
use maingui::MainGuiPtrExt;
use playlist;
use types::*;

#[derive(Clone, Debug)]
pub struct PlaylistTab {
    pub lp: LoadedPlaylist,
    pub treeview: gtk::TreeView,
    pub model: gtk::ListStore,
}

/// FIXME: clean up this section and make the various traits into different files

/// Loads a playlist, returning the ScrolledWindow, containing the treeview and creating a PlaylistTab
pub fn load_tab(
    tabs: &PlaylistTabsPtr,
    gui: MainGuiPtr,
    lp: LoadedPlaylist,
) -> (gtk::ScrolledWindow, PlaylistTab) {
    /// FIXME clean this up
    let model = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        gdk::RGBA::static_type(),
    ]);
    let treeview = gtk::TreeView::new();
    treeview
        .get_selection()
        .set_mode(gtk::SelectionMode::Multiple);

    for &(id, title, width) in &[
        (0, "#", 50),
        (1, "Title", 500),
        (2, "Artist", 200),
        (3, "Album", 300),
        (4, "Length", 150),
        (5, "Year", 100),
        (6, "Genre", 150),
    ] {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.pack_start(&cell, true);
        // Association of the view's column with the model's `id` column.
        column.add_attribute(&cell, "text", id);
        column.add_attribute(&cell, "background-rgba", 7);
        column.set_title(title);
        column.set_resizable(id > 0);
        column.set_fixed_width(width);
        treeview.append_column(&column);
        //center the column for length and year
        if (id == 4) | (id == 5) {
            cell.set_property_xalign(0.5);
        }
    }
    treeview.set_model(Some(&model));

    {
        let tabsc = tabs.clone();
        treeview.connect_key_press_event(move |tv, event| key_signal_handler(&tabsc, &tv, event));
    }

    {
        treeview.connect_button_press_event(move |tv, event| gui.clone().signal_handler(tv, event));
    }

    append_treeview_from_vector(&lp.items, &model);
    let scw = gtk::ScrolledWindow::new(None, None);
    scw.add(&treeview);

    let tab = PlaylistTab {
        lp,
        treeview,
        model,
    };

    (scw, tab)
}

//yes... this is werid, I don't know why there are not constants
const DELETE_KEY: u32 = 65535;

/// Handles keyboard presses in treeviews/playlistviews
fn key_signal_handler(
    tabs: &PlaylistTabsPtr,
    tv: &gtk::TreeView,
    event: &gdk::Event,
) -> gtk::Inhibit {
    //println!("key {:?}", event.get_event_type());
    if event.get_event_type() == gdk::EventType::KeyPress {
        if let Ok(b) = event.clone().downcast::<gdk::EventKey>() {
            //println!("event key {}", b.get_keyval());
            if b.get_keyval() == DELETE_KEY {
                tabs.borrow_mut().remove_items(tv.get_selection());
                tv.get_selection().unselect_all();
            }
        }
    }
    gtk::Inhibit(false)
}

pub struct PlaylistTabs {
    pub current_playlist: Option<usize>,
    pub tabs: Vec<PlaylistTab>,
}

impl Drop for PlaylistTabs {
    fn drop(&mut self) {
        panic!("implemente saving");
    }
}

pub fn new() -> PlaylistTabsPtr {
    println!("implemente restoring");
    Rc::new(RefCell::new(PlaylistTabs {
        current_playlist: None,
        tabs: Vec::new(),
    }))
}

pub trait PlaylistTabsExt {
    /// Returns the current track
    fn current_track(&self) -> &db::Track;

    /// returns the current position in the current playlist
    fn current_position(&self) -> i32;

    /// gets the playlist id (if it exists)
    fn id(&self, i32) -> Option<i32>;

    /// set the current playlist, used for changing tabs
    fn set_current_playlist(&mut self, i32);

    /// add a new tab
    fn add_tab(&mut self, PlaylistTab);

    /// remove the tab given by the index
    fn remove_tab(&mut self, i32) -> Option<i32>;

    /// removes the items from the vector
    fn remove_items(&mut self, gtk::TreeSelection);

    /// append the tracks to current playlists
    fn append_to_playlist(&mut self, Vec<db::Track>);

    /// replaces all currents track in current playlist
    fn replace_playlist(&mut self, Vec<db::Track>);

    /// insert the tracks at the integer given
    fn insert_tracks(&mut self, i32, Vec<db::Track>);

    /// saves the playlist tabs to the database
    fn save(&self, &DBPool);
}

impl PlaylistTabsExt for PlaylistTabs {
    fn current_track(&self) -> &db::Track {
        let pos = self.current_playlist.unwrap();
        self.tabs[pos as usize].lp.get_current_track()
    }

    fn current_position(&self) -> i32 {
        self.tabs[self.current_playlist.unwrap()]
            .lp
            .current_position
    }

    fn id(&self, index: i32) -> Option<i32> {
        self.tabs[index as usize].lp.id
    }

    fn set_current_playlist(&mut self, index: i32) {
        self.current_playlist = Some(index as usize)
    }

    fn add_tab(&mut self, plt: PlaylistTab) {
        self.tabs.push(plt);
        if self.tabs.len() == 1 {
            self.current_playlist = Some(0);
        }
    }

    fn remove_tab(&mut self, index: i32) -> Option<i32> {
        self.tabs.remove(index as usize);
        if self.current_playlist.unwrap() >= self.tabs.len() {
            Some(0)
        } else {
            None
        }
    }

    fn remove_items(&mut self, selection: gtk::TreeSelection) {
        let (vecpath, _) = selection.get_selected_rows();
        let index = self.current_playlist.unwrap();

        let mut rows = vecpath
            .into_iter()
            .flat_map(|mut v| v.get_indices_with_depth())
            .collect::<Vec<i32>>();
        // sort descending
        rows.sort_unstable_by(|x, y| y.cmp(x));

        let mut new_lp = self.tabs[index].lp.clone();

        {
            //model needs to go out of scope
            let model = &self.tabs[index].model;
            let mut position_adjustment = 0;
            let mut invalid_position = false;
            for i in rows {
                if i < new_lp.current_position {
                    position_adjustment += 1;
                } else if i == new_lp.current_position {
                    invalid_position = true;
                }
                //println!("deleting {}", i);
                new_lp.items.remove(i as usize);

                //deleting in view
                let iter = model.iter_nth_child(None, i).expect("Could not get iter");
                model.remove(&iter);
            }

            //correcting the current position index
            if invalid_position {
                new_lp.current_position = 0;
            } else {
                new_lp.current_position -= position_adjustment;
            }
        }
        self.tabs[index].lp = new_lp;
    }

    fn append_to_playlist(&mut self, t: Vec<db::Track>) {
        let mut items = self.tabs[self.current_playlist.unwrap()].lp.items.clone();
        let mut tp = t.clone();
        items.append(&mut tp);
        self.tabs[self.current_playlist.unwrap()].lp.items = items;
        let model = &self.tabs[self.current_playlist.unwrap()].model;

        append_treeview_from_vector(&t, model);
    }

    fn replace_playlist(&mut self, t: Vec<db::Track>) {
        {
            let model = &self.tabs[self.current_playlist.unwrap()].model;
            model.clear();
            append_treeview_from_vector(&t, model);
        }

        self.tabs[self.current_playlist.unwrap()].lp.items = t;
        self.tabs[self.current_playlist.unwrap()]
            .lp
            .current_position = 0;
    }

    fn insert_tracks(&mut self, index: i32, tracks: Vec<db::Track>) {
        let mut items = self.tabs[self.current_playlist.unwrap()].lp.items.clone();
        let mut i = index;
        for t in tracks {
            items.insert(i as usize, t);
            i += 1;
        }

        {
            let model = &self.tabs[self.current_playlist.unwrap()].model;
            model.clear();
            append_treeview_from_vector(&items, model);
        }
        self.tabs[self.current_playlist.unwrap()].lp.items = items;
    }

    fn save(&self, pool: &DBPool) {
        for lp in &self.tabs {
            playlist::update_playlist(pool, &lp.lp);
        }
    }
}

impl PlaylistControls for PlaylistTabs {
    fn get_current_path(&self) -> PathBuf {
        let lp = &self.tabs[self.current_playlist.unwrap()].lp;
        lp.get_current_path()
    }

    fn get_current_uri(&self) -> String {
        let lp = &self.tabs[self.current_playlist.unwrap()].lp;
        lp.get_current_uri()
    }

    fn previous(&mut self) -> String {
        self.tabs[self.current_playlist.unwrap()].lp.previous()
    }

    fn next(&mut self) -> String {
        self.tabs[self.current_playlist.unwrap()].lp.next()
    }

    fn set(&mut self, i: i32) -> String {
        self.tabs[self.current_playlist.unwrap()].lp.set(i)
    }

    fn next_or_eol(&mut self) -> Option<String> {
        self.tabs[self.current_playlist.unwrap()].lp.next_or_eol()
    }
}

pub trait PlaylistControlsImmutable {
    fn get_current_uri(&self) -> String;
    fn previous(&self) -> String;
    fn next(&self) -> String;
    fn set(&self, i32) -> String;
    fn next_or_eol(&self) -> Option<String>;
}

impl PlaylistControlsImmutable for PlaylistTabsPtr {
    fn get_current_uri(&self) -> String {
        self.borrow().get_current_uri()
    }

    fn previous(&self) -> String {
        self.borrow_mut().previous()
    }

    fn next(&self) -> String {
        self.borrow_mut().next()
    }

    fn set(&self, i: i32) -> String {
        self.borrow_mut().set(i)
    }

    fn next_or_eol(&self) -> Option<String> {
        self.borrow_mut().next_or_eol()
    }
}

/// This only modifies the treeview, not any underlying structure
fn append_treeview_from_vector(v: &[db::Track], model: &gtk::ListStore) {
    for entry in v {
        let length = format_duration(entry.length);
        model.insert_with_values(
            None,
            &[0, 1, 2, 3, 4, 5, 6, 7],
            &[
                &entry
                    .tracknumber
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| String::from("")),
                &entry.title,
                &entry.artist,
                &entry.album,
                &length,
                &entry
                    .year
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| String::from("")),
                &entry.genre,
                &gdk::RGBA {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 0.0,
                },
            ],
        );
    }
}

fn format_duration(d: i32) -> String {
    if d < 60 {
        format!("{}", d)
    } else if d < 60 * 60 {
        let s = d % 60;
        let m = d / 60;
        format!("{}:{:02}", m, s)
    } else {
        let s = d % 60;
        let m = d / 60 % (60 * 60);
        let h = d / (60 * 60);
        format!("{}:{:02}:{:02}", h, m, s)
    }
}
