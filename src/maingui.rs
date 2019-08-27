//! The main gui parts.

use gdk;
use gdk_pixbuf;
use gtk;
use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::sync_channel;

use crate::db;
use crate::gstreamer_wrapper;
use crate::gstreamer_wrapper::{GStreamer, GStreamerAction, GStreamerExt};
use crate::loaded_playlist::LoadedPlaylist;
use crate::playlist;
use crate::playlist_tabs;
use crate::playlist_tabs::PlaylistTabsExt;
use crate::types::*;
use crate::utils::format_into_full_duration;

/// Gui is the main struct for calling things on the gui or in the gstreamer. It will take care that
/// everything is the correct state. You should probably interface it with a GuiPtr.
pub struct MainGui {
    pool: DBPool,
    notebook: gtk::Notebook,
    title_label: gtk::Label,
    artist_label: gtk::Label,
    album_label: gtk::Label,
    status_label: gtk::Label,
    elapsed_label: gtk::Label,
    total_label: gtk::Label,
    total_playtime_label: gtk::Label,
    time_scale: gtk::Scale,
    cover: gtk::Image,
    repeat_once: gtk::Image,
    last_marked: RefCell<Option<gtk::TreeIter>>,
    playlist_tabs: PlaylistTabsPtr,
    gstreamer: Rc<GStreamer>,
    update_playtime_channel: std::sync::mpsc::SyncSender<i64>,
}

/// Constructs a new gui, given a BuilderPtr and a loaded playlist.
pub fn new(pool: &DBPool, builder: &BuilderPtr) -> MainGuiPtr {
    let pltabs = playlist_tabs::new();
    let (gst, recv) = gstreamer_wrapper::new(pltabs.clone(), pool.clone()).unwrap();
    let (playtime_update_send, playtime_update_reicv) = sync_channel::<i64>(10);
    let p: gtk::Paned = builder.read().unwrap().get_object("paned").unwrap();
    p.set_position(150); // this is in pixels

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
        elapsed_label: builder.read().unwrap().get_object("elapsedLabel").unwrap(),
        total_label: builder.read().unwrap().get_object("totalLabel").unwrap(),
        total_playtime_label: builder
            .read()
            .unwrap()
            .get_object("totalPlaytimeLabel")
            .unwrap(),
        time_scale: builder.read().unwrap().get_object("timeScale").unwrap(),
        cover: builder.read().unwrap().get_object("coverImage").unwrap(),
        repeat_once: builder
            .read()
            .unwrap()
            .get_object("repeatOnceImage")
            .unwrap(),
        last_marked: RefCell::new(None),
        playlist_tabs: pltabs,
        gstreamer: gst.clone(),
        update_playtime_channel: playtime_update_send,
    });

    let gc = g.clone();
    gtk::timeout_add(250, move || {
        //println!("trying to get channel");
        if let Ok(t) = recv.try_recv() {
            gc.update_gui(&t.into());
        }
        gtk::Continue(true)
    });

    {
        let guic = g.clone();
        let poolc = pool.clone();
        gtk::timeout_add_seconds(60 * 30, move || {
            info!("autosaving database");
            guic.save(&poolc);
            gtk::Continue(true)
        });
    }
    //g.add_page(loaded_playlist);
    {
        let gc = g.clone();
        g.time_scale.connect_change_value(move |_, _, pos| {
            gc.change_time_scale(pos);
            gtk::Inhibit(true)
        });
    }

    {
        let gc = g.clone();
        g.notebook.connect_switch_page(move |_, _, index| {
            gc.page_changed(index);
        });
    }

    {
        //updateing total playtime
        let gc = g.clone();
        gtk::timeout_add_seconds(5, move || {
            // there might be old values in the queue, we only want the last value
            let elem = playtime_update_reicv.try_iter().last();

            if let Some(i) = elem {
                gc.total_playtime_label
                    .set_text(&format_into_full_duration(i));
            }
            gtk::Continue(true)
        });
    }

    g
}

/// This is a trait for all gui related functions that do not need a GuiPtr, only a reference to the gui.
/// The main indication is: This are all functions that do not need to have gtk callbacks.
pub trait MainGuiExt {
    //fn get_active_treeview(&self) -> &gtk::TreeView;
    fn clear_play_marker(&self);
    fn update_gui(&self, _: &PlayerStatus); //does not need pipeline
    fn set_playback(&self, _: &GStreamerAction);
    fn change_time_scale(&self, _: f64);
    fn append_to_playlist(&self, _: Vec<db::Track>);
    fn replace_playlist(&self, _: Vec<db::Track>);
    fn insert_tracks(&self, _: i32, _: Vec<db::Track>);
    fn save(&self, _: &DBPool);
}

impl MainGuiExt for MainGui {
    //fn get_active_treeview(&self) -> &gtk::TreeView {
    //    let cur_page = self.notebook.get_current_page().unwrap();
    //    println!("The page: {:?}", cur_page);
    //    &self.playlist_tabs.borrow()[cur_page as usize].treeview
    //}

    fn clear_play_marker(&self) {
        if let Some(cur_page) = self.notebook.get_current_page() {
            let treeview = &self.playlist_tabs.borrow().tabs[cur_page as usize].treeview;
            //let pos = self.playlist_tabs.borrow().current_position();
            let model: gtk::ListStore = treeview
                .get_model()
                .unwrap()
                .downcast::<gtk::ListStore>()
                .unwrap();
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
        }
    }

    /// General purpose function to update the GuiPtr on any change
    fn update_gui(&self, status: &PlayerStatus) {
        if let Some(cur_page) = self.notebook.get_current_page() {
            let treeview = &self.playlist_tabs.borrow().tabs[cur_page as usize].treeview;
            //let treeselection = treeview.get_selection();
            self.repeat_once.clear();
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
                    {
                        let mut artist = track.artist.clone();
                        artist.truncate(15);
                        self.artist_label.set_text(&artist);
                    }
                    self.album_label.set_text(&track.album);
                    self.status_label.set_text("Playing");
                    if let Some(ref p) = track.albumpath {
                        if let Ok(ref pp) = gdk_pixbuf::Pixbuf::new_from_file_at_size(p, 200, 200) {
                            self.cover.set_from_pixbuf(Some(pp));
                        } else {
                            error!("error creating pixbuf");
                        }
                    } else {
                        self.cover.clear();
                    }
                    self.elapsed_label.set_text("0:00");
                    self.total_label
                        .set_text(&format_duration(track.length as u64, track.length as u64));
                    self.time_scale
                        .set_range(f64::from(0), f64::from(track.length));

                    //highlight row
                    let pos = self.playlist_tabs.borrow().current_position();
                    let model: gtk::ListStore = treeview
                        .get_model()
                        .unwrap()
                        .downcast::<gtk::ListStore>()
                        .unwrap();
                    let path = gtk::TreePath::new_from_indicesv(&[pos, COLOR_COLUMN as i32]);
                    let treeiter = model.get_iter(&path).unwrap();

                    self.clear_play_marker();

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
                    self.elapsed_label.set_text("0:00");
                    self.cover.clear();

                    self.clear_play_marker();
                }
                PlayerStatus::ChangedDuration((i, total)) => {
                    self.elapsed_label.set_text(&format_duration(i, total));
                    self.time_scale.set_value(i as f64);
                }
            }
        }
    }

    fn set_playback(&self, status: &GStreamerAction) {
        if *status == GStreamerAction::RepeatOnce {
            self.repeat_once
                .set_from_icon_name(Some("gtk-undelete"), gtk::IconSize::SmallToolbar);
        }
        self.gstreamer.do_gstreamer_action(status);
    }

    fn change_time_scale(&self, pos: f64) {
        let p = pos.trunc() as u64;
        self.gstreamer
            .do_gstreamer_action(&GStreamerAction::Seek(p));
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

/// takes the current_position and formats it according to the complete position, given in seconds
fn format_duration(current_position: u64, total: u64) -> String {
    let s = current_position % 60;
    let m = current_position / 60 % (60 * 60);
    let h = current_position / (60 * 60);
    //warn!("current, total {}/{}", current_position, total);
    if total >= 60 * 60 {
        format!("{}:{:02}:{:02}", h, m, s)
    } else if total >= 60 {
        format!("{}:{:02}", m, s)
    } else {
        format!("{}", s)
    }
}

/// Trait for all functions that need a GuiPtr instead of just a gui. This is different to GuiExt, as these
/// will generally setup a gtk callback. As gtk callbacks have a static lifetime, we need the lifetime guarantees
/// of GuiPtr instead of just a reference.
pub trait MainGuiPtrExt {
    fn page_changed(&self, _: u32);
    fn add_page(&self, _: LoadedPlaylist);
    fn delete_page(&self, _: u32);
    fn restore(&self, _: &DBPool);
    fn signal_handler(self, tv: &gtk::TreeView, event: &gdk::Event) -> gtk::Inhibit;
}

impl MainGuiPtrExt for MainGuiPtr {
    fn page_changed(&self, index: u32) {
        info!("Page changed to {}", index);
        (*self.playlist_tabs)
            .borrow_mut()
            .set_current_playlist(index as i32);
    }

    fn add_page(&self, lp: LoadedPlaylist) {
        let label = gtk::Label::new(Some(lp.name.as_str()));

        ///FIXME we should use one of the enum but it doesn't exist yet?
        let icon =
            gtk::Image::new_from_icon_name(Some("window-close"), gtk::IconSize::SmallToolbar);
        let button = gtk::ToolButton::new(Some(&icon), None);
        button.set_icon_name(Some("window-close"));
        button.show();

        let b = gtk::Box::new(gtk::Orientation::Horizontal, 20);
        b.pack_start(&label, false, false, 0);
        b.pack_start(&button, false, false, 0);

        let (scw, tab) = playlist_tabs::load_tab(
            &self.playlist_tabs,
            self.clone(),
            lp,
            self.update_playtime_channel.clone(),
        );
        let index = self.notebook.append_page(&scw, Some(&b));
        b.show_all();
        scw.show_all();
        {
            let s = self.clone();
            // the deletion of the data structures needs to happen here because signals are complicated in gtk
            // sometimes the notebook-page-removed signal happens on destruction of the guy
            // in the current gtk-rs it is really difficult to block that signal from being emitted
            button.connect_clicked(move |_| {
                s.notebook.remove_page(Some(index));

                //this deletes the data structure behind the playlist
                s.delete_page(index);
            });
        }

        (*self.playlist_tabs).borrow_mut().add_tab(tab);
    }

    fn delete_page(&self, index: u32) {
        info!("deleting the page {}", index);
        let db_id = (*self.playlist_tabs).borrow().id(index as i32);
        //let new_index = (*self.playlist_tabs).borrow_mut().remove_tab(index as i32);

        self.page_changed(0);
        self.notebook.set_current_page(Some(0));
        //deleting in database
        if let Some(i) = db_id {
            playlist::delete_with_id(&self.pool, i as i32);
        }
    }

    fn restore(&self, pool: &DBPool) {
        for lp in playlist::restore_playlists(pool).expect("Error restoring playlisttabs") {
            self.add_page(lp);
        }
        self.playlist_tabs.borrow().update_playtime();
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
