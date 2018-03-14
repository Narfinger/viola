
pub mod playlist;

extern crate gtk;
extern crate gstreamer;
extern crate rodio;
extern crate taglib;
extern crate walkdir;

use std::sync::Mutex;
use std::sync::Arc;
use std::rc::Rc;
use gstreamer::ElementExt;

use gtk::prelude::*;
use gtk::{Button, ListBox, Layout, Label, Grid, Orientation, PositionType, ScrolledWindow, Window, WindowType};
use walkdir::WalkDir;

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

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("../ui/main.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let mut grid: gtk::Viewport = builder.get_object("playlistviewport").unwrap();
    println!("Building list");
    let playlist = Rc::new(playlist::playlist_from_directory("/mnt/ssd-media/Musik/1rest"));
    println!("Done building list");
    
    let window: gtk::Window = builder.get_object("mainwindow").unwrap();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    
    gstreamer::init().unwrap();
    let mut pipeline: Arc<Mutex<Option<gstreamer::Element>>> = Arc::new(Mutex::new(None));
    
    {
        let button: gtk::Button = builder.get_object("playButton").unwrap();
        button.connect_clicked(clone!(playlist, pipeline => move |_| {
            let mut state = pipeline.lock().expect("Mutex wrong");
            *state = playlist::play(playlist.clone());
        }));
    }
    {
        let button: gtk::Button = builder.get_object("pauseButton").unwrap();
        button.connect_clicked(clone!(pipeline => move |_| {
            let mut state = pipeline.lock().expect("Mutex wrong");
            if let Some(ref s) = *state {
                /* match s.get_state(true) {
                    gstreamer::State::Paused => s.set_state(gstreamer::State::Playing),
                    gstreamer::State::Playing => s.set_state(gstreamer::State::Paused),
                }; */
                s.set_state(gstreamer::State::Paused);
            }
        }));
    }    

    grid.add(&playlist.grid);

    window.show_all();
    gtk::main();
}