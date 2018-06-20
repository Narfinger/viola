extern crate app_dirs;
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
pub mod maingui;
pub mod playlist;
pub mod playlist_tabs;
pub mod playlist_manager;
pub mod schema;
pub mod smartplaylist_parser;
pub mod types;

use clap::{App, Arg};
use gio::ApplicationExt;
use preferences::{AppInfo, PreferencesMap, Preferences, prefs_base_dir};

const APP_INFO: AppInfo = AppInfo{name: "viola", author: "narfinger"};
const PREFS_KEY: &'static str = "viola_prefs";

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
            Arg::with_name("fastupdate")
                .short("f")
                .long("fastupdate")
                .help("Does a fast update of the database, doing a heuristic on time modified"))
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
    } else if matches.is_present("fastupdate") {
        panic!("not yet implemented");
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
            gui::build_gui(app, &pool);
        });
        application.connect_activate(|_| {});
        application.run(&[]);
    }
}
