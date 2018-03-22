extern crate clap;
#[macro_use] extern crate diesel;
extern crate indicatif;
extern crate gdk;
extern crate gtk;
extern crate gstreamer;
extern crate rayon;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate taglib;
extern crate walkdir;

pub mod db;
pub mod playlist;
pub mod schema;
pub mod types;

use clap::{Arg, App};
use std::sync::Arc;
use std::sync::RwLock;
use gstreamer::ElementExt;
use gtk::prelude::*;

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

enum PlayerStatus {
    Playing,
    Paused,
    Stopped
}

/// poll the message bus and on eos start new
fn gstreamer_message_handler(pipeline: Pipeline, current_playlist: CurrentPlaylist, builder: Gui) -> gtk::Continue {
    let bus = {
        pipeline.read().unwrap().get_bus().unwrap()
    };
    if let Some(msg) = bus.pop() {
        use gstreamer::MessageView;
        match msg.view() {
            MessageView::Error(err) => {
                eprintln!("Error received {}", err.get_error());
                eprintln!("Debugging information: {:?}", err.get_debug());
            }
            MessageView::StateChanged(state_changed) => {
                println!("Pipeline state changed from {:?} to {:?}",
                        state_changed.get_old(),
                        state_changed.get_current());
                //if state_changed.get_current() == gstreamer::State::Playing {
                //    update_gui(&pipeline, &current_playlist, &builder);
                //}
            },
            MessageView::Eos(..) => {
                let mut p = current_playlist.write().unwrap();
                (*p).current_position += 1;
                if (*p).current_position >= (*p).items.len() as i64{
                    (*p).current_position = 0;
                    update_gui(&pipeline, &current_playlist, &builder, &PlayerStatus::Stopped);
                } else {
                    println!("Next should play");
                    let pl = pipeline.read().unwrap();
                    (*pl).set_state(gstreamer::State::Ready);
                    (*pl).set_property("uri", &playlist::get_current_uri(&p));
                    (*pl).set_state(gstreamer::State::Playing);
                    println!("Next one now playing is: {}", &playlist::get_current_uri(&p));
                    update_gui(&pipeline, &current_playlist, &builder, &PlayerStatus::Playing)
                }
                println!("Eos found");
            },
            _ => (),
        }
    }
    gtk::Continue(true)
}

fn gstreamer_init(current_playlist: CurrentPlaylist, builder: Gui) -> Result<Pipeline, String> {
    gstreamer::init().unwrap();
    let pipeline = gstreamer::parse_launch("playbin").map_err(|_| String::from("Cannot do gstreamer"))?;
    let p = Arc::new(RwLock::new(pipeline));

    let pc = p.clone();
    gtk::timeout_add(500, move || {
        let pc = p.clone();
        let cpc = current_playlist.clone();
        let bc = builder.clone();
        gstreamer_message_handler(pc, cpc, bc)
    });
 
     Ok(pc)
}

/// General purpose function to update the gui on any change
fn update_gui(pipeline: &Pipeline, playlist: &CurrentPlaylist, gui: &Gui, status: &PlayerStatus) {
    println!("Updating gui");
    let (_, state, _) = pipeline.read().unwrap().get_state(gstreamer::ClockTime(Some(1000)));  
    let treeview: gtk::TreeView = gui.read().unwrap().get_object("listview").unwrap();
    let treeselection = treeview.get_selection();
    match *status {
        PlayerStatus::Playing => {
    //if state == gstreamer::State::Paused || state == gstreamer::State::Playing {
        let index = playlist.read().unwrap().current_position;
        let mut ipath = gtk::TreePath::new();
        ipath.append_index(index as i32);
        treeselection.select_path(&ipath);

        //update track display
        let track = &playlist.read().unwrap().items[index as usize];
        let titlelabel: gtk::Label = gui.read().unwrap().get_object("titleLabel").unwrap();
        let artistlabel: gtk::Label = gui.read().unwrap().get_object("artistLabel").unwrap();
        let albumlabel: gtk::Label = gui.read().unwrap().get_object("albumLabel").unwrap();
        let cover: gtk::Image = gui.read().unwrap().get_object("coverImage").unwrap();

        titlelabel.set_markup(&track.title);
        artistlabel.set_markup(&track.artist);
        albumlabel.set_markup(&track.album);
        if let Some(ref p) = track.albumpath {
            cover.set_from_file(p);
        } else {
            cover.clear();
        }
    },
    _ => {}
    }
}

fn build_gui(pool: &DBPool) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("../ui/main.glade");
    let builder: Gui = Arc::new(RwLock::new(gtk::Builder::new_from_string(glade_src)));

    println!("Building list");
    let playlist = playlist::playlist_from_directory("/mnt/ssd-media/Musik/1rest", pool);
    let current_playlist = Arc::new(RwLock::new(playlist));
    println!("Done building list");
    
    let window: gtk::Window = builder.read().unwrap().get_object("mainwindow").unwrap();
    let treeview: gtk::TreeView = builder.read().unwrap().get_object("listview").unwrap();

    let pipeline = gstreamer_init(current_playlist.clone(), builder.clone()).unwrap();

    
    { // Play Button
        let button: gtk::Button = builder.read().unwrap().get_object("playButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline, builder => move |_| {
            {
                let p = pipeline.read().unwrap();
                let pl = current_playlist.read().unwrap();
                (*p).set_property("uri", &playlist::get_current_uri(&pl));
                p.set_state(gstreamer::State::Playing);
                update_gui(&pipeline, &current_playlist, &builder, &PlayerStatus::Playing); 
            }
        }));
    }
    { // Pause Button
        let button: gtk::Button = builder.read().unwrap().get_object("pauseButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline, builder  => move |_| {
            {
                let p = pipeline.read().unwrap();      
                match p.get_state(gstreamer::ClockTime(Some(1000))) {
                    (_, gstreamer::State::Paused, _) =>  { (*p).set_state(gstreamer::State::Playing); },
                    (_, gstreamer::State::Playing, _) => { (*p).set_state(gstreamer::State::Paused);  },
                    (_, _, _) => {}
                };
                update_gui(&pipeline, &current_playlist, &builder, &PlayerStatus::Paused); 
            }
        }));
    }
    {  // Previous button
        let button: gtk::Button = builder.read().unwrap().get_object("prevButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline, builder => move |_| {
            {
                let p = pipeline.read().unwrap();
                let mut pl = current_playlist.write().unwrap();
                (*p).set_state(gstreamer::State::Paused);
                (*p).set_state(gstreamer::State::Ready);
                (*pl).current_position = ((*pl).current_position -1) % (*pl).items.len() as i64;
                (*p).set_property("uri", &playlist::get_current_uri(&pl)).expect("Error in changing url");
                (*p).set_state(gstreamer::State::Playing);
                update_gui(&pipeline, &current_playlist, &builder, &PlayerStatus::Playing); 
            }
        }));
    }
    {  // Next button
        let button: gtk::Button = builder.read().unwrap().get_object("nextButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline, builder => move |_| {
            {
                let p = pipeline.read().unwrap();
                let mut pl = current_playlist.write().unwrap();
                (*p).set_state(gstreamer::State::Paused);
                (*p).set_state(gstreamer::State::Ready);
                (*pl).current_position = ((*pl).current_position +1) % (*pl).items.len() as i64;
                (*p).set_property("uri", &playlist::get_current_uri(&pl)).expect("Error in changing url");
                (*p).set_state(gstreamer::State::Playing);
                update_gui(&pipeline, &current_playlist, &builder, &PlayerStatus::Playing); 
            }
        }));
    }

    let model = gtk::ListStore::new(&[String::static_type(), String::static_type(), String::static_type(), String::static_type(), String::static_type(), String::static_type(), String::static_type()]);
    
    {
        let p = current_playlist.read().unwrap();
        for (i, entry) in p.items.iter().enumerate() {
             model.insert_with_values(None, &[0,1,2,3,4,5,6], &[&entry.tracknumber.map(|s| s.to_string())
                .unwrap_or_else (|| String::from("")), 
             &entry.title, &entry.artist, &entry.album, &entry.length, &entry.year.map(|s| s.to_string())
                .unwrap_or_else(|| String::from("")), 
             &entry.genre]);
        }
        for &(id, title) in &[(0,"#"), (1, "Title"), (2, "Artist"), (3, "Album"), (4, "Length"), (5, "Year"), (6, "Genre")] {
            let column = gtk::TreeViewColumn::new();
            let cell = gtk::CellRendererText::new();
            column.pack_start(&cell, true);
            // Association of the view's column with the model's `id` column.
            column.add_attribute(&cell, "text", id);
            column.set_title(title);
            column.set_resizable(id>0);
            treeview.append_column(&column);
        }
        treeview.connect_button_press_event(clone!(pipeline, current_playlist, builder => move |tv, eventbutton| {
            if eventbutton.get_event_type() == gdk::EventType::DoubleButtonPress {
                let (vec, tv2) = tv.get_selection().get_selected_rows();
                if vec.len() == 1 {
                    println!("we should work");
                    let p = pipeline.read().unwrap();
                    p.set_state(gstreamer::State::Paused);
                    p.set_state(gstreamer::State::Ready);
                    let pos = vec[0].get_indices()[0];
                    let mut cp = current_playlist.write().unwrap();
                    (*cp).current_position = i64::from(pos);
                    (*p).set_property("uri", &playlist::get_current_uri(&cp));
                    p.set_state(gstreamer::State::Playing);
                }
                update_gui(&pipeline, &current_playlist, &builder, &PlayerStatus::Playing);
                gtk::Inhibit(true)
            } else {
                gtk::Inhibit(false)
            }
        }));
        treeview.get_selection().connect_changed(move |ts| {
            println!("selecting");
        });
        treeview.set_model(Some(&model));
    }
    
    
    window.connect_delete_event(clone!(pipeline => move |_, _| {
        let p = pipeline.read().unwrap();
        (*p).set_state(gstreamer::State::Null);
        gtk::main_quit();
        Inhibit(false)
    }));

    window.show_all();
    gtk::main();
}

fn main() {
    let matches = App::new("Viola")
        .about("Music Player")
        .arg(Arg::with_name("update")
            .short("u")
            .long("update")
            .help("Updates the database"))
        .get_matches();

    let pool = db::setup_db_connection();        
    if matches.is_present("update") {
        println!("Updating Database");
        db::build_db("/mnt/ssd-media/Musik/1a - Test", &pool.clone()).unwrap();
    } else {
        build_gui(&pool);
    }
}