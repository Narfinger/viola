use std::io::BufReader;
use std::thread;
use std::time::Duration;
use std;
use std::error::Error;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Button, ListBox, Layout, Label, Grid, Orientation, PositionType, ScrolledWindow, Window, WindowType};
use gstreamer;
use gstreamer::ElementExt;
use taglib;
use walkdir;
use walkdir::{DirEntry, WalkDir};

pub struct Playlist {
    pub items: Vec<String>,
    pub current_position: i64,
    pub grid: Grid,
}

pub fn playlist_from_directory(folder: &str) -> Playlist {
    let mut grid = Grid::new();
    let strings = parse_folder(folder);
    build_widget(&strings, &mut grid);
    Playlist { items: strings, current_position: 0, grid: grid }
}

fn check_dir(s: &Result<DirEntry, walkdir::Error>) -> bool {
    if let &Ok(ref sp) = s {
        sp.file_type().is_file()
    } else {
        false
    }
}

fn parse_folder(folder: &str) -> Vec<String> {
    // TODO this currently also has folders in it 
    WalkDir::new(folder).into_iter().filter(check_dir).map(|i| String::from(i.unwrap().path().to_str().unwrap())).collect::<Vec<String>>()
}

fn build_widget(p: &Vec<String>, w: &mut Grid) {
    for (i, val) in p.iter().enumerate() {
        let fpath = &val;
        let taglibfile = taglib::File::new(fpath);
        if let Err(e) = taglibfile {
            println!("Error {:?}", e);
        } else {
            let ataglib = taglibfile.unwrap();
            let tags = ataglib.tag().unwrap();
            
            let title = Label::new(Some(tags.title().as_str()));
            let artist = Label::new(Some(tags.artist().as_str()));
            let album = Label::new(Some(tags.album().as_str()));
            w.attach(&title,  0, i as i32, 1, 1);
            w.attach(&artist, 1, i as i32, 1, 1);
            w.attach(&album,  2, i as i32, 1, 1);
        }
    }
}

pub fn play(p: Rc<Playlist>) -> Option<gstreamer::Element> {
    // Build the pipeline
    let uri = &p.items[p.current_position as usize].replace(" ", "%20");
    println!("{}", uri);
    let pipeline = gstreamer::parse_launch(&format!("playbin uri=file:////{}", uri));
    match pipeline {
        Err(e) => { 
            println!("{:?}, {}", e.cause(), e.description());
                    None 
            },
        Ok(p)  => { 
            p.set_state(gstreamer::State::Playing);
            println!("PLaying");
            Some(p)
            } 
    }
}