use crate::{gstreamer_wrapper::GStreamer, types::*};
use dbus::blocking::Connection;
use dbus_crossroads::{Context, Crossroads, IfaceBuilder};
use std::{error::Error, sync::Arc, thread};

struct DbusStruct {
    gstreamer: Arc<GStreamer>,
}

fn main(gstreamer: Arc<GStreamer>) -> Result<(), Box<dyn Error>> {
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
        "org.mpris.MediaPlayer2",
        |b: &mut IfaceBuilder<DbusStruct>| {},
    );

    cr.insert(
        "/org.mpris.MediaPlayer2",
        &[mediaplayer2, mediaplayer2player],
        DbusStruct { gstreamer },
    );
    // Serve clients forever.
    cr.serve(&c)?;
    Ok(())
}

pub(crate) fn new(gstreamer: Arc<GStreamer>) {
    thread::spawn(|| main(gstreamer).expect("Error in starting dbus"));
}
