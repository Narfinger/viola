#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate gstreamer;
extern crate gtk;
extern crate indicatif;
extern crate open;
extern crate preferences;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate taglib;
extern crate toml;
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
pub mod smartplaylist_parser;
pub mod types;

use clap::{App, Arg};
use gio::ApplicationExt;
use gtk::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;
use preferences::{AppInfo, PreferencesMap, Preferences, prefs_base_dir};

const APP_INFO: AppInfo = AppInfo{name: "viola", author: "narfinger"};
const PREFS_KEY: &'static str = "viola_prefs";

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


fn build_gui(application: &gtk::Application, pool: &DBPool) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("../ui/main.glade");
    let builder = Arc::new(RwLock::new(gtk::Builder::new_from_string(glade_src)));

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
        gui.save(&pool);       
        window.destroy();
        Inhibit(false)
    }));

    window.show_all();
    println!("Restoring tabs");
    gui.restore(&pool);

    println!("\n\n\n Current Bugs:");
}

fn main() {
    let matches = App::new("Viola")
        .about("Music Player")
        .version(crate_version!())
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .help("Updates the database"))
        .arg(
            Arg::with_name("music_dir")
                .short("m")
                .takes_value(true)
                .long("music_dir")
                .help("Set the music directory"))
        .arg(
            Arg::with_name("configpath")
                .short("c")
                .long("config")
                .help("Shows the config path"))
        .arg(
            Arg::with_name("editsmartplaylists")
                .short("e")
                .long("editsmartplaylists")
                .help("Opens an editor to edit the smartplaylist file"))
        .get_matches();

    let pool = db::setup_db_connection();
    if matches.is_present("update") {
        println!("Updating Database");
        if let Ok(preferences) = PreferencesMap::<String>::load(&APP_INFO, PREFS_KEY) {
            if let Some(music_dir) = preferences.get("music_dir") {
                db::build_db(music_dir, &pool.clone()).unwrap();
            } else {
                println!("Could not find music_dir");
            }
        } else {
            println!("could not find settings file");
        }
    } else if let Some(new_music_dir) = matches.value_of("music_dir") {
        let mut prefs = PreferencesMap::<String>::new();
        prefs.insert(String::from("music_dir"), String::from(new_music_dir));
        prefs.save(&APP_INFO, PREFS_KEY).expect("Error in saving preferences");
        println!("saved music directory");
    } else if matches.is_present("configpath") {
        let mut p =  prefs_base_dir().expect("Base dir cannot be founds");
        p.push("viola");
        let s = p.to_str().expect("Error in convert");
        println!("The config path can be found under {}.\n Please add the file smartplaylists.toml\
        if you want to add smartplaylists", s);
    } else if matches.is_present("editsmartplaylists") {
        let mut path = prefs_base_dir().expect("Could not find base dir");
        path.push("viola");
        path.push("smartplaylists.toml");
        open::that(&path).expect(&format!("Could not open file {:?}", &path));
    } else {
        use gio::ApplicationExtManual;
        let application =
            gtk::Application::new("com.github.narfinger.viola", gio::ApplicationFlags::empty())
                .expect("Initialization failed...");
        application.connect_startup(move |app| {
            build_gui(app, &pool);
        });
        application.connect_activate(|_| {});
        application.run(&[]);
    }
}
