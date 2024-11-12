use log::info;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

use crate::{gstreamer_wrapper::GStreamer, loaded_playlist::LoadedPlaylistExt, types::*};
use viola_common::{GStreamerAction, GStreamerMessage};
use zbus::{dbus_interface, interface, ConnectionBuilder};

struct BaseInterface {}

#[interface(name = "org.mpris.MediaPlayer2")]
impl BaseInterface {
    #[zbus(property)]
    async fn can_quit(&self) -> bool {
        false
    }

    #[zbus(property)]
    async fn fullscreen(&self) -> bool {
        false
    }

    #[zbus(property)]
    async fn can_set_fullscreen(&self) -> bool {
        false
    }

    #[zbus(property)]
    async fn can_raise(&self) -> bool {
        false
    }

    #[zbus(property)]
    async fn has_track_list(&self) -> bool {
        false
    }

    #[zbus(property)]
    async fn identity(&self) -> String {
        String::from("Viola")
    }

    #[zbus(property)]
    async fn supported_uri_schemes(&self) -> Vec<String> {
        vec![]
    }

    #[zbus(property)]
    async fn supported_mime_types(&self) -> Vec<String> {
        vec![]
    }

    //methods
    async fn raise(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    async fn quit(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }
}

struct PlayerInterface {
    gstreamer: Arc<GStreamer>,
    playlisttabs: PlaylistTabsPtr,
}

#[interface(name = "org.mpris.MediaPlayer2.Player")]
impl PlayerInterface {
    #[zbus(property)]
    async fn playback_status(&self) -> String {
        info!("dbus playback status");
        self.gstreamer.get_state().to_string()
    }

    #[zbus(property)]
    async fn loop_status(&self) -> String {
        "None".to_string()
    }

    #[zbus(property)]
    async fn rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    async fn metadata(&self) -> HashMap<&str, zbus::zvariant::Value> {
        info!("dbus metadata");
        if self.gstreamer.get_state() == GStreamerMessage::Playing {
            let track = self.playlisttabs.get_current_track();
            let length = 1_000_000 * track.length;
            let albumpath = track.albumpath.unwrap_or_default();
            HashMap::from([
                ("xesam:trackid", "/track".into()),
                ("xesam:artist", track.artist.into()),
                ("xesam:album", track.album.into()),
                ("xesam:title", track.title.into()),
                ("mpris:length", length.into()),
                ("xesam:artUrl", albumpath.into()),
                ])
            }
            else {
                HashMap::new()
            }
    }

    #[zbus(property)]
    async fn volume(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    async fn position(&self) -> i64 {
        1_000_000 * self.gstreamer.get_elapsed().unwrap_or(0) as i64
    }

    #[zbus(property)]
    async fn minimum_rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    async fn maximum_rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    async fn can_go_next(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn can_go_previous(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn can_play(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn can_pause(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn can_seek(&self) -> bool {
        false
    }

    #[zbus(property)]
    async fn can_control(&self) -> bool {
        true
    }

    //methods
    async fn next(&self) -> zbus::fdo::Result<()> {
        self.gstreamer
            .do_gstreamer_action(GStreamerAction::Next);
        Ok(())
    }

    async fn previous(&self) -> zbus::fdo::Result<()> {
        self.gstreamer
            .do_gstreamer_action(GStreamerAction::Previous);
        Ok(())
    }

    async fn pause(&self) -> zbus::fdo::Result<()> {
        info!("dbus send pause");
        self.gstreamer
            .do_gstreamer_action(GStreamerAction::Pausing);
        self.gstreamer
            .sender
            .send(GStreamerMessage::Pausing)
            .expect("Error in sending msg to gui channel");
        Ok(())
    }

    async fn play(&self) -> zbus::fdo::Result<()> {
        info!("dbus send play");
        self.gstreamer
            .do_gstreamer_action(GStreamerAction::Playing);
        self.gstreamer
            .sender
            .send(GStreamerMessage::Playing)
            .expect("Error in sending msg to gui channel");
        Ok(())
    }

    async fn play_pause(&self) -> zbus::fdo::Result<()> {
        info!("dbus send playpause");
        if self.gstreamer.get_state() == GStreamerMessage::Pausing {
            self.gstreamer
                .do_gstreamer_action(GStreamerAction::Playing);
            self.gstreamer
                .sender
                .send(GStreamerMessage::Playing)
                .expect("Error in sending msg to gui channel");
        } else {
            self.gstreamer
                .do_gstreamer_action(GStreamerAction::Pausing);
            self.gstreamer
                .sender
                .send(GStreamerMessage::Pausing)
                .expect("Error in sending msg to gui channel");
        }
        Ok(())
    }

    async fn stop(&self) -> zbus::fdo::Result<()> {
        println!("dbus send pause");
        self.gstreamer
            .do_gstreamer_action(GStreamerAction::Pausing);
        self.gstreamer
            .sender
            .send(GStreamerMessage::Pausing)
            .expect("Error in sending msg to gui channel");
        Ok(())
    }

    async fn seek(&self, _position: i32) -> zbus::fdo::Result<()> {
        todo!("NYI");
        //Not Implemented
        Ok(())
    }

    async fn set_position(&self, _track_id: String, _position: i32) -> zbus::fdo::Result<()> {
        //Not Implemented
        todo!("NYI");
        Ok(())
    }

    async fn open_uri(&self, _s: String) -> zbus::fdo::Result<()> {
        todo!("NYI");
        Ok(())
    }
}

pub(crate) async fn main(
    gstreamer: Arc<GStreamer>,
    playlisttabs: PlaylistTabsPtr,
    bus: tokio::sync::broadcast::Receiver<GStreamerMessage>,
) -> Result<(), String> {
    info!("Starting dbus");
    let handler = BaseInterface {};
    let player_interface = PlayerInterface {
        gstreamer,
        playlisttabs,
    };
    let conn = ConnectionBuilder::session()
        .expect("Could not connect to session bus")
        .name("org.mpris.MediaPlayer2.Viola")
        .expect("Could not use name")
        .serve_at("/org/mpris/MediaPlayer2", handler)
        .expect("Could not serve dbus")
        .serve_at("/org/mpris/MediaPlayer2", player_interface)
        .expect("Could not serve player dbus")
        //.internal_executor(true)
        .build()
        .await
        .expect("Error in creating connection");

    //{
    //tokio::task::spawn(async move {
    std::future::pending::<()>().await;
    /*
    info!("doing signal");
    let iface_ref = conn
        .object_server()
        .interface::<_, PlayerInterface>("/org/mpris/MediaPlayer2")
        .await
        .unwrap();
    let iface = iface_ref.get_mut().await;
    let mut bus = bus;
    info!("we did all the setup and wait for the bus");
    loop {
        tokio::select! {
            Ok(val) = bus.recv() => {
                info!("Found msg val {:?}", val);
                match val {
                    GStreamerMessage::Playing
                    | GStreamerMessage::Pausing
                    | GStreamerMessage::Stopped
                    | GStreamerMessage::FileNotFound => {
                        iface
                            .metadata_changed(iface_ref.signal_context())
                            .await
                            .unwrap();
                        iface
                            .playback_status_changed(iface_ref.signal_context())
                            .await
                            .unwrap();
                    }
                    GStreamerMessage::ChangedDuration(_) => {
                        /*
                            iface
                            .position_changed(iface_ref.signal_context())
                            .await
                            .unwrap();
                        */
                    }
                    GStreamerMessage::Nop | GStreamerMessage::IncreasePlayCount(_) => {}
                }
        },
        };
    }
    */
    Ok(())
}
