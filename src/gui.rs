use std::rc::Rc;
use std::sync::{Arc, RwLock};
use gdk;
use gdk_pixbuf;
use gtk;
use gtk::prelude::*;
use playlist::LoadedPlaylist;
use types::*;

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

#[derive(Clone)]
struct PlaylistTab {
    lp: Arc<RwLock<LoadedPlaylist>>,
    treeview: gtk::TreeView,
}

/// TODO try to get all this as references and not as Rc with explicit lifetimes
pub struct Gui {
    notebook: gtk::Notebook,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    album_label: gtk::Label,
    cover: gtk::Image,
    current_playlist: LoadedPlaylist,
    playlist_tabs: Vec<PlaylistTab>,
}

pub fn new(builder: GuiPtr, current_playlist: LoadedPlaylist) -> Gui {
    Gui {
    notebook: builder.read().unwrap().get_object("playlistNotebook").unwrap(),
    title_label: builder.read().unwrap().get_object("titleLabel").unwrap(),
    artist_label: builder.read().unwrap().get_object("artistLabel").unwrap(),
    album_label: builder.read().unwrap().get_object("albumLabel").unwrap(),
    cover: builder.read().unwrap().get_object("coverImage").unwrap(),
    current_playlist: current_playlist,
    playlist_tabs: Vec::new(),
    }
}


pub trait GuiExt {
    fn get_active_treeview(&self) -> &gtk::TreeView;
    fn update_gui(&self, &PlayerStatus); //does not need pipeline
    fn add_page(&self, &LoadedPlaylist);
}

impl GuiExt for Gui {
    fn get_active_treeview(&self) -> &gtk::TreeView {
        let cur_page = self.notebook.get_current_page().unwrap();
        &self.playlist_tabs[cur_page as usize].treeview
    }

    /// General purpose function to update the GuiPtr on any change
    fn update_gui(&self, status: &PlayerStatus) {
        let treeview = self.get_active_treeview();
        let treeselection = treeview.get_selection();
        match *status {
            PlayerStatus::Playing => {
                //if state == gstreamer::State::Paused || state == gstreamer::State::Playing {
                let index = self.current_playlist.current_position;
                let mut ipath = gtk::TreePath::new();
                ipath.append_index(index as i32);
                treeselection.select_path(&ipath);

                //update track display
                let track = &self.current_playlist.items[index as usize];

                self.title_label.set_markup(&track.title);
                self.artist_label.set_markup(&track.artist);
                self.album_label.set_markup(&track.album);
                if let Some(ref p) = track.albumpath {
                    if let Ok(ref pp) = gdk_pixbuf::Pixbuf::new_from_file_at_size(p,300,300) {
                        self.cover.set_from_pixbuf(pp);
                    } else {
                        println!("error creating pixbuf");
                    }

                } else {
                    self.cover.clear();
                }
            }
            _ => {}
        }
    }

    fn add_page(&self, lp: &LoadedPlaylist) {

    }
}

fn create_populated_treeview(gui: &Gui, lp: &LoadedPlaylist) -> gtk::TreeView {
    let treeview = gtk::TreeView::new();
    for &(id, title, width) in &[
        (0, "#", 50),
        (1, "Title", 500),
        (2, "Artist", 200),
        (3, "Album", 200),
        (4, "Length", 200),
        (5, "Year", 200),
        (6, "Genre", 200),
    ] {
        let column = gtk::TreeViewColumn::new();
        let cell = gtk::CellRendererText::new();
        column.pack_start(&cell, true);
        // Association of the view's column with the model's `id` column.
        column.add_attribute(&cell, "text", id);
        column.set_title(title);
        column.set_resizable(id > 0);
        column.set_fixed_width(width);
        treeview.append_column(&column);
    }

    treeview.set_model(Some(&populate_model_with_playlist(lp)));
    treeview.connect_button_press_event(|tv, eventbutton| {
        if eventbutton.get_event_type() == gdk::EventType::DoubleButtonPress {
            let (vec, _) = tv.get_selection().get_selected_rows();
            if vec.len() == 1 {
                let pos = vec[0].get_indices()[0];
                //gui.update_gui(&GStreamerAction::Play(pos));
            }
            gtk::Inhibit(true)
        } else {
            gtk::Inhibit(false)
        }
    }
    );
    treeview.show();
    treeview
}

fn connect_treeview(treeview: &gtk::TreeView) {
}

fn populate_model_with_playlist(lp: &LoadedPlaylist) -> gtk::ListStore  {
    let model = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    for (i, entry) in lp.items.iter().enumerate() {
    model.insert_with_values(
        None,
        &[0, 1, 2, 3, 4, 5, 6],
        &[
            &entry
                .tracknumber
                .map(|s| s.to_string())
                .unwrap_or_else(|| String::from("")),
            &entry.title,
            &entry.artist,
            &entry.album,
            &entry.length,
            &entry
                .year
                .map(|s| s.to_string())
                .unwrap_or_else(|| String::from("")),
            &entry.genre,
        ],
    );
    }

    model
}