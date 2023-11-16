#![recursion_limit = "4096"]
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

use anyhow::{bail, Context, Result};
use clap::Parser;
use log::info;
use parking_lot::Mutex;
use preferences::{prefs_base_dir, Preferences, PreferencesMap};
use std::sync::Arc;
use types::DBPool;

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

fn update_db(pool: &DBPool) -> Result<(), anyhow::Error> {
    info!("Updating Database");
    let mut pref_reader =
        crate::utils::get_config_file(&utils::ConfigWriteMode::Read).expect("No settings file");
    let preferences = PreferencesMap::<String>::load_from(&mut pref_reader)
        .context("Could not read settings file")?;
    let music_dir = preferences
        .get("music_dir")
        .context("Could not get musicdir")?;
    db::build_db(music_dir, pool, true).unwrap();
    println!("creating m3u playlists");
    smartplaylist_parser::m3u_from_smartplaylist(music_dir, pool)?;
    Ok(())
}

fn update_db_fast(path: String, pool: &DBPool) {
    info!("Updating database with path {}", path);
    if !std::path::Path::new(&path).exists() {
        println!("Path does not seem to exist");
    }
    db::build_db(&path, pool, false).unwrap();
}

fn set_music_directory(new_music_dir: String) -> Result<(), anyhow::Error> {
    let mut prefs = PreferencesMap::<String>::new();
    prefs.insert(String::from("music_dir"), new_music_dir);
    let mut prefs_file =
        crate::utils::get_config_file(&utils::ConfigWriteMode::Write).expect("Cannot find config");
    prefs
        .save_to(&mut prefs_file)
        .context("Error in saving preferences")?;
    info!("saved music directory");
    Ok(())
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
        update_db(&pool)?;
    } else if let Some(path) = args.fast_update {
        update_db_fast(path, &pool);
    } else if let Some(new_music_dir) = args.music_dir {
        set_music_directory(new_music_dir)?;
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
        path.extend(["viola", "smartplaylists.toml"]);
        open::that(&path).unwrap_or_else(|_| panic!("Could not open file {:?}", &path));
    } else if args.webview {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(maingui_web::run(pool));
    } else {
        println!("Unsupported for now");
    };
    Ok(())
}
