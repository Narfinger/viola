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
pub mod loaded_playlist;
pub mod gui;
pub mod gstreamer_wrapper;
pub mod libraryviewstore;
pub mod playlist;
pub mod playlist_tabs;
pub mod playlist_manager;
pub mod schema;
pub mod types;

use clap::{App, Arg};
use gio::ApplicationExt;
use gtk::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;

use gui::{GuiExt, GuiPtrExt};
use gstreamer_wrapper::GStreamerAction;

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


fn build_gui(application: &gtk::Application, pool: DBPool) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("../ui/main.glade");
    let builder = Arc::new(RwLock::new(gtk::Builder::new_from_string(glade_src)));

    println!("Building list");
    //let playlist = playlist::load_playlist_from_directory("/mnt/ssd-media/Musik", &pool);
    println!("Done building list");

    let window: gtk::ApplicationWindow = builder.read().unwrap().get_object("mainwindow").unwrap();
    //let pipeline = gstreamer_init(current_playlist.clone()).unwrap();
    let gui = gui::new(&pool, &builder);
  
    {
        // Play Button
        let button: gtk::Button = builder.read().unwrap().get_object("playButton").unwrap();
        button.connect_clicked(clone!(gui => move |_| {
            {
                (*gui).set_playback(&GStreamerAction::Playing);
            }
        }));
    }
    {
        // Pause Button
        let button: gtk::Button = builder.read().unwrap().get_object("pauseButton").unwrap();
        button.connect_clicked(clone!(gui  => move |_| {
            {
                (*gui).set_playback(&GStreamerAction::Pausing);
            }
        }));
    }
    {
        // Previous button
        let button: gtk::Button = builder.read().unwrap().get_object("prevButton").unwrap();
        button.connect_clicked(clone!(gui => move |_| {
            {
                (*gui).set_playback(&GStreamerAction::Previous);
            }
        }));
    }
    {
        // Next button
        let button: gtk::Button = builder.read().unwrap().get_object("nextButton").unwrap();
        button.connect_clicked(clone!(gui => move |_| {
            {
                (*gui).set_playback(&GStreamerAction::Next)
            }
        }));
    }

    let libview = libraryviewstore::new(pool.clone(), &builder, gui.clone());
    let plmview = playlist_manager::new(pool.clone(), &builder, gui.clone());

    window.maximize();
    window.set_application(application);
    window.set_title("Viola");
    window.connect_delete_event(clone!(window, gui, pool => move |_, _| {
        playlist::clear_tabs(&pool);
        let playlisttabs = &gui.playlist_tabs.borrow().tabs;
        for pl in playlisttabs.into_iter().map(|ref t| &t.lp) {
            playlist::update_playlist(&pool, &pl);
        }
        window.destroy();
        Inhibit(false)
    }));

    window.show_all();
    println!("Restoring tabs");
    for pl in playlist::restore_playlists(&pool).expect("Error in database (restoring tabs)") {
        gui.add_page(pl);
    }

    println!("\n\n\n Current Bugs:");
    println!("tab close button does not work when we close a different tab then the current one.");
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
