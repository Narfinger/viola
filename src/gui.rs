//! The main gui parts.

use std::rc::Rc;
use std::cell::RefCell;
use gdk;
use gdk_pixbuf;
use gtk;
use gtk::prelude::*;

use gstreamer_wrapper;
use gstreamer_wrapper::{GStreamer, GStreamerExt, GStreamerAction};
use playlist;
use loaded_playlist::LoadedPlaylist;
use playlist_tabs;
use playlist_tabs::PlaylistTabsExt;
use types::*;

/// Gui is the main struct for calling things on the gui or in the gstreamer. It will take care that
/// everything is the correct state. You should probably interface it with a GuiPtr.
pub struct Gui {
    pool: DBPool,
    notebook: gtk::Notebook,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    album_label: gtk::Label,
    status_label: gtk::Label,
    cover: gtk::Image,
    last_marked: RefCell<Option<gtk::TreeIter>>,
    playlist_tabs: PlaylistTabsPtr,
    gstreamer: Rc<GStreamer>,
}

/// Constructs a new gui, given a BuilderPtr and a loaded playlist.
pub fn new(pool: &DBPool, builder: &BuilderPtr) -> GuiPtr {
    let pltabs = playlist_tabs::new();
    let (gst, recv) = gstreamer_wrapper::new(pltabs.clone()).unwrap();

    let g = Rc::new(Gui {
        pool: pool.clone(),
        notebook: builder.read().unwrap().get_object("playlistNotebook").unwrap(),
        title_label: builder.read().unwrap().get_object("titleLabel").unwrap(),
        artist_label: builder.read().unwrap().get_object("artistLabel").unwrap(),
        album_label: builder.read().unwrap().get_object("albumLabel").unwrap(),
        status_label: builder.read().unwrap().get_object("statusLabel").unwrap(),
        cover: builder.read().unwrap().get_object("coverImage").unwrap(),
        last_marked: RefCell::new(None),
        playlist_tabs: pltabs,
        gstreamer: gst.clone(),
    });

    let gc = g.clone();
    gtk::timeout_add(250, move || {
        //println!("trying to get channel");
        if let Ok(t) = recv.try_recv() {
            gc.update_gui(&t.into());
        }
        gtk::Continue(true)
    });

    //g.add_page(loaded_playlist);

    {
        let gc = g.clone();
    
        g.notebook.connect_switch_page(move |_, _, index| {
            gc.page_changed(index);
        });
    }

    g
}

/// This is a trait for all gui related functions that do not need a GuiPtr, only a reference to the gui.
/// The main indication is: This are all functions that do not need to have gtk callbacks.
pub trait GuiExt {
    //fn get_active_treeview(&self) -> &gtk::TreeView;
    fn update_gui(&self, &PlayerStatus); //does not need pipeline
    fn set_playback(&self, &GStreamerAction);
    fn save(&self, &DBPool);
}

impl GuiExt for Gui {
    //fn get_active_treeview(&self) -> &gtk::TreeView {
    //    let cur_page = self.notebook.get_current_page().unwrap();
    //    println!("The page: {:?}", cur_page);
    //    &self.playlist_tabs.borrow()[cur_page as usize].treeview
    //}

    /// General purpose function to update the GuiPtr on any change
    fn update_gui(&self, status: &PlayerStatus) {
        if let Some(cur_page) = self.notebook.get_current_page() {
            let treeview = &self.playlist_tabs.borrow().tabs[cur_page as usize].treeview;
            //let treeselection = treeview.get_selection();
            match *status {
                PlayerStatus::Playing => {
                    //if state == gstreamer::State::Paused || state == gstreamer::State::Playing {
                    let index = self.playlist_tabs.borrow().current_position();
                    let mut ipath = gtk::TreePath::new();
                    ipath.append_index(index as i32);
                    //treeselection.select_path(&ipath);

                    //update track display
                    let tabs = self.playlist_tabs.borrow();
                    let track = &tabs.current_track();

                    self.title_label.set_markup(&track.title);
                    self.artist_label.set_markup(&track.artist);
                    self.album_label.set_markup(&track.album);
                    self.status_label.set_markup("Playing");
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
                    let pos = self.playlist_tabs.borrow().current_position();
                    let model: gtk::ListStore = treeview.get_model().unwrap().downcast::<gtk::ListStore>().unwrap();
                    let path = gtk::TreePath::new_from_indicesv(&[pos, 7]);
                    let treeiter = model.get_iter(&path).unwrap();
                    //let (_, selection) = treeselection.get_selected().unwrap();
                    println!("doing color");
                    println!("this does not reset the colors of the other things");
                    println!("Resetting the other color");
                    {
                        let cell = self.last_marked.borrow();
                        if let Some(ref previous_row) = *cell {
                            let color = gdk::RGBA {red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0};
                            let c = gdk_pixbuf::Value::from(&color);
                            model.set_value(&previous_row, 7, &c);
                        }
                    }
                    let color = gdk::RGBA { red: 0.6, green: 0.0, blue: 0.3, alpha: 0.6};
                    let c = gdk_pixbuf::Value::from(&color);
                    model.set_value(&treeiter, 7, &c);

                    *self.last_marked.borrow_mut() = Some(treeiter);                    
                }
                PlayerStatus::Paused => {
                    self.status_label.set_markup("Paused");
                }
                PlayerStatus::Stopped => {
                    self.title_label.set_markup("");
                    self.artist_label.set_markup("");
                    self.album_label.set_markup("");
                    self.status_label.set_markup("Playing");
                }
            }
        }
    }

    fn set_playback(&self, status: &GStreamerAction) {
        self.gstreamer.do_gstreamer_action(status);
    }

    fn save(&self, pool: &DBPool) {
        self.playlist_tabs.borrow().save(pool);
    }
}

/// Trait for all functions that need a GuiPtr instead of just a gui. This is different to GuiExt, as these
/// will generally setup a gtk callback. As gtk callbacks have a static lifetime, we need the lifetime guarantees
/// of GuiPtr instead of just a reference.
pub trait GuiPtrExt {
    fn page_changed(&self, u32);
    fn add_page(&self, LoadedPlaylist);
    fn delete_page(&self, u32);
    fn restore(&self, &DBPool);
}

impl GuiPtrExt for GuiPtr {
    fn page_changed(&self, index: u32) {
        (*self.playlist_tabs).borrow_mut().set_current_playlist(index as i32);
        //panic!("NOT YET IMPLEMENTED");
    }

    fn add_page(&self, lp: LoadedPlaylist) {
        let tv = create_populated_treeview(&self, &lp);
        let scw = gtk::ScrolledWindow::new(None, None);
        scw.add(&tv);
        let label = gtk::Label::new(Some(lp.name.as_str()));

        ///FIXME we should use one of the enum but it doesn't exist yet?
        let icon = gtk::Image::new_from_icon_name("window-close", 32);
        let button = gtk::ToolButton::new(&icon,"");
        button.set_icon_name("window-close");
        button.show();

        let b = gtk::Box::new(gtk::Orientation::Horizontal,20);
        b.pack_start(&label, false, false, 0);
        b.pack_start(&button, false, false, 0);

        let index = self.notebook.append_page(&scw, Some(&b));
        /*{
            let mut cp = self.current_playlist.write().unwrap();
            *cp = lp;
        }*/
        b.show_all();
        scw.show();

        {
            let s = self.clone();
            button.connect_clicked(move |_| {
                    s.delete_page(index)
            });
        }

        let tab = playlist_tabs::PlaylistTab { lp, treeview: tv };
        (*self.playlist_tabs).borrow_mut().add(tab);
    }

    fn delete_page(&self, index: u32) {
        let db_id = (*self.playlist_tabs).borrow().id(index as i32);
        (*self.playlist_tabs).borrow_mut().remove(index as i32);
        self.notebook.remove_page(Some(index));
        println!("deleting the page");
        //deleting in database
        if let Some(i) = db_id {
            playlist::delete_with_id(&self.pool, i as i32);
        }
    }

    fn restore(&self, pool: &DBPool) {
        for lp in playlist::restore_playlists(pool).expect("Error restoring playlisttabs") {
            self.add_page(lp);
        }
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

    for entry in &lp.items {
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