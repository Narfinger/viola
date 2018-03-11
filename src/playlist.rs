use gtk::prelude::*;
use gtk::{Button, ListBox, Layout, Label, Grid, Orientation, PositionType, ScrolledWindow, Window, WindowType};
use taglib;
use walkdir::WalkDir;

pub struct Playlist {
    pub current_position: i64,
    pub grid: Grid,
}

pub fn playlist_from_directory(folder: &str) -> Playlist {
    let mut grid = Grid::new();
    parse_folder(folder, &mut grid);
    Playlist { current_position: 0, grid: grid }
}

fn parse_folder(folder: &str, w: &mut Grid) {
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
            
                let title = Label::new(Some(tags.title().as_str()));
                let artist = Label::new(Some(tags.artist().as_str()));
                let album = Label::new(Some(tags.album().as_str()));
                w.attach(&title,  0, row, 1, 1);
                w.attach(&artist, 1, row, 1, 1);
                w.attach(&album,  2, row, 1, 1);
                row += 1;

                }
            }
        }
    }
}
