use crate::glib::ObjectExt;
use gstreamer;
use gstreamer::{ElementExt, ElementExtManual, GstObjectExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;

use crate::loaded_playlist::{LoadedPlaylistExt, PlaylistControls};
use crate::playlist_tabs::{PlaylistTabs, PlaylistTabsExt};
//use crate::playlist_tabs::PlaylistControlsImmutable;
use crate::types::*;

pub struct GStreamer {
    pipeline: gstreamer::Element,
    current_playlist: PlaylistTabsPtr,
    /// Handles gstreamer changes to the gui
    sender: SyncSender<GStreamerMessage>,
    pool: DBPool,
    repeat_once: AtomicBool,
}

impl Drop for GStreamer {
    fn drop(&mut self) {
        self.pipeline
            .set_state(gstreamer::State::Null)
            .expect("Error in setting gstreamer state: Null");
    }
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub enum GStreamerMessage {
    Pausing,
    Stopped,
    Playing,
    ChangedDuration((u64, u64)), //in seconds
}

/// Tells the GuiPtr and the gstreamer what action is performed. Splits the GuiPtr and the backend a tiny bit
#[derive(Debug, Eq, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "c")]
pub enum GStreamerAction {
    Next,
    Playing,
    Pausing,
    Previous,
    Stop,
    /// This means we selected one specific track
    Play(usize),
    Seek(u64),
    RepeatOnce, // Repeat the current playing track after it finishes
}

impl From<GStreamerAction> for GStreamerMessage {
    fn from(action: GStreamerAction) -> Self {
        match action {
            GStreamerAction::Pausing => GStreamerMessage::Pausing,
            GStreamerAction::Stop => GStreamerMessage::Stopped,
            _ => GStreamerMessage::Playing,
        }
    }
}

impl From<gstreamer::State> for GStreamerMessage {
    fn from(state: gstreamer::State) -> GStreamerMessage {
        match state {
            gstreamer::State::VoidPending => GStreamerMessage::Stopped,
            gstreamer::State::Null => GStreamerMessage::Stopped,
            gstreamer::State::Ready => GStreamerMessage::Stopped,
            gstreamer::State::Paused => GStreamerMessage::Pausing,
            gstreamer::State::Playing => GStreamerMessage::Playing,
            _ => GStreamerMessage::Stopped,
        }
    }
}

pub fn new(
    current_playlist: PlaylistTabsPtr,
    pool: DBPool,
) -> Result<(Arc<GStreamer>, Receiver<GStreamerMessage>), String> {
    gstreamer::init().unwrap();
    let pipeline = gstreamer::ElementFactory::make("playbin", None)
        .map_err(|e| format!("Cannot do gstreamer: {}", e))?;

    let (tx, rx) = sync_channel::<GStreamerMessage>(1);

    //old method for eos
    let (eos_tx, eos_rx) = sync_channel::<()>(1);
    //pipeline
    //    .connect("about-to-finish", true, move |_| {
    //        warn!("received signal to go to next track");
    //eos_tx.send(()).expect("Error in sending eos to own bus");
    //        None
    //    })
    //    .expect("Could not connect to about-to-finish signal");

    let bus = pipeline.get_bus().unwrap();
    let res = Arc::new(GStreamer {
        pipeline,
        current_playlist,
        sender: tx,
        pool,
        repeat_once: AtomicBool::new(false),
    });

    let resc = res.clone();
    std::thread::spawn(move || {
        use gstreamer::MessageView;
        for msg in bus.iter_timed(gstreamer::CLOCK_TIME_NONE) {
            match msg.view() {
                MessageView::Eos(..) => {
                    warn!("We found an eos on the bus!");
                    resc.gstreamer_handle_eos()
                }
                MessageView::Error(err) => println!(
                    "Error from {:?}: {} ({:?})",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error(),
                    err.get_debug()
                ),
                MessageView::StateChanged(state_changed) => {
                    warn!("Message bus has state change: {:?}", state_changed)
                }
                m => (warn!("Found message {:?}", m)),
            }
        }
    });

    //let resc = res.clone();
    //glin::timeout_add(250, move || resc.gstreamer_update_gui());
    Ok((res, rx))
}

pub trait GStreamerExt {
    fn do_gstreamer_action(&self, _: GStreamerAction);
    fn gstreamer_update_gui(&self) -> glib::Continue;
    fn gstreamer_handle_eos(&self);
    fn get_state(&self) -> GStreamerMessage;
}

impl GStreamerExt for GStreamer {
    fn do_gstreamer_action(&self, action: GStreamerAction) {
        info!("Gstreamer action {:?}", action);

        //everytime we call return, we do not want to send the message we got to the gui, as it will be done in a subcall we have done
        match action {
            GStreamerAction::Next => {
                if let Some(i) = self.current_playlist.current().unwrap().next_or_eol() {
                    self.do_gstreamer_action(GStreamerAction::Play(i));
                } else {
                    self.do_gstreamer_action(GStreamerAction::Stop);
                }
                return;
            }
            GStreamerAction::Playing => {
                if self.get_state() == GStreamerMessage::Pausing {
                    self.pipeline
                        .set_state(gstreamer::State::Playing)
                        .expect("Error in setting gstreamer state");
                } else {
                    self.do_gstreamer_action(GStreamerAction::Play(
                        self.current_playlist.current().unwrap().current_position(),
                    ));
                    return;
                }
            }
            GStreamerAction::Pausing => {
                let is_playing = gstreamer::State::Playing
                    == self.pipeline.get_state(gstreamer::ClockTime(Some(5))).1;
                if is_playing {
                    self.pipeline
                        .set_state(gstreamer::State::Paused)
                        .expect("Error setting gstreamer state");
                } else {
                    self.do_gstreamer_action(GStreamerAction::Play(
                        self.current_playlist.current().unwrap().current_position(),
                    ));
                    return;
                }
            }
            GStreamerAction::Previous => {
                if let Some(i) = self.current_playlist.current().unwrap().previous() {
                    self.do_gstreamer_action(GStreamerAction::Play(i));
                } else {
                    self.do_gstreamer_action(GStreamerAction::Stop);
                }
                return;
            }
            GStreamerAction::Stop => {
                self.pipeline
                    .set_state(gstreamer::State::Ready)
                    .expect("Error setting gstreamer state");
            }
            GStreamerAction::Play(i) => {
                self.current_playlist.current().unwrap().set(i);
                let uri = self.current_playlist.current().unwrap().get_current_uri();
                if !self
                    .current_playlist
                    .current()
                    .unwrap()
                    .get_current_path()
                    .exists()
                {
                    panic!("The file we want to play does not exist");
                }
                println!(
                    "Playing uri: {:?}",
                    self.current_playlist.current().unwrap().get_current_path()
                );
                self.pipeline
                    .set_state(gstreamer::State::Ready)
                    .expect("Error in setting gstreamer state");
                self.pipeline
                    .set_property("uri", &uri)
                    .expect("Error setting new gstreamer url");
                self.pipeline
                    .set_state(gstreamer::State::Playing)
                    .expect("Error setting gstreamer state");
                println!("gstreamer state: {:?}", self.get_state());
            }
            GStreamerAction::Seek(u64) => {
                panic!("NYI");
            }
            GStreamerAction::RepeatOnce => {
                self.repeat_once.store(true, Ordering::Relaxed);
            }
        }
        self.sender.send(action.into()).expect("Error in sending");
    }

    /// poll the message bus and on eos start new
    fn gstreamer_update_gui(&self) -> glib::Continue {
        //update gui for running time
        let cltime_opt: Option<gstreamer::ClockTime> = self.pipeline.query_position();
        let cltotal_opt: Option<gstreamer::ClockTime> = self.pipeline.query_duration();
        if let Some(cltime) = cltime_opt {
            if let Some(cl) = cltotal_opt {
                let total = cl.seconds().unwrap_or(0);
                //warn!("total: {}", total);
                self.sender
                    .send(GStreamerMessage::ChangedDuration((
                        cltime.seconds().unwrap_or(0),
                        total,
                    )))
                    .expect("Error in gstreamer sending message to gui");
            }
        }
        glib::Continue(true)
    }

    fn gstreamer_handle_eos(&self) {
        use crate::db::UpdatePlayCount;
        info!("Handling EOS");

        let mut old_track = self.current_playlist.current().unwrap().get_current_track();
        let pc = self.pool.clone();
        std::thread::spawn(move || {
            old_track.update_playcount(pc);
        });

        let res = if self.repeat_once.load(Ordering::Relaxed) {
            info!("we are repeat playing");
            self.repeat_once.store(false, Ordering::Relaxed);
            Some(self.current_playlist.current().unwrap().current_position())
        } else {
            self.current_playlist.current().unwrap().next_or_eol()
        };
        if let Some(i) = res {
            self.do_gstreamer_action(GStreamerAction::Play(i));
        } else {
            self.do_gstreamer_action(GStreamerAction::Stop);
        }
    }

    fn get_state(&self) -> GStreamerMessage {
        self.pipeline
            .get_state(gstreamer::ClockTime(Some(5)))
            .1
            .into()
    }
}
