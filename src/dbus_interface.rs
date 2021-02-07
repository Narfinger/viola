use crate::{
    gstreamer_wrapper::{GStreamer, GStreamerExt},
    loaded_playlist::LoadedPlaylistExt,
    playlist_tabs::PlaylistTabsExt,
    types::*,
};
use dbus::blocking::Connection;
use dbus_crossroads::{Context, Crossroads, IfaceBuilder};
use std::{collections::HashMap, error::Error, sync::Arc, thread};

struct DbusStruct {
    gstreamer: Arc<GStreamer>,
    playlisttabs: PlaylistTabsPtr,
}

fn main(gstreamer: Arc<GStreamer>, playlisttabs: PlaylistTabsPtr) -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub(crate) fn new(gstreamer: Arc<GStreamer>, playlisttabs: PlaylistTabsPtr) {
    thread::spawn(|| main(gstreamer, playlisttabs).expect("Error in starting dbus"));
}
