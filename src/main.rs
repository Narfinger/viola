extern crate gtk;
extern crate taglib;
extern crate walkdir;
use gtk::prelude::*;
use gtk::{Button, ListBox, Layout, Label, Grid, Orientation, PositionType, ScrolledWindow, Window, WindowType};
use walkdir::WalkDir;

use playlist::Playlist;
pub mod playlist;


fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("../ui/main.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let mut grid: gtk::Viewport = builder.get_object("playlistviewport").unwrap();
    println!("Building list");
    let playlist = playlist::playlist_from_directory("/mnt/ssd-media/Musik/1rest");
    println!("Done building list");
    
    let window: gtk::Window = builder.get_object("mainwindow").unwrap();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    grid.add(&playlist.grid);

    window.show_all();
    gtk::main();
}