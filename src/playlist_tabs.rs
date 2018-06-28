use gtk;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;
use gtk::{ListStoreExt, TreeModelExt, TreeSelectionExt};

use db;
use loaded_playlist::{LoadedPlaylist, LoadedPlaylistExt, PlaylistControls};
use playlist;
use types::*;

#[derive(Clone,Debug)]
pub struct PlaylistTab {
    pub lp: LoadedPlaylist,
    pub treeview: gtk::TreeView,
    pub model: gtk::ListStore,
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
    Rc::new(
        RefCell::new(
            PlaylistTabs { current_playlist: None, tabs: Vec::new() }
        )
    )
}

pub trait PlaylistTabsExt {
    /// Returns the current track
    fn current_track<'a>(&'a self) -> &'a db::Track;

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

    /// saves the playlist tabs to the database
    fn save(&self, &DBPool);
}

impl PlaylistTabsExt for PlaylistTabs {
    fn current_track<'a>(&'a self) -> &'a db::Track {
        let pos = self.current_playlist.unwrap();
        self.tabs[pos as usize].lp.get_current_track()
    }

    fn current_position(&self) -> i32 {
        self.tabs[self.current_playlist.unwrap()].lp.current_position
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

        let mut rows = vecpath.into_iter().flat_map(|mut v| v.get_indices_with_depth()).collect::<Vec<i32>>();
        // sort descending
        rows.sort_unstable_by(|x,y| y.cmp(x));

        let mut new_lp = self.tabs[index].lp.clone();

        {   //model needs to go out of scope
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

    fn set(&mut self, i: i32) -> String  {
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