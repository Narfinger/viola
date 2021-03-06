use glib::object;
use parking_lot::RwLock;
use std::{
    collections::HashMap, convert::TryInto, error::Error, sync::Arc, thread, time::Duration,
};

use crate::{
    gstreamer_wrapper::{GStreamer, GStreamerExt},
    loaded_playlist::LoadedPlaylistExt,
    types::*,
};
use viola_common::{GStreamerAction, GStreamerMessage};
use zbus::{dbus_interface, export::zvariant, fdo};

struct BaseInterface {}

#[dbus_interface(name = "org.mpris.MediaPlayer2")]
impl BaseInterface {
    #[dbus_interface(property)]
    fn can_quit(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn fullscreen(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn can_set_fullscreen(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn can_raise(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn has_track_list(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn identity(&self) -> String {
        String::from("Viola")
    }

    #[dbus_interface(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        vec![]
    }

    #[dbus_interface(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        vec![]
    }

    //methods
    fn raise(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    fn quit(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }
}

struct PlayerInterface {
    gstreamer: Arc<RwLock<GStreamer>>,
    playlisttabs: PlaylistTabsPtr,
}

#[dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
impl PlayerInterface {
    #[dbus_interface(property)]
    fn playback_status(&self) -> String {
        self.gstreamer.read().get_state().to_string()
    }

    #[dbus_interface(property)]
    fn loop_status(&self) -> String {
        "None".to_string()
    }

    #[dbus_interface(property)]
    fn rate(&self) -> f64 {
        1.0
    }

    #[dbus_interface(property)]
    fn metadata(&self) -> HashMap<&str, zvariant::Value> {
        let mut map = HashMap::new();
        let track = self.playlisttabs.get_current_track();
        let length = 1000000 * track.length;
        map.insert("xesam:artist", track.artist.into());
        map.insert("xesam:album", track.album.into());
        map.insert("xesam:title", track.title.into());
        map.insert("mpris:length", length.into());
        map
    }

    #[dbus_interface(property)]
    fn volume(&self) -> f64 {
        1.0
    }

    #[dbus_interface(property)]
    fn position(&self) -> i64 {
        1000000 * self.gstreamer.read().get_elapsed().unwrap_or(0) as i64
    }

    #[dbus_interface(property)]
    fn minimum_rate(&self) -> f64 {
        1.0
    }

    #[dbus_interface(property)]
    fn maximum_rate(&self) -> f64 {
        1.0
    }

    #[dbus_interface(property)]
    fn can_go_next(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_go_previous(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_play(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_pause(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_seek(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn can_control(&self) -> bool {
        true
    }

    #[dbus_interface(signal)]
    fn properties_changed(&self) -> zbus::Result<()>;

    //methods
    fn next(&self) -> zbus::fdo::Result<()> {
        self.gstreamer
            .write()
            .do_gstreamer_action(GStreamerAction::Next);
        Ok(())
    }

    fn previous(&self) -> zbus::fdo::Result<()> {
        self.gstreamer
            .write()
            .do_gstreamer_action(GStreamerAction::Previous);
        Ok(())
    }

    fn pause(&self) -> zbus::fdo::Result<()> {
        println!("dbus send pause");
        self.gstreamer
            .write()
            .do_gstreamer_action(GStreamerAction::Pausing);
        Ok(())
    }

    fn play(&self) -> zbus::fdo::Result<()> {
        println!("dbus send play");
        self.gstreamer
            .write()
            .do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn play_pause(&self) -> zbus::fdo::Result<()> {
        println!("dbus send playpause");
        if self.gstreamer.read().get_state() == GStreamerMessage::Pausing {
            self.gstreamer
                .write()
                .do_gstreamer_action(GStreamerAction::Playing);
        } else {
            self.gstreamer
                .write()
                .do_gstreamer_action(GStreamerAction::Pausing);
        }
        Ok(())
    }

    fn stop(&self) -> zbus::fdo::Result<()> {
        println!("dbus send pause");
        self.gstreamer
            .write()
            .do_gstreamer_action(GStreamerAction::Pausing);
        Ok(())
    }

    fn seek(&self, _position: i32) -> zbus::fdo::Result<()> {
        //Not Implemented
        Ok(())
    }

    fn set_position(&self, _track_id: String, _position: i32) -> zbus::fdo::Result<()> {
        //Not Implemented
        Ok(())
    }

    fn open_uri(&self, _s: String) -> zbus::fdo::Result<()> {
        Ok(())
    }
}

fn main(
    gstreamer: Arc<RwLock<GStreamer>>,
    playlisttabs: PlaylistTabsPtr,
    bus: tokio::sync::watch::Receiver<GStreamerMessage>,
) -> Result<(), Box<dyn Error>> {
    let connection = zbus::Connection::new_session()?;
    fdo::DBusProxy::new(&connection)?.request_name(
        "org.mpris.MediaPlayer2.Viola",
        fdo::RequestNameFlags::ReplaceExisting.into(),
    )?;
    let mut object_server = zbus::ObjectServer::new(&connection);
    let handler = BaseInterface {};
    object_server.at(&"/org/mpris/MediaPlayer2".try_into()?, handler)?;
    let handler2 = PlayerInterface {
        gstreamer: gstreamer.clone(),
        playlisttabs: playlisttabs.clone(),
    };
    object_server.at(&"/org/mpris/MediaPlayer2".try_into()?, handler2)?;
    //tokio::spawn(async {
    //    if bus.changed().await.is_ok() {
    //        object_server.with(
    //            &"/org/mpris/MediaPlayer2".try_into().expect("Error"),
    //            |iface: &PlayerInterface| iface.properties_changed(),
    //        );
    //    }
    //});

    loop {
        if let Err(err) = object_server.try_handle_next() {
            println!("working on dbus message");
            eprintln!("{}", err);
        }
        //if bus.changed().into_future().poll() {
        //    object_server.with(
        //        &"/org/mpris/MediaPlayer2".try_into().expect("Error"),
        //        |iface: &PlayerInterface| iface.properties_changed(),
        //    );
        //}
        thread::sleep(Duration::new(1, 0));
    }
}

pub(crate) fn new(
    gstreamer: Arc<RwLock<GStreamer>>,
    playlisttabs: PlaylistTabsPtr,
    bus: tokio::sync::watch::Receiver<GStreamerMessage>,
) {
    thread::spawn(|| main(gstreamer, playlisttabs, bus).expect("Error in starting dbus"));
}
