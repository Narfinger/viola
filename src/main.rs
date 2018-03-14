
pub mod errors;
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

use errors::*;

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

fn gstreamer_init() -> Result<Arc<Mutex<gstreamer::Element>>> {
    gstreamer::init().unwrap();
    let pipeline = gstreamer::parse_launch("playbin")?;
    let bus = pipeline.get_bus();
    let p = Arc::new(Mutex::new(pipeline));
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
    let playlist = Rc::new(playlist::playlist_from_directory("/mnt/ssd-media/Musik/1rest"));
    println!("Done building list");
    
    let window: gtk::Window = builder.get_object("mainwindow").unwrap();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    
    let pipeline = gstreamer_init().unwrap();

    /// TODO: make all this use the bus instead?
    {
        let button: gtk::Button = builder.get_object("playButton").unwrap();
        button.connect_clicked(clone!(playlist, pipeline => move |_| {
            let mut p = pipeline.lock().unwrap();
            (*p).set_property("uri", &playlist::get_current_uri(&playlist));
            p.set_state(gstreamer::State::Playing);
            println!("Doing");
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
 
    grid.add(&playlist.grid);

    window.show_all();
    gtk::main();
}