use std::{collections::HashMap, convert::TryInto, error::Error, sync::Arc, thread};

use crate::{
    gstreamer_wrapper::{GStreamer, GStreamerExt},
    loaded_playlist::LoadedPlaylistExt,
    playlist_tabs::PlaylistTabsExt,
    types::*,
};
use viola_common::{GStreamerAction, GStreamerMessage};
use zbus::{dbus_interface, export::zvariant, fdo};

struct BaseInterface {
    gstreamer: Arc<GStreamer>,
    playlisttabs: PlaylistTabsPtr,
}

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
    gstreamer: Arc<GStreamer>,
    playlisttabs: PlaylistTabsPtr,
}

#[dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
impl PlayerInterface {
    #[dbus_interface(property)]
    fn playback_status(&self) -> String {
        self.gstreamer.get_state().to_string()
    }

    #[dbus_interface(property)]
    fn loop_status(&self) -> String {
        "None".to_string()
    }

    #[dbus_interface(property)]
    fn rate(&self) -> i32 {
        44100
    }

    #[dbus_interface(property)]
    fn metadata(&self) -> HashMap<&str, zvariant::Value> {
        let mut map = HashMap::new();
        let track = self.playlisttabs.get_current_track();
        map.insert("xesam:artist", track.artist.into());
        map.insert("xesam:album", track.album.into());
        map.insert("xesam:title", track.title.into());
        map.insert("mpris:length", track.length.into());
        map
    }

    #[dbus_interface(property)]
    fn volume(&self) -> i32 {
        50
    }

    #[dbus_interface(property)]
    fn position(&self) -> i32 {
        self.gstreamer.get_elapsed().unwrap_or(0) as i32
    }

    #[dbus_interface(property)]
    fn minimum_rate(&self) -> i32 {
        1
    }

    #[dbus_interface(property)]
    fn maximum_rate(&self) -> i32 {
        1
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
        self.gstreamer.do_gstreamer_action(GStreamerAction::Next);
        Ok(())
    }

    fn previous(&self) -> zbus::fdo::Result<()> {
        self.gstreamer
            .do_gstreamer_action(GStreamerAction::Previous);
        Ok(())
    }

    fn pause(&self) -> zbus::fdo::Result<()> {
        self.gstreamer.do_gstreamer_action(GStreamerAction::Pausing);
        Ok(())
    }

    fn play_pause(&self) -> zbus::fdo::Result<()> {
        if self.gstreamer.get_state() == GStreamerMessage::Pausing {
            self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        } else {
            self.gstreamer.do_gstreamer_action(GStreamerAction::Pausing);
        }
        Ok(())
    }

    fn stop(&self) -> zbus::fdo::Result<()> {
        self.gstreamer.do_gstreamer_action(GStreamerAction::Pausing);
        Ok(())
    }

    fn seek(&self, position: i32) -> zbus::fdo::Result<()> {
        todo!("todo")
    }

    fn set_position(&self, track_id: String, position: i32) -> zbus::fdo::Result<()> {
        todo!("todo")
    }

    fn open_uri(&self, s: String) -> zbus::fdo::Result<()> {
        todo!("todo")
    }
}

fn main(gstreamer: Arc<GStreamer>, playlisttabs: PlaylistTabsPtr) -> Result<(), Box<dyn Error>> {
    let connection = zbus::Connection::new_session()?;
    fdo::DBusProxy::new(&connection)?.request_name(
        "org.mpris.MediaPlayer2.Viola",
        fdo::RequestNameFlags::ReplaceExisting.into(),
    )?;
    let mut object_server = zbus::ObjectServer::new(&connection);
    let handler = BaseInterface {
        gstreamer: gstreamer.clone(),
        playlisttabs: playlisttabs.clone(),
    };
    object_server.at(&"/org/mpris/MediaPlayer2".try_into()?, handler)?;
    let handler2 = PlayerInterface {
        gstreamer: gstreamer.clone(),
        playlisttabs: playlisttabs.clone(),
    };
    object_server.at(&"/org/mpris/MediaPlayer2".try_into()?, handler2)?;
    loop {
        if let Err(err) = object_server.try_handle_next() {
            eprintln!("{}", err);
        }
    }
    Ok(())
}

pub(crate) fn new(gstreamer: Arc<GStreamer>, playlisttabs: PlaylistTabsPtr) {
    thread::spawn(|| main(gstreamer, playlisttabs).expect("Error in starting dbus"));
}
