extern crate gtk;
extern crate taglib;
extern crate walkdir;
use gtk::prelude::*;
use gtk::{Button, ListBox, Layout, Label, Grid, Orientation, PositionType, ScrolledWindow, Window, WindowType};
use walkdir::WalkDir;

fn parse_folder(w: &mut Grid) {
    let folder = "/mnt/ssd-media/Musik/1rest";
    let mut row = 0;
    for i in  WalkDir::new(folder) {
        if let Ok(f) = i {
            if f.file_type().is_file() {
                
                let fpath = f.path().to_str();
                let taglibfile = taglib::File::new(fpath.unwrap());
                if let Err(e) = taglibfile {
                    println!("Error {:?}", e);
                } else {
                let ataglib = taglibfile.unwrap();
                let tags = ataglib.tag().unwrap();
                let st:String = format!("{} - {} - {}", tags.title(), tags.artist(), tags.album());
                let stt:&str = &st;
                let label = Label::new(Some(stt));
                w.attach(&label, 0,row,1,1);
                row += 1;

                }
            }
        }
    }
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("../ui/main.glade");
    let builder = gtk::Builder::new_from_string(glade_src);

    let mut grid: gtk::Grid = builder.get_object("playlist").unwrap();
    println!("Building list");
    parse_folder(&mut grid);
    println!("Done building list");
    

    let window: gtk::Window = builder.get_object("mainwindow").unwrap();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all();
    gtk::main();
}