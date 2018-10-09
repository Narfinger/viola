use gstreamer;
use gstreamer::{ElementExt, ElementExtManual};
use gtk;
use gtk::ObjectExt;
use std::rc::Rc;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender};

use loaded_playlist::PlaylistControls;
use playlist_tabs::PlaylistControlsImmutable;
use types::*;

pub struct GStreamer {
    pipeline: gstreamer::Element,
    current_playlist: PlaylistTabsPtr,
    /// Handles gstreamer changes to the gui
    sender: Sender<GStreamerMessage>,
    /// Handles if we get the almost finished signal
    finish_reicv: Receiver<()>,
    pool: DBPool,
}

impl Drop for GStreamer {
    fn drop(&mut self) {
        self.pipeline
            .set_state(gstreamer::State::Null)
            .into_result()
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
    current_playlist: PlaylistTabsPtr, pool: DBPool,
) -> Result<(Rc<GStreamer>, Receiver<GStreamerMessage>), String> {
    gstreamer::init().unwrap();
    let pipeline =
        gstreamer::parse_launch("playbin").map_err(|_| String::from("Cannot do gstreamer"))?;

    let (tx, rx) = channel::<GStreamerMessage>();
    let (finish_send, finish_reicv) = sync_channel::<()>(1);

    pipeline
        .connect("about-to-finish", true, move |_| {
            finish_send
                .send(())
                .expect("Error in sending almost_finished signal");
            None
        }).expect("Error in connecting");

    let res = Rc::new(GStreamer {
        pipeline,
        current_playlist,
        sender: tx,
        finish_reicv,
        pool,
    });

    //panic!("this would leave us with a circ reference");

    let resc = res.clone();
    gtk::timeout_add(500, move || resc.gstreamer_message_handler());
    Ok((res, rx))
}

/// Tells the GuiPtr and the gstreamer what action is performed. Splits the GuiPtr and the backend a tiny bit
#[derive(Debug, Eq, PartialEq)]
pub enum GStreamerAction {
    Next,
    Playing,
    Pausing,
    Previous,
    /// This means we selected one specific track
    Play(i32),
    Seek(u64),
}

pub trait GStreamerExt {
    fn do_gstreamer_action(&self, &GStreamerAction);
    fn gstreamer_message_handler(&self) -> gtk::Continue;
}

impl GStreamerExt for GStreamer {
    fn do_gstreamer_action(&self, action: &GStreamerAction) {
        //we need to set the state to paused and ready

        error!("We do not implement seek yet");
        match *action {
            GStreamerAction::Seek(i) => {
                let t = gstreamer::ClockTime::from_seconds(i);
                self.pipeline.seek_simple(gstreamer::SeekFlags::NONE, t).expect("Could not seek");
            }
            _ => {}
        };

        match *action {
            GStreamerAction::Play(_) | GStreamerAction::Previous | GStreamerAction::Next => {
                if gstreamer::State::Playing
                    == self.pipeline.get_state(gstreamer::ClockTime(Some(1000))).1
                {
                    info!("Doing");
                    self.pipeline
                        .set_state(gstreamer::State::Paused)
                        .into_result()
                        .expect("Error in gstreamer state set, paused");
                    self.pipeline
                        .set_state(gstreamer::State::Ready)
                        .into_result()
                        .expect("Error in gstreamer state set, ready");
                }
            }
            _ => {}
        }
        //getting correct url or None
        let url = match *action {
            GStreamerAction::Playing => Some(self.current_playlist.borrow().get_current_uri()),
            GStreamerAction::Pausing => {
                if gstreamer::State::Playing
                    != self.pipeline.get_state(gstreamer::ClockTime(Some(1000))).1
                {
                    Some(self.current_playlist.borrow().get_current_uri())
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
            if !self.current_playlist.borrow().get_current_path().exists() {
                panic!("The file we want to play does not exist");
            }

            self.pipeline
                .set_property("uri", &u)
                .expect("Error setting new gstreamer url");
        }
        //which gstreamer action
        let gstreamer_action = if (*action == GStreamerAction::Pausing)
            & (gstreamer::State::Playing
                == self.pipeline.get_state(gstreamer::ClockTime(Some(1000))).1)
        {
            gstreamer::State::Paused
        } else {
            gstreamer::State::Playing
        };

        //sending to gui
        self.sender
            .send(gstreamer_action.into())
            .expect("Error in sending updated state");

        //sending to gstreamer
        if let Err(e) = self.pipeline.set_state(gstreamer_action).into_result() {
            if let Some(bus) = self.pipeline.get_bus() {
                while let Some(msg) = bus.pop() {
                    info!("we found messages on the bus {:?}", msg);
                }
            }
            panic!(
                "Error in setting gstreamer state playing, found the following error {:?}",
                e
            );
        }
    }

    /// poll the message bus and on eos start new
    fn gstreamer_message_handler(&self) -> gtk::Continue {
        //update gui for running time
        {
            let cltime_opt: Option<gstreamer::ClockTime> = self.pipeline.query_position();
            let cltotal_opt: Option<gstreamer::ClockTime> = self.pipeline.query_duration();
            if let Some(cltime) = cltime_opt {
                let total = cltotal_opt.unwrap().seconds().unwrap_or(0);
                warn!("total: {}", total);
                self.sender
                    .send(GStreamerMessage::ChangedDuration((cltime.seconds().unwrap_or(0), total)))
                    .expect("Error in gstreamer sending message to gui");
                
                }

            }

        if self.finish_reicv.try_recv().is_ok() {
            //println!("next is: {:?}", self.current_playlist.next_or_eol());
            //self.current_playlist.next_or_eol();
            let res = self.current_playlist.next_or_eol(&self.pool);
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
                        .into_result()
                        .expect("Error in changing gstreamer state to ready");
                    self.pipeline
                        .set_property("uri", &i)
                        .expect("Error setting new url for gstreamer");
                    self.pipeline
                        .set_state(gstreamer::State::Playing)
                        .into_result()
                        .expect("Error in changing gstreamer state to playing");
                    self.sender
                        .send(GStreamerMessage::Playing)
                        .expect("Error in gstreamer sending message to gui");
                }
            };
        }
        gtk::Continue(true)
    }
}
