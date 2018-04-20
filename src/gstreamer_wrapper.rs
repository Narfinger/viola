use gstreamer;
use gstreamer::ElementExt;
use gtk;
use gtk::ObjectExt;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::rc::Rc;

use gui::Gui;
use playlist;
use playlist::LoadedPlaylist;
use types::*;

pub struct GStreamer {
    pipeline: gstreamer::Element,
    current_playlist: CurrentPlaylist,
}

impl Drop for GStreamer {
    fn drop(&mut self) {
        self.pipeline.set_state(gstreamer::State::Null).into_result().expect("Error in setting gstreamer state: Null");
    }
}

pub enum GStreamerMessage {
    Stopped,
    Playing,
}

pub fn new(current_playlist: CurrentPlaylist) -> Result<(Rc<GStreamer>, Receiver<GStreamerMessage>), String> {
    gstreamer::init().unwrap();
    let pipeline =
        gstreamer::parse_launch("playbin").map_err(|_| String::from("Cannot do gstreamer"))?;

    let (tx, rx) = channel::<GStreamerMessage>();
    let res = Rc::new(GStreamer { pipeline: pipeline, current_playlist: current_playlist });

    let resc = res.clone();
    gtk::timeout_add(500, move || {
        resc.gstreamer_message_handler(tx.clone())
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
    fn gstreamer_message_handler(&self, Sender<GStreamerMessage>) -> gtk::Continue;
}

impl GStreamerExt for GStreamer {
    fn do_gstreamer_action(&self, action: &GStreamerAction) {
        let mut gui_update = PlayerStatus::Playing;
        let mut gstreamer_action = gstreamer::State::Playing;
        {
            //releaingx the locks later
            //let mut pl = current_playlist.write().unwrap();
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

            let mut pl = self.current_playlist.write().unwrap();
            match *action {
                GStreamerAction::Playing => {
                    self.pipeline.set_property("uri", &playlist::get_current_uri(&pl))
                        .expect("Error setting new gstreamer url");
                }
                GStreamerAction::Pausing => {
                    if gstreamer::State::Playing == self.pipeline.get_state(gstreamer::ClockTime(Some(1000))).1 {
                        gstreamer_action = gstreamer::State::Paused;
                        gui_update = PlayerStatus::Paused;
                    }
                }
                GStreamerAction::Previous => {
                    (*pl).current_position = ((*pl).current_position - 1) % (*pl).items.len() as i32;
                    self.pipeline.set_property("uri", &playlist::get_current_uri(&pl))
                        .expect("Error in changing url");
                }
                GStreamerAction::Next => {
                    (*pl).current_position = ((*pl).current_position + 1) % (*pl).items.len() as i32;
                    self.pipeline.set_property("uri", &playlist::get_current_uri(&pl))
                        .expect("Error in changing url");
                }
                GStreamerAction::Play(i) => {
                    (*pl).current_position = i;
                    self.pipeline.set_property("uri", &playlist::get_current_uri(&pl))
                        .expect("Error in chaning url");
                }
            }
            self.pipeline.set_state(gstreamer_action)
                .into_result()
                .expect("Error in setting gstreamer state playing");
        } //locks releaed
    }   
    /// poll the message bus and on eos start new
    fn gstreamer_message_handler(&self, sender: Sender<GStreamerMessage>) -> gtk::Continue {
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
                    //if state_changed.get_current() == gstreamer::State::Playing {
                    //    update_GuiPtr(&pipeline, &current_playlist, &builder);
                    //}
                }
                MessageView::Eos(..) => {
                    let mut p = self.current_playlist.write().unwrap();
                    (*p).current_position += 1;
                    if (*p).current_position >= (*p).items.len() as i32 {
                        (*p).current_position = 0;
                        sender.send(GStreamerMessage::Playing);

                    } else {
                        println!("Next should play");
                        self.pipeline
                            .set_state(gstreamer::State::Ready)
                            .into_result()
                            .expect("Error in changing gstreamer state to ready");
                        self.pipeline
                            .set_property("uri", &playlist::get_current_uri(&p))
                            .expect("Error setting new url for gstreamer");
                        self.pipeline
                            .set_state(gstreamer::State::Playing)
                            .into_result()
                            .expect("Error in changing gstreamer state to playing");
                        println!(
                            "Next one now playing is: {}",
                            &playlist::get_current_uri(&p)
                        );
                        
                        sender.send(GStreamerMessage::Stopped);
                    }
                    println!("Eos found");
                }
                _ => (),
            }
        }
        gtk::Continue(true)
    }
}
