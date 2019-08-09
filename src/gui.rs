use gtk;
use gtk::prelude::*;
use std::sync::Arc;
use std::sync::RwLock;

use crate::albumviewstore;
use crate::gstreamer_wrapper::GStreamerAction;
use crate::libraryviewstore;
use crate::maingui;
use crate::maingui::{MainGuiExt, MainGuiPtrExt};
use crate::playlist_manager;
use crate::types::*;

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

pub fn build_gui(application: &gtk::Application, pool: &DBPool) {
    if gtk::init().is_err() {
        error!("Failed to initialize GTK.");
        return;
    }
    let glade_src = include_str!("../ui/main.glade");
    let builder = Arc::new(RwLock::new(gtk::Builder::new_from_string(glade_src)));

    let window: gtk::ApplicationWindow = builder.read().unwrap().get_object("mainwindow").unwrap();
    //let pipeline = gstreamer_init(current_playlist.clone()).unwrap();
    let gui = maingui::new(&pool, &builder);

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
    {
        // Repeat Once button
        let button: gtk::Button = builder
            .read()
            .unwrap()
            .get_object("repeatCurrentButton")
            .unwrap();
        button.connect_clicked(clone!(gui => move |_| {
            {
                (*gui).set_playback(&GStreamerAction::RepeatOnce)
            }
        }));
    }

    let _libview = libraryviewstore::new(&pool.clone(), &builder, &gui.clone());
    let _albumview = albumviewstore::new(&pool.clone(), &builder, &gui.clone());
    let _plmview = playlist_manager::new(pool.clone(), &builder, gui.clone());

    window.maximize();
    window.set_application(Some(application));
    window.set_title("Viola");
    window.connect_delete_event(clone!(window, gui, pool => move |_, _| {
        gui.save(&pool);
        window.destroy();
        Inhibit(false)
    }));

    window.show_all();
    info!("Restoring tabs");
    gui.restore(&pool);
}
