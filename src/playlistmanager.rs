use gdk;
use gstreamer;
use gstreamer::ElementExt;
use gtk;
use gtk::prelude::*;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

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
    treeview: Rc<gtk::TreeView>,
}

#[derive(Clone)]
pub struct PlaylistManager {
    notebook: gtk::Notebook,
    playlist_tabs: Vec<PlaylistTab>,
    pipeline: Pipeline,
    current_playlist: CurrentPlaylist,
    builder: Gui,
    gui_action: Rc<GuiActionFn>,
}

pub trait PlaylistManagerExt {
    fn put_playlist_in_gui(&mut self, LoadedPlaylist);
}

impl PlaylistManagerExt for PlaylistManager {
    fn put_playlist_in_gui(&mut self, lp: LoadedPlaylist) {
        println!("doing new");

        let label = gtk::Label::new(Some(lp.name.as_str()));
        label.show();
        
        //populate the thing
        let tv = create_populated_treeview(&lp, &self);

        self.notebook.append_page(&tv, Some(&label));
        self.notebook.next_page();
        println!("{}", self.notebook.get_n_pages());
        
        //I want to replace the value arc is pointing to but not the arc
        //this arc is in a bunch of values and I need to keep it the same
        let mut np = self.current_playlist.write().unwrap();
        *np = lp;    

        let tab = PlaylistTab { lp: self.current_playlist.clone(), treeview: Rc::new(tv)};
        self.playlist_tabs.push(tab);
    }
}

fn create_populated_treeview(lp: &LoadedPlaylist, plm: &PlaylistManager) -> gtk::TreeView {
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
    connect_treeview(&treeview, plm);
    treeview.show();
    treeview
}

type GuiActionFn = Fn(CurrentPlaylist, Gui, Pipeline, &GStreamerAction) -> ();

fn connect_treeview(treeview: &gtk::TreeView, plm: &PlaylistManager) {
    let current_playlist = &plm.current_playlist;
    let builder = &plm.builder;
    let gui_action = &plm.gui_action;
    let pipeline = &plm.pipeline;

    treeview.connect_button_press_event(
        clone!(pipeline, current_playlist, builder, gui_action => move |tv, eventbutton| {
        if eventbutton.get_event_type() == gdk::EventType::DoubleButtonPress {
            let (vec, _) = tv.get_selection().get_selected_rows();
            if vec.len() == 1 {
                let pos = vec[0].get_indices()[0];
                (gui_action)(current_playlist.clone(), builder.clone(), pipeline.clone(), &GStreamerAction::Play(pos));
            }
            gtk::Inhibit(true)
        } else {
            gtk::Inhibit(false)
        }
    }),
    );
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

pub fn new(
    notebook: gtk::Notebook,
    pipeline: Pipeline,
    current_playlist: CurrentPlaylist,
    builder: Gui,
    gui_action: Rc<GuiActionFn>,
) -> PlaylistManager {
    let plm = PlaylistManager {
        notebook: notebook,
        pipeline: pipeline,
        playlist_tabs: Vec::new(),
        current_playlist: current_playlist,
        builder: builder,
        gui_action: gui_action,
    };
    plm
}
