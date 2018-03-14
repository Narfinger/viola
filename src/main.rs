
pub mod playlist;

extern crate gtk;
extern crate gstreamer;
extern crate rodio;
extern crate taglib;
extern crate walkdir;

use std::rc::Rc;
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
    
    

    let button: gtk::Button = builder.get_object("playButton").unwrap();
    button.connect_clicked(clone!(playlist => move |_| {
        playlist::play(playlist.clone());
    }));
    

    grid.add(&playlist.grid);

    window.show_all();
    gtk::main();
}