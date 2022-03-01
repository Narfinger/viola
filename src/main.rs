#![recursion_limit = "4096"]
#[macro_use]
extern crate anyhow;
extern crate base64;
extern crate clap;
extern crate tokio;
extern crate warp;
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
extern crate indicatif;
extern crate open;
extern crate preferences;
extern crate rand;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate parking_lot;
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

use anyhow::{Context, Result};
use clap::Parser;
use parking_lot::Mutex;
use preferences::{prefs_base_dir, Preferences, PreferencesMap};
use std::sync::Arc;

///A Music player that does exactly what I want with a webinterface.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Updates the database
    #[clap(short, long)]
    update: bool,

    /// Does a fast update of the database, doing a heuristic on time modified
    #[clap(short, long)]
    fast_update: Option<String>,

    /// Sets the music directory
    #[clap(short, long)]
    music_dir: Option<String>,

    /// Shows the config path
    #[clap(short, long)]
    config_path: bool,

    /// Opens and editor to edit the smartplaylist file
    #[clap(short, long)]
    edit_smartplaylist: bool,

    /// Does not run the embedded webview
    #[clap(short, long)]
    webview: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    //env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let (shutdown_send, mut shutdown_recv) = tokio::sync::mpsc::unbounded_channel::<()>();

    let tmp_pool = db::setup_db_connection();
    if tmp_pool.is_err() {
        println!("Something is wrong with db, creating it.");
        db::create_db();
        println!("Please call viola with -m to set the music dir.");
        println!("Afterwards, update the music library by calling with -u.");
        bail!("See Above: ");
    }
    let pool = Arc::new(Mutex::new(tmp_pool.unwrap()));
    if args.update {
        info!("Updating Database");
        let mut pref_reader =
            crate::utils::get_config_file(&utils::ConfigWriteMode::Read).expect("No settings file");

        let preferences = PreferencesMap::<String>::load_from(&mut pref_reader)
            .context("Could not read settings file")?;
        let music_dir = preferences
            .get("music_dir")
            .context("Could not get musicdir")?;
        db::build_db(music_dir, &pool, true).unwrap();
    } else if let Some(path) = args.fast_update {
        info!("Updating database with path {}", path);
        if !std::path::Path::new(&path).exists() {
            println!("Path does not seem to exist");
        }
        db::build_db(&path, &pool, false).unwrap();
    } else if let Some(new_music_dir) = args.music_dir {
        let mut prefs = PreferencesMap::<String>::new();
        prefs.insert(String::from("music_dir"), new_music_dir);
        let mut prefs_file = crate::utils::get_config_file(&utils::ConfigWriteMode::Write)
            .expect("Cannot find config");
        prefs
            .save_to(&mut prefs_file)
            .context("Error in saving preferences")?;
        info!("saved music directory");
    } else if args.config_path {
        let mut p = prefs_base_dir().context("Base dir cannot be founds")?;
        p.push("viola");
        let s = p.to_str().context("Error in convert")?;
        println!(
            "The config path can be found under {}.\n Please add the file smartplaylists.toml\
             if you want to add smartplaylists",
            s
        );
    } else if args.edit_smartplaylist {
        let mut path = prefs_base_dir().context("Could not find base dir")?;
        path.extend(&["viola", "smartplaylists.toml"]);
        open::that(&path).unwrap_or_else(|_| panic!("Could not open file {:?}", &path));
    } else if args.webview {
        //tokio::runtime::Builder::new_current_thread()
        //.enable_all()
        //    .build()
        //    .unwrap()
        //    .block_on(maingui_web::run(pool));
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(maingui_web::run(pool));
    //});
    } else {
        use web_view::*;
        println!("Starting webview");
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            WebViewBuilder::new()
                .title("Viola")
                .content(Content::Url(crate::types::URL))
                .size(1920, 1080)
                .resizable(true)
                //.debug(true)
                .user_data(())
                .invoke_handler(|_webview, _arg| Ok(()))
                .build()
                .unwrap()
                .run()
                .unwrap();
            info!("Webview exited");
            shutdown_send.send(()).expect("error in shutdown");
        });
        //std::thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                tokio::select! {
                    _ = shutdown_recv.recv() => {},
                    _ =  maingui_web::run(pool) => {},
                }
            });
        //});
    };
    Ok(())
}
