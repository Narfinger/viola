extern crate app_dirs;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gstreamer;
extern crate gtk;
extern crate indicatif;
extern crate open;
extern crate pango;
extern crate preferences;
extern crate rand;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate taglib;
extern crate toml;
extern crate url;
extern crate walkdir;


pub mod albumviewstore;
pub mod db;
pub mod gstreamer_wrapper;
pub mod gui;
pub mod libraryviewstore;
pub mod loaded_playlist;
pub mod maingui;
pub mod playlist;
pub mod playlist_manager;
pub mod playlist_tabs;
pub mod schema;
pub mod smartplaylist_parser;
pub mod types;

use clap::{App, Arg};
use gio::ApplicationExt;
use preferences::{prefs_base_dir, AppInfo, Preferences, PreferencesMap};

const APP_INFO: AppInfo = AppInfo {
    name: "viola",
    author: "narfinger",
};
const PREFS_KEY: &str = "viola_prefs";

fn main() {
    let matches = App::new("Viola")
        .about("Music Player")
        .version(crate_version!())
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .help("Updates the database"),
        ).arg(
            Arg::with_name("fastupdate")
                .short("f")
                .long("fastupdate")
                .help("Does a fast update of the database, doing a heuristic on time modified"),
        ).arg(
            Arg::with_name("music_dir")
                .short("m")
                .takes_value(true)
                .long("music_dir")
                .help("Set the music directory"),
        ).arg(
            Arg::with_name("configpath")
                .short("c")
                .long("config")
                .help("Shows the config path"),
        ).arg(
            Arg::with_name("editsmartplaylists")
                .short("e")
                .long("editsmartplaylists")
                .help("Opens an editor to edit the smartplaylist file"),
        ).get_matches();

    env_logger::init();


    let pool = db::setup_db_connection();
    if matches.is_present("update") {
        info!("Updating Database");
        if let Ok(preferences) = PreferencesMap::<String>::load(&APP_INFO, PREFS_KEY) {
            if let Some(music_dir) = preferences.get("music_dir") {
                db::build_db(music_dir, &pool.clone()).unwrap();
            } else {
                error!("Could not find music_dir");
            }
        } else {
            error!("could not find settings file");
        }
    } else if matches.is_present("fastupdate") {
        panic!("not yet implemented");
    } else if let Some(new_music_dir) = matches.value_of("music_dir") {
        let mut prefs = PreferencesMap::<String>::new();
        prefs.insert(String::from("music_dir"), String::from(new_music_dir));
        prefs
            .save(&APP_INFO, PREFS_KEY)
            .expect("Error in saving preferences");
        info!("saved music directory");
    } else if matches.is_present("configpath") {
        let mut p = prefs_base_dir().expect("Base dir cannot be founds");
        p.push("viola");
        let s = p.to_str().expect("Error in convert");
        error!(
            "The config path can be found under {}.\n Please add the file smartplaylists.toml\
             if you want to add smartplaylists",
            s
        );
    } else if matches.is_present("editsmartplaylists") {
        let mut path = prefs_base_dir().expect("Could not find base dir");
        path.extend(&["viola", "smartplaylists.toml"]);
        open::that(&path).unwrap_or_else(|_| panic!("Could not open file {:?}", &path));
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
