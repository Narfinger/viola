pub mod playlist;

#[macro_use] extern crate error_chain;
extern crate gtk;
extern crate gstreamer;
extern crate rodio;
extern crate taglib;
extern crate walkdir;

use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::rc::Rc;
use gstreamer::ElementExt;

use gtk::prelude::*;
use gtk::{Button, ListBox, Layout, Label, Grid, Orientation, PositionType, ScrolledWindow, Window, WindowType};
use walkdir::WalkDir;

error_chain! {
    foreign_links {
        GTK(gtk::Error);
    }
}

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

type CurrentPlaylist = Arc<Mutex<playlist::Playlist>>;
type Pipeline = Arc<Mutex<gstreamer::Element>>; 

/// poll the message bus and on eos start new
fn gstreamer_message_handler(pipeline: Pipeline, current_playlist: CurrentPlaylist) {
    let bus = {
        pipeline.lock().unwrap().get_bus().unwrap()
    };
    while let Some(msg) = bus.timed_pop(gstreamer::CLOCK_TIME_NONE) {
        use gstreamer::MessageView;
        match msg.view() {
            MessageView::Error(err) => {
                eprintln!("Error received {}", err.get_error());
                eprintln!("Debugging information: {:?}", err.get_debug());
            }
            MessageView::StateChanged(state_changed) => {
            println!("Pipeline state changed from {:?} to {:?}",
                        state_changed.get_old(),
                        state_changed.get_current());
            },
            MessageView::Eos(..) => {
                let p = current_playlist.lock().unwrap();
                (*p).current_position = ((*p).current_position +1);
                if (*p).current_position >= (*p).items.len() as i64{
                    (*p).current_position = 0;
                } else {
                    
                }
                println!("Eos found");
            },
            _ => (),
        }
    }
}

fn gstreamer_init(current_playlist: CurrentPlaylist) -> Result<Arc<Mutex<gstreamer::Element>>> {
    gstreamer::init().unwrap();
    let pipeline = gstreamer::parse_launch("playbin")?;
    let p = Arc::new(Mutex::new(pipeline));

    let pp = p.clone();
    let cp = current_playlist.clone();
    std::thread::spawn(|| {
        gstreamer_message_handler(pp, cp);
    });
    Ok(p)
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("../ui/main.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let mut grid: gtk::Viewport = builder.get_object("playlistviewport").unwrap();
    println!("Building list");
    let  (playlist, current_playlist_grid) = playlist::playlist_from_directory("/mnt/ssd-media/Musik/1rest");
    let current_playlist = Arc::new(Mutex::new(playlist));
    println!("Done building list");
    
    let window: gtk::Window = builder.get_object("mainwindow").unwrap();
    
    let pipeline = gstreamer_init(current_playlist.clone()).unwrap();

    /// TODO: make all this use the bus instead?
    {
        let button: gtk::Button = builder.get_object("playButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline => move |_| {
            let mut p = pipeline.lock().unwrap();
            (*p).set_property("uri", &playlist::get_current_uri(current_playlist.clone()));
            p.set_state(gstreamer::State::Playing);
        }));
    }
    {
        let button: gtk::Button = builder.get_object("pauseButton").unwrap();
        button.connect_clicked(clone!(pipeline => move |_| {
            let mut p = pipeline.lock().unwrap();      
            match p.get_state(gstreamer::ClockTime(Some(1000))) {
                (_, gstreamer::State::Paused, _) =>  { (*p).set_state(gstreamer::State::Playing); },
                (_, gstreamer::State::Playing, _) => { (*p).set_state(gstreamer::State::Paused);  },
                (_, _, _) => {}
            }
        }));
    }
 
    {
        grid.add(&current_playlist_grid);
    }

    
    window.connect_delete_event(clone!(pipeline => move |_, _| {
        let mut p = pipeline.lock().unwrap();
        (*p).set_state(gstreamer::State::Null);
        gtk::main_quit();
        Inhibit(false)
    }));

    window.show_all();
    gtk::main();
}