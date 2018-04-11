extern crate clap;
#[macro_use]
extern crate diesel;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate gstreamer;
extern crate gtk;
extern crate indicatif;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rayon;
extern crate taglib;
extern crate walkdir;

pub mod db;
pub mod libraryviewstore;
pub mod playlist;
pub mod playlistmanager;
pub mod schema;
pub mod types;

use clap::{App, Arg};
use gio::ApplicationExt;
use gstreamer::ElementExt;
use gtk::prelude::*;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;

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

/// poll the message bus and on eos start new
fn gstreamer_message_handler(
    pipeline: Pipeline,
    current_playlist: CurrentPlaylist,
    builder: Gui,
) -> gtk::Continue {
    let bus = { pipeline.read().unwrap().get_bus().unwrap() };
    if let Some(msg) = bus.pop() {
        use gstreamer::MessageView;
        match msg.view() {
            MessageView::Error(err) => {
                eprintln!("Error received {}", err.get_error());
                eprintln!("Debugging information: {:?}", err.get_debug());
            }
            MessageView::StateChanged(state_changed) => {
                println!(
                    "Pipeline state changed from {:?} to {:?}",
                    state_changed.get_old(),
                    state_changed.get_current()
                );
                //if state_changed.get_current() == gstreamer::State::Playing {
                //    update_gui(&pipeline, &current_playlist, &builder);
                //}
            }
            MessageView::Eos(..) => {
                let mut p = current_playlist.write().unwrap();
                (*p).current_position += 1;
                if (*p).current_position >= (*p).items.len() as i32 {
                    (*p).current_position = 0;
                    update_gui(
                        &pipeline,
                        &current_playlist,
                        &builder,
                        &PlayerStatus::Stopped,
                    );
                } else {
                    println!("Next should play");
                    let pl = pipeline.read().unwrap();
                    (*pl)
                        .set_state(gstreamer::State::Ready)
                        .into_result()
                        .expect("Error in changing gstreamer state to ready");
                    (*pl)
                        .set_property("uri", &playlist::get_current_uri(&p))
                        .expect("Error setting new url for gstreamer");
                    (*pl)
                        .set_state(gstreamer::State::Playing)
                        .into_result()
                        .expect("Error in changing gstreamer state to playing");
                    println!(
                        "Next one now playing is: {}",
                        &playlist::get_current_uri(&p)
                    );
                    update_gui(
                        &pipeline,
                        &current_playlist,
                        &builder,
                        &PlayerStatus::Playing,
                    )
                }
                println!("Eos found");
            }
            _ => (),
        }
    }
    gtk::Continue(true)
}

fn gstreamer_init(current_playlist: CurrentPlaylist, builder: Gui) -> Result<Pipeline, String> {
    gstreamer::init().unwrap();
    let pipeline =
        gstreamer::parse_launch("playbin").map_err(|_| String::from("Cannot do gstreamer"))?;
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
    let (_, state, _) = pipeline
        .read()
        .unwrap()
        .get_state(gstreamer::ClockTime(Some(1000)));
    let treeview: gtk::TreeView = gui.read().unwrap().get_object("playlistView").unwrap();
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
                if let Ok(ref pp) = gdk_pixbuf::Pixbuf::new_from_file_at_size(p,300,300) {
                    cover.set_from_pixbuf(pp);
                } else {
                    println!("error creating pixbuf");
                }
                
            } else {
                cover.clear();
            }
        }
        _ => {}
    }
}

fn do_gui_gstreamer_action(
    current_playlist: CurrentPlaylist,
    builder: Gui,
    pipeline: Pipeline,
    action: &GStreamerAction,
) {
    let mut gui_update = PlayerStatus::Playing;
    let mut gstreamer_action = gstreamer::State::Playing;
    {
        //releaingx the locks later
        let p = pipeline.read().unwrap();
        let mut pl = current_playlist.write().unwrap();
        //we need to set the state to paused and ready
        match *action {
            GStreamerAction::Play(_) | GStreamerAction::Previous | GStreamerAction::Next => {
                if gstreamer::State::Playing == (*p).get_state(gstreamer::ClockTime(Some(1000))).1 {
                    (*p).set_state(gstreamer::State::Paused)
                        .into_result()
                        .expect("Error in gstreamer state set, paused");
                    (*p).set_state(gstreamer::State::Ready)
                        .into_result()
                        .expect("Error in gstreamer state set, ready");
                }
            }
            _ => {}
        }

        match *action {
            GStreamerAction::Playing => {
                (*p).set_property("uri", &playlist::get_current_uri(&pl))
                    .expect("Error setting new gstreamer url");
            }
            GStreamerAction::Pausing => {
                if gstreamer::State::Playing == p.get_state(gstreamer::ClockTime(Some(1000))).1 {
                    gstreamer_action = gstreamer::State::Paused;
                    gui_update = PlayerStatus::Paused;
                }
            }
            GStreamerAction::Previous => {
                (*pl).current_position = ((*pl).current_position - 1) % (*pl).items.len() as i32;
                (*p).set_property("uri", &playlist::get_current_uri(&pl))
                    .expect("Error in changing url");
            }
            GStreamerAction::Next => {
                (*pl).current_position = ((*pl).current_position + 1) % (*pl).items.len() as i32;
                (*p).set_property("uri", &playlist::get_current_uri(&pl))
                    .expect("Error in changing url");
            }
            GStreamerAction::Play(i) => {
                (*pl).current_position = i;
                (*p).set_property("uri", &playlist::get_current_uri(&pl))
                    .expect("Error in chaning url");
            }
        }
        p.set_state(gstreamer_action)
            .into_result()
            .expect("Error in setting gstreamer state playing");
    } //locks releaed

    update_gui(&pipeline, &current_playlist, &builder, &gui_update);
}

fn build_gui(application: &gtk::Application, pool: DBPool) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("../ui/main.glade");
    let builder: Gui = Arc::new(RwLock::new(gtk::Builder::new_from_string(glade_src)));

    println!("Building list");
    let playlist = playlist::playlist_from_directory("/mnt/ssd-media/Musik/", &pool);
    let current_playlist = Arc::new(RwLock::new(playlist));
    println!("Done building list");

    let window: gtk::ApplicationWindow = builder.read().unwrap().get_object("mainwindow").unwrap();
    let treeview: gtk::TreeView = builder.read().unwrap().get_object("playlistView").unwrap();

    let pipeline = gstreamer_init(current_playlist.clone(), builder.clone()).unwrap();

    {
        // Play Button
        let button: gtk::Button = builder.read().unwrap().get_object("playButton").unwrap();
        button.connect_clicked(clone!(current_playlist,  builder, pipeline => move |_| {
            {
                do_gui_gstreamer_action(current_playlist.clone(), builder.clone(), pipeline.clone(), &GStreamerAction::Playing);
            }
        }));
    }
    {
        // Pause Button
        let button: gtk::Button = builder.read().unwrap().get_object("pauseButton").unwrap();
        button.connect_clicked(clone!(current_playlist, builder, pipeline  => move |_| {
            {
                do_gui_gstreamer_action(current_playlist.clone(), builder.clone(), pipeline.clone(), &GStreamerAction::Pausing);
            }
        }));
    }
    {
        // Previous button
        let button: gtk::Button = builder.read().unwrap().get_object("prevButton").unwrap();
        button.connect_clicked(clone!(current_playlist, builder, pipeline => move |_| {
            {
                do_gui_gstreamer_action(current_playlist.clone(), builder.clone(), pipeline.clone(), &GStreamerAction::Previous);
            }
        }));
    }
    {
        // Next button
        let button: gtk::Button = builder.read().unwrap().get_object("nextButton").unwrap();
        button.connect_clicked(clone!(current_playlist, builder, pipeline => move |_| {
            {
                do_gui_gstreamer_action(current_playlist.clone(), builder.clone(), pipeline.clone(), &GStreamerAction::Next)
            }
        }));
    }

    
    let notebook: gtk::Notebook = builder
        .read()
        .unwrap()
        .get_object("playlistNotebook")
        .unwrap();
    let plm: playlistmanager::PlaylistManager = playlistmanager::new(
        notebook,
        treeview,
        pipeline.clone(),
        current_playlist.clone(),
        builder.clone(),
        Rc::new(do_gui_gstreamer_action),
    );
    // building libraryview
    {
        //gtk::idle_add(clone!(pool => move || {
        let libview: gtk::TreeView = builder.read().unwrap().get_object("libraryview").unwrap();
        libraryviewstore::connect(pool.clone(), Rc::new(plm), &libview);
        //    Continue(false)
        //}));
    }

    window.maximize();
    window.set_application(application);
    window.set_title("Viola");
    window.connect_delete_event(clone!(pipeline, window => move |_, _| {
        let p = pipeline.read().unwrap();
        (*p).set_state(gstreamer::State::Null).into_result().expect("Error in setting gstreamer state: Null");
        window.destroy();
        Inhibit(false)
    }));

    window.show_all();
}

fn main() {
    let matches = App::new("Viola")
        .about("Music Player")
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .help("Updates the database"),
        )
        .get_matches();

    let pool = db::setup_db_connection();
    if matches.is_present("update") {
        println!("Updating Database");
        db::build_db("/mnt/ssd-media/Musik/", &pool.clone()).unwrap();
    } else {
        use gio::ApplicationExtManual;
        let application =
            gtk::Application::new("com.github.narfinger.viola", gio::ApplicationFlags::empty())
                .expect("Initialization failed...");
        application.connect_startup(move |app| {
            build_gui(app, pool.clone());
        });
        application.connect_activate(|_| {});
        application.run(&vec![]);
    }
}
