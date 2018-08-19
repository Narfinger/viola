//! The main gui parts.

use gdk;
use gdk_pixbuf;
use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use db;
use gstreamer_wrapper;
use gstreamer_wrapper::{GStreamer, GStreamerAction, GStreamerExt};
use loaded_playlist::LoadedPlaylist;
use playlist;
use playlist_tabs;
use playlist_tabs::PlaylistTabsExt;
use types::*;

/// Gui is the main struct for calling things on the gui or in the gstreamer. It will take care that
/// everything is the correct state. You should probably interface it with a GuiPtr.
pub struct MainGui {
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
pub fn new(pool: &DBPool, builder: &BuilderPtr) -> MainGuiPtr {
    let pltabs = playlist_tabs::new();
    let (gst, recv) = gstreamer_wrapper::new(pltabs.clone(), pool.clone()).unwrap();
    let p: gtk::Paned = builder.read().unwrap().get_object("paned").unwrap();
    p.set_position(80);

    let g = Rc::new(MainGui {
        pool: pool.clone(),
        notebook: builder
            .read()
            .unwrap()
            .get_object("playlistNotebook")
            .unwrap(),
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
pub trait MainGuiExt {
    //fn get_active_treeview(&self) -> &gtk::TreeView;
    fn update_gui(&self, &PlayerStatus); //does not need pipeline
    fn set_playback(&self, &GStreamerAction);
    fn append_to_playlist(&self, Vec<db::Track>);
    fn replace_playlist(&self, Vec<db::Track>);
    fn insert_tracks(&self, i32, Vec<db::Track>);
    fn save(&self, &DBPool);
}

impl MainGuiExt for MainGui {
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

                    self.title_label.set_text(&track.title);
                    self.artist_label.set_text(&track.artist);
                    self.album_label.set_text(&track.album);
                    self.status_label.set_text("Playing");
                    if let Some(ref p) = track.albumpath {
                        if let Ok(ref pp) = gdk_pixbuf::Pixbuf::new_from_file_at_size(p, 200, 200) {
                            self.cover.set_from_pixbuf(pp);
                        } else {
                            println!("error creating pixbuf");
                        }
                    } else {
                        self.cover.clear();
                    }

                    //highlight row
                    let pos = self.playlist_tabs.borrow().current_position();
                    let model: gtk::ListStore = treeview
                        .get_model()
                        .unwrap()
                        .downcast::<gtk::ListStore>()
                        .unwrap();
                    let path = gtk::TreePath::new_from_indicesv(&[pos, COLOR_COLUMN as i32]);
                    let treeiter = model.get_iter(&path).unwrap();
                    //let (_, selection) = treeselection.get_selected().unwrap();
                    {
                        let cell = self.last_marked.borrow();
                        if let Some(ref previous_row) = *cell {
                            let color = gdk::RGBA {
                                red: 0.0,
                                green: 0.0,
                                blue: 0.0,
                                alpha: 0.0,
                            };
                            let c = gdk_pixbuf::Value::from(&color);
                            model.set_value(&previous_row, COLOR_COLUMN, &c);
                        }
                    }
                    let color = gdk::RGBA {
                        red: 0.6,
                        green: 0.0,
                        blue: 0.3,
                        alpha: 0.6,
                    };
                    let c = gdk_pixbuf::Value::from(&color);
                    model.set_value(&treeiter, COLOR_COLUMN, &c);

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
                    self.cover.clear();

                    //clear playling line
                    panic!("need to clear playling line");
                }
            }
        }
    }

    fn set_playback(&self, status: &GStreamerAction) {
        self.gstreamer.do_gstreamer_action(status);
    }

    fn append_to_playlist(&self, t: Vec<db::Track>) {
        self.playlist_tabs.borrow_mut().append_to_playlist(t);
    }

    fn replace_playlist(&self, t: Vec<db::Track>) {
        self.playlist_tabs.borrow_mut().replace_playlist(t);
    }

    fn insert_tracks(&self, index: i32, t: Vec<db::Track>) {
        self.playlist_tabs.borrow_mut().insert_tracks(index, t);
    }

    fn save(&self, pool: &DBPool) {
        self.playlist_tabs.borrow().save(pool);
    }
}

/// Trait for all functions that need a GuiPtr instead of just a gui. This is different to GuiExt, as these
/// will generally setup a gtk callback. As gtk callbacks have a static lifetime, we need the lifetime guarantees
/// of GuiPtr instead of just a reference.
pub trait MainGuiPtrExt {
    fn page_changed(&self, u32);
    fn add_page(&self, LoadedPlaylist);
    fn delete_page(&self, u32);
    fn restore(&self, &DBPool);
    fn signal_handler(self, tv: &gtk::TreeView, event: &gdk::Event) -> gtk::Inhibit;
}

impl MainGuiPtrExt for MainGuiPtr {
    fn page_changed(&self, index: u32) {
        println!("Page changed to {}", index);
        (*self.playlist_tabs)
            .borrow_mut()
            .set_current_playlist(index as i32);
    }

    fn add_page(&self, lp: LoadedPlaylist) {
        let label = gtk::Label::new(Some(lp.name.as_str()));

        ///FIXME we should use one of the enum but it doesn't exist yet?
        let icon = gtk::Image::new_from_icon_name("window-close", 32);
        let button = gtk::ToolButton::new(&icon, "");
        button.set_icon_name("window-close");
        button.show();

        let b = gtk::Box::new(gtk::Orientation::Horizontal, 20);
        b.pack_start(&label, false, false, 0);
        b.pack_start(&button, false, false, 0);

        let (scw, tab) = playlist_tabs::load_tab(&self.playlist_tabs, self.clone(), lp);
        let index = self.notebook.append_page(&scw, Some(&b));
        b.show_all();
        scw.show_all();

        {
            let s = self.clone();
            button.connect_clicked(move |_| s.delete_page(index));
        }

        (*self.playlist_tabs).borrow_mut().add_tab(tab);
    }

    fn delete_page(&self, index: u32) {
        println!("deleting the page {}", index);
        let db_id = (*self.playlist_tabs).borrow().id(index as i32);
        (*self.playlist_tabs).borrow_mut().remove_tab(index as i32);
        self.notebook.remove_page(Some(index));
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

    /// Handles mouse button presses in treeviews/playlistviews
    fn signal_handler(self, tv: &gtk::TreeView, event: &gdk::Event) -> gtk::Inhibit {
        if event.get_event_type() == gdk::EventType::DoubleButtonPress {
            let (vec, _) = tv.get_selection().get_selected_rows();
            if vec.len() == 1 {
                let pos = vec[0].get_indices()[0];
                self.gstreamer
                    .do_gstreamer_action(&GStreamerAction::Play(pos));
                self.update_gui(&PlayerStatus::Playing);
            }
            gtk::Inhibit(true)
        } else {
            gtk::Inhibit(false)
        }
    }
}
