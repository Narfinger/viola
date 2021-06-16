#![recursion_limit = "4096"]
extern crate app_dirs;
extern crate base64;
extern crate bus;
extern crate tokio;
extern crate warp;
#[macro_use]
extern crate clap;
extern crate zbus;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate glib;
extern crate gstreamer;
extern crate humantime;
extern crate image;
extern crate indicatif;
extern crate open;
extern crate preferences;
extern crate rand;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate parking_lot;
extern crate rusqlite;
extern crate serde_json;
extern crate toml;
extern crate walkdir;
//extern crate jwalk;

pub mod db;
pub mod dbus_interface;
pub mod gstreamer_wrapper;
pub mod libraryviewstore;
pub mod loaded_playlist;
pub mod maingui_web;
pub mod my_websocket;
pub mod playlist;
pub mod playlist_tabs;
pub mod smartplaylist_parser;
pub mod types;
pub mod utils;

use clap::{App, Arg};
use parking_lot::Mutex;
use preferences::{prefs_base_dir, Preferences, PreferencesMap};
use std::env;
use std::sync::Arc;

fn main() {
    let matches = App::new("Viola")
        .about("Music Player")
        .version(crate_version!())
        .arg(
            Arg::with_name("update")
                .short("u")
                .long("update")
                .help("Updates the database"),
        )
        .arg(
            Arg::with_name("fastupdate")
                .short("f")
                .value_name("path")
                .long("fastupdate")
                .help("Does a fast update of the database, doing a heuristic on time modified"),
        )
        .arg(
            Arg::with_name("music_dir")
                .short("m")
                .takes_value(true)
                .long("music_dir")
                .help("Set the music directory"),
        )
        .arg(
            Arg::with_name("configpath")
                .short("c")
                .long("config")
                .help("Shows the config path"),
        )
        .arg(
            Arg::with_name("editsmartplaylists")
                .short("e")
                .long("editsmartplaylists")
                .help("Opens an editor to edit the smartplaylist file"),
        )
        .arg(
            Arg::with_name("no-webview")
                .short("w")
                .long("nowebview")
                .help("Does not run the embedded webview"),
        )
        .get_matches();

    //env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let tmp_pool = db::setup_db_connection();
    if tmp_pool.is_err() {
        println!("Something is wrong with db, creating it.");
        db::create_db();
        println!("Please call viola with -m to set the music dir.");
        println!("Afterwards, update the music library by calling with -u.");
        return;
    }
    let pool = Arc::new(Mutex::new(tmp_pool.unwrap()));
    if matches.is_present("update") {
        info!("Updating Database");
        if let Ok(preferences) =
            PreferencesMap::<String>::load(&crate::types::APP_INFO, crate::types::PREFS_KEY)
        {
            if let Some(music_dir) = preferences.get("music_dir") {
                db::build_db(music_dir, &pool, true).unwrap();
            } else {
                error!("Could not find music_dir");
            }
        } else {
            error!("could not find settings file");
        }
    } else if let Some(path) = matches.value_of("fastupdate") {
        info!("Updating database with path {}", path);
        if !std::path::Path::new(path).exists() {
            println!("Path does not seem to exist");
        }
        db::build_db(path, &pool, false).unwrap();
    } else if let Some(new_music_dir) = matches.value_of("music_dir") {
        let mut prefs = PreferencesMap::<String>::new();
        prefs.insert(String::from("music_dir"), String::from(new_music_dir));
        prefs
            .save(&crate::types::APP_INFO, crate::types::PREFS_KEY)
            .expect("Error in saving preferences");
        info!("saved music directory");
    } else if matches.is_present("configpath") {
        let mut p = prefs_base_dir().expect("Base dir cannot be founds");
        p.push("viola");
        let s = p.to_str().expect("Error in convert");
        println!(
            "The config path can be found under {}.\n Please add the file smartplaylists.toml\
             if you want to add smartplaylists",
            s
        );
    } else if matches.is_present("editsmartplaylists") {
        let mut path = prefs_base_dir().expect("Could not find base dir");
        path.extend(&["viola", "smartplaylists.toml"]);
        open::that(&path).unwrap_or_else(|_| panic!("Could not open file {:?}", &path));
    } else if matches.is_present("no-webview") {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(maingui_web::run(pool));
    //});
    } else {
        std::thread::spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(maingui_web::run(pool));
        });
        std::thread::sleep(std::time::Duration::from_secs(1));

        use web_view::*;
        println!("Starting webview");
        WebViewBuilder::new()
            .title("Viola")
            .content(Content::Url("http://localhost:8080"))
            .size(1920, 1080)
            .resizable(true)
            //.debug(true)
            .user_data(())
            .invoke_handler(|_webview, _arg| Ok(()))
            .build()
            .unwrap()
            .run()
            .unwrap();
    };
}
