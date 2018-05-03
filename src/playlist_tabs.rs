use gtk;
use std::rc::Rc;
use std::cell::RefCell;

use db;
use loaded_playlist::{LoadedPlaylist, LoadedPlaylistExt, PlaylistControls};
use types::*;

#[derive(Clone,Debug)]
pub struct PlaylistTab {
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
    fn current_track<'a>(&'a self) -> &'a db::Track;
    fn current_position(&self) -> i32;
}

impl PlaylistTabsExt for PlaylistTabs {
    fn current_track<'a>(&'a self) -> &'a db::Track {
        let pos = self.current_playlist.unwrap();
        self.tabs[pos as usize].lp.get_current_track()
    }

    fn current_position(&self) -> i32 {
        self.tabs[self.current_playlist.unwrap()].lp.current_position
    }
}

impl PlaylistControls for PlaylistTabs {
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