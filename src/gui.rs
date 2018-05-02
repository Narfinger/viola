//! The main gui parts.

use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::cell::RefCell;
use gdk;
use gdk_pixbuf;
use gtk;
use gtk::prelude::*;

use gstreamer_wrapper;
use gstreamer_wrapper::{GStreamer, GStreamerExt, GStreamerAction, GStreamerMessage};
use playlist::LoadedPlaylist;
use types::*;

#[derive(Clone)]
struct PlaylistTab {
    /// TODO this probably does not need multithread safe
    lp: Arc<RwLock<LoadedPlaylist>>,
    treeview: gtk::TreeView,
}

/// Gui is the main struct for calling things on the gui or in the gstreamer. It will take care that
/// everything is the correct state. You should probably interface it with a GuiPtr.
pub struct Gui {
    notebook: gtk::Notebook,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    album_label: gtk::Label,
    cover: gtk::Image,
    current_playlist: CurrentPlaylist,
    playlist_tabs: RefCell<Vec<PlaylistTab>>,
    gstreamer: Rc<GStreamer>,
}

/// Constructs a new gui, given a BuilderPtr and a loaded playlist.
pub fn new(builder: &BuilderPtr, loaded_playlist: LoadedPlaylist) -> Rc<Gui> {
    let cp = Arc::new(RwLock::new(
        LoadedPlaylist { 
            id: None, 
            name: String::from("DUMMY"), 
            items: vec![], 
            current_position: 0}));
    let (gst, recv) = gstreamer_wrapper::new(cp.clone()).unwrap();

    let g = Rc::new(Gui {
    notebook: builder.read().unwrap().get_object("playlistNotebook").unwrap(),
    title_label: builder.read().unwrap().get_object("titleLabel").unwrap(),
    artist_label: builder.read().unwrap().get_object("artistLabel").unwrap(),
    album_label: builder.read().unwrap().get_object("albumLabel").unwrap(),
    cover: builder.read().unwrap().get_object("coverImage").unwrap(),
    current_playlist: cp,
    playlist_tabs: RefCell::new(Vec::new()),
    gstreamer: gst,
    });

    let gc = g.clone();
    gtk::timeout_add(250, move || {
        //println!("trying to get channel");
        if let Ok(t) = recv.try_recv() {
            //println!("updating gui");
            match t {
                GStreamerMessage::Stopped => gc.update_gui(&PlayerStatus::Stopped),
                GStreamerMessage::Playing => gc.update_gui(&PlayerStatus::Playing),
            }
        }
        gtk::Continue(true)
    });

    g.add_page(loaded_playlist);
    g
}

/// This is a trait for all gui related functions that do not need a GuiPtr, only a reference to the gui.
/// The main indication is: This are all functions that do not need to have gtk callbacks.
pub trait GuiExt {
    //fn get_active_treeview(&self) -> &gtk::TreeView;
    fn update_gui(&self, &PlayerStatus); //does not need pipeline
    fn set_playback(&self, &GStreamerAction);
}

impl GuiExt for Gui {
    //fn get_active_treeview(&self) -> &gtk::TreeView {
    //    let cur_page = self.notebook.get_current_page().unwrap();
    //    println!("The page: {:?}", cur_page);
    //    &self.playlist_tabs.borrow()[cur_page as usize].treeview
    //}

    /// General purpose function to update the GuiPtr on any change
    fn update_gui(&self, status: &PlayerStatus) {
        let cur_page = self.notebook.get_current_page().unwrap();
        let treeview = &self.playlist_tabs.borrow()[cur_page as usize].treeview;
        let treeselection = treeview.get_selection();
        match *status {
            PlayerStatus::Playing => {
                //if state == gstreamer::State::Paused || state == gstreamer::State::Playing {
                let index = self.current_playlist.read().unwrap().current_position;
                let mut ipath = gtk::TreePath::new();
                ipath.append_index(index as i32);
                treeselection.select_path(&ipath);

                //update track display
                let track = &self.current_playlist.read().unwrap().items[index as usize];

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

                //highlight row
                let model: gtk::ListStore = treeview.get_model().unwrap().downcast::<gtk::ListStore>().unwrap();
                let (_, selection) = treeselection.get_selected().unwrap();
                println!("doing color");
                let color = gdk::RGBA { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0};
                let c = gdk_pixbuf::Value::from(&color);
                model.set_value(&selection, 7, &c);
            }
            _ => {}
        }
    }

    fn set_playback(&self, status: &GStreamerAction) {
        self.gstreamer.do_gstreamer_action(status);
    }

}

/// Trait for all functions that need a GuiPtr instead of just a gui. This is different to GuiExt, as these
/// will generally setup a gtk callback. As gtk callbacks have a static lifetime, we need the lifetime guarantees
/// of GuiPtr instead of just a reference.
pub trait GuiPtrExt {
    fn add_page(&self, lp: LoadedPlaylist);
}

impl GuiPtrExt for GuiPtr {
    fn add_page(&self, lp: LoadedPlaylist) {
        println!("added thingies");
        let tv = create_populated_treeview(&self, &lp);
        let scw = gtk::ScrolledWindow::new(None, None);
        scw.add(&tv);
        let label = gtk::Label::new(Some(lp.name.as_str()));
        scw.show();
        label.show();
        self.notebook.append_page(&scw, Some(&label));
        {
            let mut cp = self.current_playlist.write().unwrap();
            *cp = lp;
        }
        let tab = PlaylistTab { lp: self.current_playlist.clone(), treeview: tv };
        self.playlist_tabs.borrow_mut().push(tab);
        println!("added new page");
    }
}

fn create_populated_treeview(gui: &GuiPtr, lp: &LoadedPlaylist) -> gtk::TreeView {
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
        column.add_attribute(&cell, "background-rgba", 7);
        column.set_title(title);
        column.set_resizable(id > 0);
        column.set_fixed_width(width);
        treeview.append_column(&column);
    }
    treeview.set_model(Some(&populate_model_with_playlist(lp)));
    //panic!("Do the connection");
    let guic = gui.clone();
    treeview.connect_button_press_event(move |tv, eventbutton| {
        if eventbutton.get_event_type() == gdk::EventType::DoubleButtonPress {
            let (vec, _) = tv.get_selection().get_selected_rows();
            if vec.len() == 1 {
                let pos = vec[0].get_indices()[0];
                guic.gstreamer.do_gstreamer_action(&GStreamerAction::Play(pos));
                guic.update_gui(&PlayerStatus::Playing);
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

fn populate_model_with_playlist(lp: &LoadedPlaylist) -> gtk::ListStore  {
    let model = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        gdk::RGBA::static_type(),
    ]);

    for (i, entry) in lp.items.iter().enumerate() {
    model.insert_with_values(
        None,
        &[0, 1, 2, 3, 4, 5, 6, 7],
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
            &gdk::RGBA { red: 1.0, green: 1.0, blue: 1.0, alpha: 0.0},
        ],
    );
    }

    model
}