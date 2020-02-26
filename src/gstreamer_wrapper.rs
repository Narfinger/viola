use rodio::Sink;
use std::file::{BufReader, File};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;

use crate::loaded_playlist::{LoadedPlaylistExt, PlaylistControls};
//use crate::playlist_tabs::PlaylistControlsImmutable;
use crate::types::*;

pub struct GStreamer {
    sink: rodio::Sink,
    current_playlist: LoadedPlaylistPtr,
    /// Handles gstreamer changes to the gui
    sender: SyncSender<GStreamerMessage>,
    pool: DBPool,
    repeat_once: AtomicBool,
}

impl Drop for GStreamer {
    fn drop(&mut self) {
        self.sink.stop();
    }
}

#[derive(Debug, Serialize)]
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
    /// This means we selected one specific track
    Play(usize),
    Seek(u64),
    RepeatOnce, // Repeat the current playing track after it finishes
}

impl From<GStreamerAction> for GStreamerMessage {
    fn from(action: GStreamerAction) -> Self {
        match action {
            GStreamerAction::Next => GStreamerMessage::Playing,
            GStreamerAction::Playing => GStreamerMessage::Playing,
            GStreamerAction::Pausing => GStreamerMessage::Pausing,
            GStreamerAction::Previous => GStreamerMessage::Playing,
            GStreamerAction::Play(i) => GStreamerMessage::Playing,
            GStreamerAction::Seek(i) => GStreamerMessage::Playing,
            GStreamerAction::RepeatOnce => GStreamerMessage::Playing,
        }
    }
}

pub fn new(
    current_playlist: LoadedPlaylistPtr,
    pool: DBPool,
) -> Result<(Arc<GStreamer>, Receiver<GStreamerMessage>), String> {
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);

    let (tx, rx) = sync_channel::<GStreamerMessage>(1);
    let res = Arc::new(GStreamer {
        sink,
        current_playlist,
        sender: tx,
        pool,
        repeat_once: AtomicBool::new(false),
    });

    {
        let resc = res.clone();
        std::thread::spawn(move || loop {
            resc.sink.sleep_until_end();
            resc.gstreamer_handle_eos();
        });
    }

    Ok((res, rx))
}

pub trait GStreamerExt {
    fn do_gstreamer_action(&self, _: GStreamerAction);
    fn gstreamer_update_gui(&self);
    fn gstreamer_handle_eos(&self);
    fn get_state(&self) -> GStreamerMessage;
}

impl GStreamerExt for GStreamer {
    fn do_gstreamer_action(&self, action: GStreamerAction) {
        info!("Gstreamer action {:?}", action);
        match action {
            GStreamerAction::Next => {
                let i = self.current_playlist.next_or_eol(&self.pool);
                if let Some(j) = i {
                    self.do_gstreamer_action(GStreamerAction::Play(j));
                } else {
                    self.sink.stop();
                }
                return;
            }
            GStreamerAction::Playing => {
                let i = self.current_playlist.previous();
                self.do_gstreamer_action(GStreamerAction::Play(i));
                return;
            }
            GStreamerAction::Pausing => {
                if self.sink.is_paused() {
                    self.do_gstreamer_action(GStreamerAction::Playing);
                    return;
                } else {
                    self.sink.pause();
                }
            }
            GStreamerAction::Previous => {
                let i = self.current_playlist.previous();
                if let Some(j) = i {
                    self.do_gstreamer_action(GStreamerAction::Play(i));
                    return;
                } else {
                    self.do_gstreamer_action(GStreamerAction::Pausing);
                }
            }
            GStreamerAction::Play(i) => {
                self.sink.stop();
                let path = self.current_playlist.set(i);
                let f = File::open(p);
                let source = rodio::Decoder::new(BufReader::new(f)).unwrap();
                self.sink.append(source);
            }
            GStreamerAction::Seek(i) => {}
            GStreamerAction::RepeatOnce => {
                self.repeat_once.store(true, Ordering::Relaxed);
            }
        }

        //sending to gstreamer
        println!("state we set to: {:?}", action);
        self.sender.send(action.into()).expect("Error in sending");
    }

    /// poll the message bus and on eos start new
    fn gstreamer_update_gui(&self) {
        panic!("NYI");
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

    fn get_state(&self) -> GStreamerMessage {
        self.pipeline
            .get_state(gstreamer::ClockTime(Some(5)))
            .1
            .into()
    }
}
