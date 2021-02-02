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
    // Let's start by starting up a connection to the session bus and request a name.
    let c = Connection::new_session()?;
    c.request_name("org.mpris.MediaPlayer2.viola", false, true, false)?;

    // Create a new crossroads instance.
    // The instance is configured so that introspection and properties interfaces
    // are added by default on object path additions.
    let mut cr = Crossroads::new();

    // Let's build a new interface, which can be used for "Hello" objects.

    let mediaplayer2 = cr.register(
        "org.mpris.MediaPlayer2",
        |b: &mut IfaceBuilder<DbusStruct>| {
            b.property("CanQuit").get(|_, _| Ok(false));
            b.property("CanRaise").get(|_, _| Ok(false));
            b.property("HasTrackList").get(|_, _| Ok(false));
            b.property("Identity").get(|_, _| Ok("viola".to_string()));
            b.property("SupportedUriSchemes")
                .get(|_, _| Ok("".to_string()));
            b.property("SupportedMimeTypes")
                .get(|_, _| Ok("".to_string()));
        },
    );

    let mediaplayer2player = cr.register(
        "org.mpris.MediaPlayer2.Player",
        |b: &mut IfaceBuilder<DbusStruct>| {
            b.property("PlaybackStatus")
                .get(|_, b| Ok(b.gstreamer.get_state().to_string()));
            b.property("Metadata").get(|_, data| {
                let mut m = HashMap::new();
                m.insert(
                    "xesam:album".to_string(),
                    data.playlisttabs.get_current_track().album,
                );
                m.insert(
                    "xesam:artist".to_string(),
                    data.playlisttabs.get_current_track().artist,
                );
                m.insert(
                    "xesam:title".to_string(),
                    data.playlisttabs.get_current_track().title,
                );
                Ok(m)
            });
        },
    );

    cr.insert(
        "/org/mpris/MediaPlayer2",
        &[mediaplayer2, mediaplayer2player],
        DbusStruct {
            gstreamer,
            playlisttabs,
        },
    );
    // Serve clients forever.
    cr.serve(&c)?;
    Ok(())
}

pub(crate) fn new(gstreamer: Arc<GStreamer>, playlisttabs: PlaylistTabsPtr) {
    thread::spawn(|| main(gstreamer, playlisttabs).expect("Error in starting dbus"));
}
