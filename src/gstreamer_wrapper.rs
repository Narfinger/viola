use gstreamer;
use gstreamer::{ElementExt, ElementExtManual};
use gtk;
use gtk::ObjectExt;
use gtk::ToValue;

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::sync::Arc;

use crate::loaded_playlist::PlaylistControls;
//use crate::playlist_tabs::PlaylistControlsImmutable;
use crate::types::*;

pub struct GStreamer {
    pipeline: gstreamer::Element,
    current_playlist: LoadedPlaylistPtr,
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

pub enum GStreamerMessage {
    Pausing,
    Stopped,
    Playing,
    ChangedDuration((u64, u64)), //in seconds
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
    current_playlist: LoadedPlaylistPtr,
    pool: DBPool,
) -> Result<(Arc<GStreamer>, Receiver<GStreamerMessage>), String> {
    gstreamer::init().unwrap();
    let pipeline =
        gstreamer::parse_launch("playbin").map_err(|e| format!("Cannot do gstreamer: {}", e))?;

    let (tx, rx) = sync_channel::<GStreamerMessage>(1);

    //old method for eos
    let (eos_tx, eos_rx) = sync_channel::<()>(1);
    pipeline
        .connect("about-to-finish", true, move |_| {
            warn!("received signal to go to next track");
            eos_tx.send(()).expect("Error in sending eos to own bus");
            None
        })
        .expect("Could not connect to about-to-finish signal");

    let res = Arc::new(GStreamer {
        pipeline,
        current_playlist,
        sender: tx,
        pool,
        repeat_once: AtomicBool::new(false),
    });

    let resc = res.clone();
    glib::timeout_add(50, move || {
        if eos_rx.try_recv().is_ok() {
            info!("we found eos");
            resc.gstreamer_handle_eos();
        }
        gtk::Continue(true)
    });

    //let resc = res.clone();
    //glin::timeout_add(250, move || resc.gstreamer_update_gui());
    Ok((res, rx))
}

/// Tells the GuiPtr and the gstreamer what action is performed. Splits the GuiPtr and the backend a tiny bit
#[derive(Debug, Eq, Serialize, Deserialize, PartialEq)]
#[serde(tag = "t", content = "c")]
pub enum GStreamerAction {
    Next,
    Playing,
    Pausing,
    Previous,
    /// This means we selected one specific track
    Play(i32),
    Seek(u64),
    RepeatOnce, // Repeat the current playing track after it finishes
}

pub trait GStreamerExt {
    fn do_gstreamer_action(&self, _: &GStreamerAction);
    fn gstreamer_update_gui(&self) -> gtk::Continue;
    fn gstreamer_handle_eos(&self);
}

impl GStreamerExt for GStreamer {
    fn do_gstreamer_action(&self, action: &GStreamerAction) {
        info!("Gstreamer action {:?}", action);
        if *action == GStreamerAction::RepeatOnce {
            self.repeat_once.store(true, Ordering::Relaxed);
            return;
        }

        //we need to set the state to paused and ready
        match *action {
            GStreamerAction::Seek(i) => {
                let t = gstreamer::ClockTime::from_seconds(i);
                self.pipeline
                    .seek_simple(gstreamer::SeekFlags::NONE, t)
                    .expect("Could not seek");
            }
            _ => {}
        };

        match *action {
            GStreamerAction::Play(_) | GStreamerAction::Previous | GStreamerAction::Next => {
                if gstreamer::State::Playing
                    == self.pipeline.get_state(gstreamer::ClockTime(Some(5))).1
                {
                    info!("Doing (setting to paused and ready");
                    self.pipeline
                        .set_state(gstreamer::State::Paused)
                        .expect("Error in gstreamer state set, paused");
                    self.pipeline
                        .set_state(gstreamer::State::Ready)
                        .expect("Error in gstreamer state set, ready");
                }
            }
            _ => {}
        }
        //getting correct url or None
        let url = match *action {
            GStreamerAction::RepeatOnce => None, //this is captured above but matches need to be complete
            GStreamerAction::Playing => Some(self.current_playlist.get_current_uri()),
            GStreamerAction::Pausing => {
                if gstreamer::State::Playing
                    != self.pipeline.get_state(gstreamer::ClockTime(Some(5))).1
                {
                    Some(self.current_playlist.get_current_uri())
                } else {
                    None
                }
            }
            GStreamerAction::Previous => Some(self.current_playlist.previous()),
            GStreamerAction::Next => Some(self.current_playlist.next()),
            GStreamerAction::Play(i) => Some(self.current_playlist.set(i)),
            GStreamerAction::Seek(_) => None,
        };
        //setting the url
        if let Some(u) = url {
            if !self.current_playlist.get_current_path().exists() {
                panic!("The file we want to play does not exist");
            }

            self.pipeline
                .set_property("uri", &u)
                .expect("Error setting new gstreamer url");
        }
        //which gstreamer action
        let gstreamer_action = if (*action == GStreamerAction::Pausing)
            & (gstreamer::State::Playing
                == self.pipeline.get_state(gstreamer::ClockTime(Some(5))).1)
        {
            println!("Obviously, we do not reach here");
            gstreamer::State::Paused
        } else {
            gstreamer::State::Playing
        };

        //sending to gstreamer
        println!("state we set to: {:?}", gstreamer_action);
        self.pipeline
            .set_state(gstreamer_action)
            .expect("Error in sending to gstreamer");

        self.sender
            .send(gstreamer_action.into())
            .expect("Error in sending");
    }

    /// poll the message bus and on eos start new
    fn gstreamer_update_gui(&self) -> gtk::Continue {
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
        gtk::Continue(true)
    }

    fn gstreamer_handle_eos(&self) {
        info!("Handling EOS");
        let res = if self.repeat_once.load(Ordering::Relaxed) {
            info!("we are repeat playing");
            self.repeat_once.store(false, Ordering::Relaxed);
            Some(self.current_playlist.get_current_uri())
        } else {
            self.current_playlist.next_or_eol(&self.pool)
        };
        match res {
            None => {
                self.sender
                    .send(GStreamerMessage::Stopped)
                    .expect("Message Queue Error");
                self.sender
                    .send(GStreamerMessage::Stopped)
                    .expect("Error in gstreamer sending message to gui");
            }
            Some(i) => {
                info!("Next should play {:?}", &i);
                self.pipeline
                    .set_state(gstreamer::State::Ready)
                    .expect("Error in changing gstreamer state to ready");
                self.pipeline
                    .set_property("uri", &i)
                    .expect("Error setting new url for gstreamer");
                self.pipeline
                    .set_state(gstreamer::State::Playing)
                    .expect("Error in changing gstreamer state to playing");
                self.sender
                    .send(GStreamerMessage::Playing)
                    .expect("Error in gstreamer sending message to gui");
            }
        };
    }
}
