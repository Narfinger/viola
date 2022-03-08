use crate::gstreamer::prelude::ObjectExt;
use gstreamer::prelude::{ElementExt, ElementExtManual, GstBinExtManual};
use gstreamer::traits::PadExt;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::loaded_playlist::{LoadedPlaylistExt, PlaylistControls};
//use crate::playlist_tabs::PlaylistControlsImmutable;
use crate::types::*;
use viola_common::{GStreamerAction, GStreamerMessage};

pub struct GStreamer {
    element: gstreamer::Element,
    current_playlist: PlaylistTabsPtr,
    /// Handles gstreamer changes to the gui
    sender: tokio::sync::watch::Sender<GStreamerMessage>,
    pool: DBPool,
    repeat_once: AtomicBool,
}

impl Drop for GStreamer {
    fn drop(&mut self) {
        self.element
            .set_state(gstreamer::State::Null)
            .expect("Error in setting gstreamer state: Null");
    }
}

pub fn new(
    current_playlist: PlaylistTabsPtr,
    pool: DBPool,
    msg_bus: tokio::sync::watch::Sender<GStreamerMessage>,
) -> Result<Arc<RwLock<GStreamer>>, String> {
    gstreamer::init().unwrap();
    let element = {
        let playbin = gstreamer::ElementFactory::make("playbin", None)
            .map_err(|e| format!("Cannot do gstreamer: {}", e))?;
        playbin.set_property("volume", 0.5);
        /* based on
               bin = gst_bin_new ("audio_sink_bin");
        gst_bin_add_many (GST_BIN (bin), equalizer, convert, sink, NULL);
        gst_element_link_many (equalizer, convert, sink, NULL);

        pad = gst_element_get_static_pad (equalizer, "sink");
        ghost_pad = gst_ghost_pad_new ("sink", pad);
        gst_pad_set_active (ghost_pad, TRUE);
        gst_element_add_pad (bin, ghost_pad);
        gst_object_unref (pad);

        /* Configure the equalizer */
        g_object_set (G_OBJECT (equalizer), "band1", (gdouble)-24.0, NULL);
        g_object_set (G_OBJECT (equalizer), "band2", (gdouble)-24.0, NULL);

        /* Set playbin's audio sink to be our sink bin */
        g_object_set (GST_OBJECT (pipeline), "audio-sink", bin, NULL);
        */
        let audioconvert1 = gstreamer::ElementFactory::make("audioconvert", Some("audioconvert1"))
            .expect("Error in convert");
        let rgvolume = gstreamer::ElementFactory::make("rgvolume", Some("rgvolume"))
            .expect("Error in rgvolume");
        let audioconvert2 = gstreamer::ElementFactory::make("audioconvert", Some("audioconvert2"))
            .expect("Error in convert2");
        let audioresample = gstreamer::ElementFactory::make("audioresample", Some("audioresample"))
            .expect("Errror in resample");
        let sink = gstreamer::ElementFactory::make("autoaudiosink", Some("autosink"))
            .expect("Errror in sink");
        let bin = gstreamer::Bin::new(Some("mybin"));
        bin.add_many(&[
            &audioconvert1,
            &rgvolume,
            &audioconvert2,
            &audioresample,
            &sink,
        ])
        .expect("Could not add");
        gstreamer::Element::link_many(&[
            &audioconvert1,
            &rgvolume,
            &audioconvert2,
            &audioresample,
            &sink,
        ])
        .expect("Could not link");
        let pad = audioconvert1.static_pad("sink").expect("Could not get pad");
        let ghost =
            gstreamer::GhostPad::with_target(Some("sink"), &pad).expect("Could not create ghost");
        ghost.set_active(true).expect("Could not set active");
        bin.add_pad(&ghost).expect("Could not add pad");
        playbin.set_property("audio-sink", bin);
        playbin
    };
    let bus = element.bus().unwrap();
    let res = Arc::new(RwLock::new(GStreamer {
        element,
        current_playlist,
        sender: msg_bus,
        pool,
        repeat_once: AtomicBool::new(false),
    }));

    let resc = res.clone();
    std::thread::spawn(move || {
        use gstreamer::MessageView;
        for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
            match msg.view() {
                MessageView::Eos(..) => {
                    info!("We found an eos on the bus!");
                    resc.write().gstreamer_handle_eos();
                    info!("returned from eos handling");
                }
                MessageView::Error(err) => println!("Error {:?}", err),
                MessageView::StateChanged(state_changed) => {
                    warn!("Message bus has state change: {:?}", state_changed)
                }
                MessageView::Tag(_) => {
                    warn!("Found tag msg")
                }
                m => (warn!("Found message {:?}", m)),
            }
        }
    });

    //let resc = res.clone();
    //glin::timeout_add(250, move || resc.gstreamer_update_gui());
    Ok(res)
}

pub trait GStreamerExt {
    fn do_gstreamer_action(&mut self, _: GStreamerAction);
    fn gstreamer_update_gui(&self) -> glib::Continue;
    fn gstreamer_handle_eos(&mut self);
    fn get_state(&self) -> GStreamerMessage;
    fn get_elapsed(&self) -> Option<u64>;
}

impl GStreamerExt for GStreamer {
    fn do_gstreamer_action(&mut self, action: GStreamerAction) {
        info!("Gstreamer action {:?}", action);

        //everytime we call return, we do not want to send the message we got to the gui, as it will be done in a subcall we have done
        match action {
            GStreamerAction::Next => {
                self.repeat_once.store(false, Ordering::SeqCst);
                if let Some(i) = self.current_playlist.next_or_eol() {
                    self.do_gstreamer_action(GStreamerAction::Play(i));
                } else {
                    self.do_gstreamer_action(GStreamerAction::Stop);
                }
                return;
            }
            GStreamerAction::Playing => {
                if self.get_state() == GStreamerMessage::Pausing {
                    self.element
                        .set_state(gstreamer::State::Playing)
                        .expect("Error in setting gstreamer state");
                } else {
                    self.do_gstreamer_action(GStreamerAction::Play(
                        self.current_playlist.current_position(),
                    ));
                    return;
                }
            }
            GStreamerAction::Pausing => {
                //let is_playing = GStreamerMessage::Playing == self.get_state();
                //if is_playing {
                self.element
                    .set_state(gstreamer::State::Paused)
                    .expect("Error setting gstreamer state");
                //} else {
                //    self.do_gstreamer_action(GStreamerAction::Play(
                //        self.current_playlist.current_position(),
                //    ));
                //    return;
                //}
            }
            GStreamerAction::Previous => {
                self.repeat_once.store(false, Ordering::SeqCst);
                if let Some(i) = self.current_playlist.previous() {
                    self.do_gstreamer_action(GStreamerAction::Play(i));
                } else {
                    self.do_gstreamer_action(GStreamerAction::Stop);
                }
                return;
            }
            GStreamerAction::Stop => {
                self.element
                    .set_state(gstreamer::State::Ready)
                    .expect("Error setting gstreamer state");
            }
            GStreamerAction::Play(i) => {
                self.current_playlist.set(i);
                if let Some(uri) = self.current_playlist.get_current_uri() {
                    if !self
                        .current_playlist
                        .get_current_path()
                        .expect("URI Error")
                        .exists()
                    {
                        panic!("The file we want to play does not exist");
                    }
                    info!(
                        "Playing uri: {:?}",
                        self.current_playlist.get_current_path()
                    );

                    //looking at gstreamer state transition diagram
                    //https://gstreamer.freedesktop.org/documentation/additional/design/states.html?gi-language=c
                    if self.get_state() == GStreamerMessage::Playing {
                        self.element
                            .set_state(gstreamer::State::Paused)
                            .expect("Error setting gstreamer state");
                    }
                    self.element
                        .set_state(gstreamer::State::Ready)
                        .expect("Errorr in setting gstreamer state");

                    self.element.set_property("uri", &uri);
                    self.element
                        .set_state(gstreamer::State::Playing)
                        .expect("Error setting gstreamer state");
                    info!("gstreamer state: {:?}", self.get_state());
                    info!(
                        "gstreamer real state: {:?}",
                        self.element.state(gstreamer::ClockTime::SECOND)
                    );
                } else {
                    info!("Stopping gstreamer because we did not find next track");
                    self.current_playlist.set(0);
                    self.do_gstreamer_action(GStreamerAction::Stop);
                    return;
                }
            }
            GStreamerAction::Seek(pos) => {
                let time = gstreamer::ClockTime::from_seconds(pos);
                self.element
                    .seek_simple(gstreamer::SeekFlags::FLUSH, time)
                    .expect("Error in seeking");
            }
            GStreamerAction::RepeatOnce => {
                self.repeat_once.store(true, Ordering::SeqCst);
            }
        }
        if let Err(e) = self.sender.send(action.into()) {
            warn!("Could not broadcast, ignoring: {}", e);
        }
    }

    /// poll the message bus and on eos start new
    fn gstreamer_update_gui(&self) -> glib::Continue {
        glib::Continue(true)
    }

    fn gstreamer_handle_eos(&mut self) {
        use crate::db::UpdatePlayCount;
        info!("Handling EOS");

        self.current_playlist.update_current_playcount();

        //we want to separately update the playcount in the database because we never want to miss if something was played
        let mut old_track = self.current_playlist.get_current_track();
        let pc = self.pool.clone();
        std::thread::spawn(move || {
            old_track.update_playcount(pc);
        });
        self.sender
            .send(GStreamerMessage::IncreasePlayCount(
                self.current_playlist.current_position(),
            ))
            .expect("Error in sending gstreamer message");

        let res = if self.repeat_once.load(Ordering::Acquire) {
            info!("we are repeat playing");
            self.repeat_once.store(false, Ordering::SeqCst);
            Some(self.current_playlist.current_position())
        } else {
            self.current_playlist.next_or_eol()
        };
        if let Some(i) = res {
            self.do_gstreamer_action(GStreamerAction::Play(i));
        } else {
            self.do_gstreamer_action(GStreamerAction::Stop);
        }
    }

    fn get_state(&self) -> viola_common::GStreamerMessage {
        match self.element.state(gstreamer::ClockTime::SECOND).1 {
            gstreamer::State::VoidPending | gstreamer::State::Null | gstreamer::State::Ready => {
                GStreamerMessage::Stopped
            }
            gstreamer::State::Paused => GStreamerMessage::Pausing,
            gstreamer::State::Playing => GStreamerMessage::Playing,
            _ => GStreamerMessage::Stopped,
        }
    }

    fn get_elapsed(&self) -> Option<u64> {
        let cltime_opt: Option<gstreamer::ClockTime> = self.element.query_position();
        cltime_opt.map(gstreamer::ClockTime::seconds)
    }
}
