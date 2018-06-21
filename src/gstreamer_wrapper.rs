use gstreamer;
use gstreamer::ElementExt;
use gtk;
use gtk::ObjectExt;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::rc::Rc;

use playlist_tabs::PlaylistControlsImmutable;
use loaded_playlist::PlaylistControls;
use types::*;

pub struct GStreamer {
    pipeline: gstreamer::Element,
    current_playlist: PlaylistTabsPtr,
    sender: Sender<GStreamerMessage>,
}

impl Drop for GStreamer {
    fn drop(&mut self) {
        self.pipeline.set_state(gstreamer::State::Null).into_result().expect("Error in setting gstreamer state: Null");
    }
}

pub enum GStreamerMessage {
    Pausing,
    Stopped,
    Playing,
}

impl From<gstreamer::State> for GStreamerMessage {
    fn from(state: gstreamer::State) -> GStreamerMessage {
        match state {
            gstreamer::State::VoidPending => GStreamerMessage::Stopped,
            gstreamer::State::Null        => GStreamerMessage::Stopped,
            gstreamer::State::Ready       => GStreamerMessage::Stopped,
            gstreamer::State::Paused      => GStreamerMessage::Pausing,
            gstreamer::State::Playing     => GStreamerMessage::Playing,
            _                             => GStreamerMessage::Stopped,
        }
    }
}

pub fn new(current_playlist: PlaylistTabsPtr) -> Result<(Rc<GStreamer>, Receiver<GStreamerMessage>), String> {
    gstreamer::init().unwrap();
    let pipeline =
        gstreamer::parse_launch("playbin").map_err(|_| String::from("Cannot do gstreamer"))?;

    let (tx, rx) = channel::<GStreamerMessage>();
    let res = Rc::new(GStreamer { pipeline, current_playlist, sender: tx });

    let resc = res.clone();
    gtk::timeout_add(500, move || {
        resc.gstreamer_message_handler()
    });
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
}

pub trait GStreamerExt {
    fn do_gstreamer_action(&self, &GStreamerAction);
    fn gstreamer_message_handler(&self) -> gtk::Continue;
}

impl GStreamerExt for GStreamer {
    fn do_gstreamer_action(&self, action: &GStreamerAction) {
        //we need to set the state to paused and ready
        match *action {
            GStreamerAction::Play(_) | GStreamerAction::Previous | GStreamerAction::Next => {
                if gstreamer::State::Playing == self.pipeline.get_state(gstreamer::ClockTime(Some(1000))).1 {
                    self.pipeline.set_state(gstreamer::State::Paused)
                        .into_result()
                        .expect("Error in gstreamer state set, paused");
                    self.pipeline.set_state(gstreamer::State::Ready)
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
                if gstreamer::State::Playing != self.pipeline.get_state(gstreamer::ClockTime(Some(1000))).1 {
                    Some(self.current_playlist.borrow().get_current_uri())
                } else {
                    None
                }
            },
            GStreamerAction::Previous => Some(self.current_playlist.previous()),
            GStreamerAction::Next => Some(self.current_playlist.next()),
                GStreamerAction::Play(i) => Some(self.current_playlist.set(i)),
        };
        //setting the url
        if let Some(u) = url {
            self.pipeline.set_property("uri", &u).expect("Error setting new gstreamer url");
        }
        //which gstreamer action
        let gstreamer_action = if (*action == GStreamerAction::Pausing) & 
            (gstreamer::State::Playing == self.pipeline.get_state(gstreamer::ClockTime(Some(1000))).1) {
            gstreamer::State::Paused
            
        } else {
            gstreamer::State::Playing
        };

        //sending to gui
        self.sender.send(gstreamer_action.into()).expect("Error in sending updated state");
        
        //sending to gstreamer
        if let Err(e) = self.pipeline.set_state(gstreamer_action).into_result() {
                panic!("Error in setting gstreamer state playing, found the following error {:?}", e);
        }
    }

    /// poll the message bus and on eos start new
    fn gstreamer_message_handler(&self) -> gtk::Continue {
        let bus = self.pipeline.get_bus().unwrap();
        if let Some(msg) = bus.pop() {
            use gstreamer::MessageView;
            match msg.view() {
                MessageView::Error(err) => {
                    eprintln!("Error received {}", err.get_error());
                    eprintln!("Debugging information: {:?}", err.get_debug());
                }
                MessageView::StateChanged(state_changed) => {
                    println!(
                        "Pipeline state changed from {:?} to {:?}",
                        state_changed.get_old(),
                        state_changed.get_current()
                    );
                    //sender.send(GStreamerMessage::Playing).expect("Error in gstreamer sending message to gui");
                }
                MessageView::Eos(..) => {
                    let res = self.current_playlist.next_or_eol();
                    match res {
                        None => { 
                            self.sender.send(GStreamerMessage::Stopped).expect("Message Queue Error");
                            self.sender.send(GStreamerMessage::Stopped).expect("Error in gstreamer sending message to gui");
                            },
                        Some(i) => {
                            println!("Next should play");
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
                            self.sender.send(GStreamerMessage::Playing).expect("Error in gstreamer sending message to gui");
                        }
                    }
                    println!("Eos found");
                }
                _ => (),
            }
            

        }
        gtk::Continue(true)
    }
}
