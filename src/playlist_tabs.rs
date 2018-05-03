use gtk;
use std::rc::Rc;
use std::cell::RefCell;

use db;
use playlist::LoadedPlaylist;
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
    fn current_position(&self) -> usize;
}

impl PlaylistTabsExt for PlaylistTabsPtr {
    fn current_track<'a>(&self) -> &'a db::Track {
        panic!("NOT IMPLEMENTED YED");
    }

    fn current_position(&self) -> usize {
        self.tabs[self.current_playlist.unwrap()].current_position
    }
}