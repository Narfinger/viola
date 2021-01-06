use seed::prelude::*;

use viola_common::{GStreamerMessage, Track};

#[derive(Debug)]
pub(crate) struct Model {
    pub playlist_tabs: Vec<PlaylistTab>,
    pub playlist_window: PlaylistWindow,
    pub current_playlist_tab: usize,
    pub current_time: u64,
    pub play_status: GStreamerMessage,
    pub web_socket: WebSocket,
    pub is_repeat_once: bool,
    pub sidebar: Sidebar,
    pub treeviews: Vec<TreeView>,
    pub delete_range_input: Option<String>,
}
impl Model {
    pub fn get_current_playlist_tab_tracks_mut(&mut self) -> Option<&mut Vec<Track>> {
        self.playlist_tabs
            .get_mut(self.current_playlist_tab)
            .map(|tab| &mut tab.tracks)
    }

    pub fn get_current_playlist_tab_tracks(&self) -> Option<&Vec<Track>> {
        self.playlist_tabs
            .get(self.current_playlist_tab)
            .map(|tab| &tab.tracks)
    }

    pub fn get_current_playlist_tab(&mut self) -> Option<&PlaylistTab> {
        self.playlist_tabs.get(self.current_playlist_tab)
    }

    pub fn get_current_playlist_tab_mut(&mut self) -> Option<&mut PlaylistTab> {
        self.playlist_tabs.get_mut(self.current_playlist_tab)
    }
}

#[derive(Debug)]
pub(crate) struct Sidebar {
    pub smartplaylists: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct TreeView {
    pub tree: indextree::Arena<String>,
    pub root: indextree::NodeId,
    pub type_vec: Vec<viola_common::TreeType>,
    pub current_window: usize,
    pub stream_handle: Option<StreamHandle>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct PlaylistTab {
    pub tracks: Vec<Track>,
    pub name: String,
    pub current_index: usize,
}

/// Struct for having a window into our playlist and slowly fill it
#[derive(Debug, Default)]
pub(crate) struct PlaylistWindow {
    pub current_window: usize,
    pub stream_handle: Option<StreamHandle>,
}
