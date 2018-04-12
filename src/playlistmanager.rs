use gdk;
use gstreamer;
use gstreamer::ElementExt;
use gtk;
use gtk::prelude::*;
use std::rc::Rc;

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

type GuiActionFn = Fn(CurrentPlaylist, Gui, Pipeline, &GStreamerAction) -> ();

#[derive(Clone)]
pub struct PlaylistManager {
    notebook: gtk::Notebook,
    treeview: gtk::TreeView,
    pipeline: Pipeline,
    current_playlist: CurrentPlaylist,
    builder: Gui,
    gui_action: Rc<GuiActionFn>,
}

pub trait PlaylistManagerExt {
    fn put_playlist_in_gui(&self, LoadedPlaylist);
}

impl PlaylistManagerExt for PlaylistManager {
    fn put_playlist_in_gui(&self, lp: LoadedPlaylist) {
        println!("doing new");


        let tv = gtk::TreeView::new();
        let label = gtk::Label::new(Some(lp.name.as_str()));
        tv.show();
        label.show();
        populate_treeview(&lp, &tv);
        self.notebook.append_page(&tv, Some(&label));
        self.notebook.next_page();
        println!("{}", self.notebook.get_n_pages());
    }
}

fn populate_treeview(lp: &LoadedPlaylist, tv: &gtk::TreeView) {

}

pub fn new(
    notebook: gtk::Notebook,
    treeview: gtk::TreeView,
    pipeline: Pipeline,
    current_playlist: CurrentPlaylist,
    builder: Gui,
    gui_action: Rc<GuiActionFn>,
) -> PlaylistManager {
    let plm = PlaylistManager {
        notebook: notebook,
        treeview: treeview,
        pipeline: pipeline,
        current_playlist: current_playlist,
        builder: builder,
        gui_action: gui_action,
    };
    setup(&plm);
    plm
}

/// TODO clean this up
fn setup(plm: &PlaylistManager) {
    let current_playlist = &plm.current_playlist;
    let notebook = &plm.notebook;
    let treeview = &plm.treeview;
    let pipeline = &plm.pipeline;
    let builder = &plm.builder;

    let model = gtk::ListStore::new(&[
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);
    {
        let p = current_playlist.read().unwrap();
        let child = &notebook.get_children()[0];
        notebook.set_tab_label_text(child, p.name.as_str());
        for (i, entry) in p.items.iter().enumerate() {
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
        for &(id, title, width) in &[
            (0, "#", 50),
            (1, "Title", 700),
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

        let gui_action = &plm.gui_action;
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
        /* treeview.get_selection().connect_changed(move |ts| {
            println!("selecting");
        }); */
        treeview.set_model(Some(&model));
    }
}
