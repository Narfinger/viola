use gstreamer;
use gstreamer::{ElementExt, ElementExtManual};
use gtk;
use gtk::ObjectExt;
use gstreamer_player;
use crate::gtk::Cast;

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver, Sender};


use crate::loaded_playlist::PlaylistControls;
use crate::playlist_tabs::PlaylistControlsImmutable;
use crate::types::*;

pub struct GStreamer {
    //pipeline: gstreamer::Element,
    player: gstreamer_player::Player,
    current_playlist: PlaylistTabsPtr,
    /// Handles gstreamer changes to the gui
    sender: Sender<GStreamerMessage>,
    /// Handles if we get the almost finished signal
    finish_bool: Arc<AtomicBool>,
    pool: DBPool,
    repeat_once: Cell<bool>,
}

/*
impl Drop for GStreamer {
    fn drop(&mut self) {
        self.pipeline
            .set_state(gstreamer::State::Null)
            .expect("Error in setting gstreamer state: Null");
    }
}*/

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
    current_playlist: PlaylistTabsPtr,
    pool: DBPool,
) -> Result<(Rc<GStreamer>, Receiver<GStreamerMessage>), String> {
    gstreamer::init().unwrap();

    let dispatcher = gstreamer_player::PlayerGMainContextSignalDispatcher::new(None);
    let player = gstreamer_player::Player::new(
        None,
        Some(&dispatcher.upcast::<gstreamer_player::PlayerSignalDispatcher>()),
    );

    //let pipeline =
    //    gstreamer::parse_launch("playbin").map_err(|_| String::from("Cannot do gstreamer"))?;


    let (tx, rx) = channel::<GStreamerMessage>();
    let finish_bool = Arc::new(AtomicBool::new(false));

    /*
    {
        let fc = finish_bool.clone();
        pipeline
            .connect("about-to-finish", true, move |_| {
                //this signal seems to be called multiple times
                fc.store(true, std::sync::atomic::Ordering::Relaxed);
                None
            })
            .expect("Error in connecting");
    }*/

    let res = Rc::new(GStreamer {
        player,
        current_playlist,
        sender: tx,
        finish_bool,
        pool,
        repeat_once: Cell::new(false),
    });


    player.connect_end_of_stream(|player| {
        res.end_of_stream();
    });
    //panic!("this would leave us with a circ reference");

    let resc = res.clone();
    gtk::timeout_add(250, move || resc.gstreamer_message_handler());
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
    RepeatOnce, // Repeat the current playing track after it finishes
}

pub trait GStreamerExt {
    fn do_gstreamer_action(&self, _: &GStreamerAction);
    fn gstreamer_message_handler(&self) -> gtk::Continue;
    fn end_of_stream(&self);
}

impl GStreamerExt for GStreamer {
    fn do_gstreamer_action(&self, action: &GStreamerAction) {
        info!("Gstreamer action {:?}", action);
        if *action == GStreamerAction::RepeatOnce {
            self.repeat_once.set(true);
            return;
        }

        //getting correct url or None
        let url = match *action {
            GStreamerAction::RepeatOnce => None, //this is captured above but matches need to be complete
            GStreamerAction::Playing => Some(self.current_playlist.borrow().get_current_uri()),
            GStreamerAction::Pausing => {
                if gstreamer::State::Playing
                    != self.player.get_pipeline().get_state(gstreamer::ClockTime(Some(5))).1
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

        match *action {
           GStreamerAction::Previous | GStreamerAction::Next => {
                if gstreamer::State::Playing
                    == self.player.get_pipeline().get_state(gstreamer::ClockTime(Some(5))).1
                {
                    info!("Doing");
                    self.player.stop();
                }
            }
            _ => {}
        }

        //setting the url
        if let Some(u) = url {
            if !self.current_playlist.borrow().get_current_path().exists() {
                panic!("The file we want to play does not exist");
            }
            self.player.set_uri(&u);
        }

        //switch gstreamer play/pause
        let gstreamer_action = if (*action == GStreamerAction::Pausing)
            & (gstreamer::State::Playing
                == self.player.get_pipeline().get_state(gstreamer::ClockTime(Some(5))).1)
        {
            self.player.pause();
            gstreamer::State::Paused
        } else {
            self.player.play();
            gstreamer::State::Playing
        };

        //sending to gui
        self.sender
            .send(gstreamer_action.into())
            .expect("Error in sending updated state");
    }

    /// poll the message bus and on eos start new
    fn gstreamer_message_handler(&self) -> gtk::Continue {
        //update gui for running time
        {
            let cltime  = self.player.get_position();
            let cltotal = self.player.get_duration();
            let total = cltime.seconds().unwrap_or(0);
            self.sender
                .send(GStreamerMessage::ChangedDuration((
                    cltime.seconds().unwrap_or(0),
                    total,
                )))
                .expect("Error in gstreamer sending message to gui");
        }
        if self.finish_bool.load(std::sync::atomic::Ordering::Relaxed) {
            let res = if self.repeat_once.take() {
                info!("we are repeat playing");
                self.repeat_once.set(false);
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
                    self.player.set_uri(&i);
                }
            };
            self.finish_bool
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }
        gtk::Continue(true)
    }

    fn end_of_stream(&self) {
        if let Some(s) = self.current_playlist.next_or_eol(&self.pool) {
            self.player.set_uri(&s);
            self.player.play();
            self.sender
                .send(GStreamerMessage::Playing)
                .expect("Error in sending updated state");
        } else {
            self.player.stop();
             self.sender
                .send(GStreamerMessage::Stopped)
                .expect("Error in sending updated state");
        }
}
