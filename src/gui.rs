use std::rc::Rc;
use gtk;
use types::*;


/// TODO try to get all this as references and not as Rc with explicit lifetimes
pub struct Gui {
    notebook: gtk::Notebook,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    album_label: gtk::Label,
    cover: gtk::Image, 
}

pub fn new(builder: GuiPtr) -> Gui {
    Gui {
    notebook: builder.read().unwrap().get_object("playlistNotebook").unwrap(),
    title_label: builder.read().unwrap().get_object("titleLabel").unwrap(),
    artist_label: builder.read().unwrap().get_object("artistLabel").unwrap(),
    album_label: builder.read().unwrap().get_object("albumLabel").unwrap(),
    cover: builder.read().unwrap().get_object("coverImage").unwrap(),
    }
}