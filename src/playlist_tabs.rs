use gtk;
use std::rc::Rc;
use std::cell::RefCell;

use db;
use loaded_playlist::{LoadedPlaylist, PlaylistControls};
use types::*;

#[derive(Clone,Debug)]
pub struct PlaylistTab {
    /// TODO this probably does not need multithread safe
    pub lp: LoadedPlaylist,
    pub treeview: gtk::TreeView,
}

pub struct PlaylistTabs {
    pub current_playlist: Option<usize>,
    pub tabs: Vec<PlaylistTab>,
}

pub fn new() -> PlaylistTabsPtr {
    Rc::new(
        RefCell::new(
            PlaylistTabs { current_playlist: None, tabs: Vec::new() }
        )
    )
}

pub trait PlaylistTabsExt {
    fn current_track<'a>(&self) -> &'a db::Track;
    fn current_position(&self) -> i32;
}

impl PlaylistTabsExt for PlaylistTabs {
    fn current_track<'a>(&self) -> &'a db::Track {
        panic!("NOT IMPLEMENTED YED");
    }

    fn current_position(&self) -> i32 {
        self.tabs[self.current_playlist.unwrap()].lp.current_position
    }
}

impl PlaylistControls for PlaylistTabs {
    fn get_current_uri(&self) -> String {
        let lp = self.tabs[self.current_playlist.unwrap()].lp;
        lp.get_current_uri()
    }

    fn previous(&mut self) -> String {
        let mut lp = self.tabs[self.current_playlist.unwrap()].lp;
        lp.previous()
    }

    fn next(&mut self) -> String {
        let mut lp = self.tabs[self.current_playlist.unwrap()].lp;
        lp.next();
    }

    fn set(&mut self, i: i32) -> String  {
        let mut lp = self.tabs[self.current_playlist.unwrap()].lp;
        lp.set(i)
    }

    fn next_or_eol(&mut self) -> Option<String> {
        let mut lp = self.tabs[self.current_playlist.unwrap()].lp;
        lp.next_or_eol()
    }
}

impl PlaylistControls for PlaylistTabsPtr {
    fn get_current_uri(&self) -> String {
        self.borrow().get_current_uri()
    }

    fn previous(&self) -> String {
        let mut s = *self.borrow_mut();
        s.previous()
    }

    fn next(&self) -> String {
        self.borrow_mut().next()
    }

    fn set(&self, i: i32) -> String {
        self.borrow_mut().set(i)
    }

    fn next_or_eol(&self) -> String {
        self.borrow_mut().next_or_eol()
    }
}