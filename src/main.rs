pub mod playlist;

#[macro_use] extern crate error_chain;
extern crate gtk;
extern crate gstreamer;
extern crate taglib;
extern crate walkdir;

use std::sync::Mutex;
use std::sync::Arc;
use gstreamer::ElementExt;

use gtk::prelude::*;

error_chain! {
    foreign_links {
        GTK(gtk::Error);
    }
}

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

type CurrentPlaylist = Arc<Mutex<playlist::Playlist>>;
type Pipeline = Arc<Mutex<gstreamer::Element>>;
type Gui = Arc<Mutex<gtk::Builder>>;

/// poll the message bus and on eos start new
fn gstreamer_message_handler(pipeline: Pipeline, current_playlist: CurrentPlaylist) {
    let bus = {
        pipeline.lock().unwrap().get_bus().unwrap()
    };
    while let Some(msg) = bus.timed_pop(gstreamer::CLOCK_TIME_NONE) {
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
            },
            MessageView::Eos(..) => {
                let mut p = current_playlist.lock().unwrap();
                (*p).current_position = (*p).current_position +1;
                if (*p).current_position >= (*p).items.len() as i64{
                    (*p).current_position = 0;
                } else {
                    println!("Next should play");
                    let pl = pipeline.lock().unwrap();
                    (*pl).set_state(gstreamer::State::Ready);
                    (*pl).set_property("uri", &playlist::get_current_uri(&p));
                    (*pl).set_state(gstreamer::State::Playing);
                    println!("Next one now playing is: {}", &playlist::get_current_uri(&p));
                }
                println!("Eos found");
            },
            _ => (),
        }
    }
}

fn gstreamer_init(current_playlist: CurrentPlaylist) -> Result<Pipeline> {
    gstreamer::init().unwrap();
    let pipeline = gstreamer::parse_launch("playbin")?;
    let p = Arc::new(Mutex::new(pipeline));

    let pp = p.clone();
    let cp = current_playlist.clone();
    std::thread::spawn(|| {
        gstreamer_message_handler(pp, cp);
    });
    Ok(p)
}

/// General purpose function to update the gui on any change
fn update_gui(pipeline: Pipeline, playlist: CurrentPlaylist, gui: Gui) {
    let (_, state, _) = pipeline.lock().unwrap().get_state(gstreamer::ClockTime(Some(1000)));  
    let treeview: gtk::TreeView = gui.lock().unwrap().get_object("listview").unwrap();
    let treeselection = treeview.get_selection();
    if state == gstreamer::State::Paused || state == gstreamer::State::Playing {
        let index = playlist.lock().unwrap().current_position;
        let mut ipath = gtk::TreePath::new();
        ipath.append_index(index as i32);
        treeselection.select_path(&ipath);
    } else {
        treeselection.unselect_all();
    }
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let glade_src = include_str!("../ui/main.glade");
    let builder: Gui = Arc::new(Mutex::new(gtk::Builder::new_from_string(glade_src)));

    println!("Building list");
    let playlist = playlist::playlist_from_directory("/mnt/ssd-media/Musik/1rest/");
    let current_playlist = Arc::new(Mutex::new(playlist));
    println!("Done building list");
    
    let window: gtk::Window = builder.lock().unwrap().get_object("mainwindow").unwrap();
    let treeview: gtk::TreeView = builder.lock().unwrap().get_object("listview").unwrap();

    let pipeline = gstreamer_init(current_playlist.clone()).unwrap();

    
    { // Play Button
        let button: gtk::Button = builder.lock().unwrap().get_object("playButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline, builder => move |_| {
            {
                let p = pipeline.lock().unwrap();
                let pl = current_playlist.lock().unwrap();
                (*p).set_property("uri", &playlist::get_current_uri(&pl));
                p.set_state(gstreamer::State::Playing); 
            }
            update_gui(pipeline.clone(), current_playlist.clone(), builder.clone());
        }));
    }
    { // Pause Button
        let button: gtk::Button = builder.lock().unwrap().get_object("pauseButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline, builder  => move |_| {
            {
                let p = pipeline.lock().unwrap();      
                match p.get_state(gstreamer::ClockTime(Some(1000))) {
                    (_, gstreamer::State::Paused, _) =>  { (*p).set_state(gstreamer::State::Playing); },
                    (_, gstreamer::State::Playing, _) => { (*p).set_state(gstreamer::State::Paused);  },
                    (_, _, _) => {}
                }
            }
            update_gui(pipeline.clone(), current_playlist.clone(), builder.clone());
        }));
    }
    {  // Previous button
        let button: gtk::Button = builder.lock().unwrap().get_object("prevButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline, builder => move |_| {
            {
                let p = pipeline.lock().unwrap();
                let mut pl = current_playlist.lock().unwrap();
                (*p).set_state(gstreamer::State::Paused);
                (*p).set_state(gstreamer::State::Ready);
                (*pl).current_position = ((*pl).current_position -1) % (*pl).items.len() as i64;
                (*p).set_property("uri", &playlist::get_current_uri(&pl)).expect("Error in changing url");
                (*p).set_state(gstreamer::State::Playing);
            }
            update_gui(pipeline.clone(), current_playlist.clone(), builder.clone());
        }));
    }
    {  // Next button
        let button: gtk::Button = builder.lock().unwrap().get_object("nextButton").unwrap();
        button.connect_clicked(clone!(current_playlist, pipeline, builder => move |_| {
            {
                let p = pipeline.lock().unwrap();
                let mut pl = current_playlist.lock().unwrap();
                (*p).set_state(gstreamer::State::Paused);
                (*p).set_state(gstreamer::State::Ready);
                (*pl).current_position = ((*pl).current_position +1) % (*pl).items.len() as i64;
                (*p).set_property("uri", &playlist::get_current_uri(&pl)).expect("Error in changing url");
                (*p).set_state(gstreamer::State::Playing);
            }
            update_gui(pipeline.clone(), current_playlist.clone(), builder.clone());
        }));
    }

    let model = gtk::ListStore::new(&[u32::static_type(), String::static_type(), String::static_type(), String::static_type()]);
    
    {
        let p = current_playlist.lock().unwrap();
        for (i, entry) in p.items.iter().enumerate() {
            let taglibfile = taglib::File::new(entry);
                if let Err(e) = taglibfile {
                    println!("Error {:?}", e);
                } else {
                    let ataglib = taglibfile.unwrap();
                    let tags = ataglib.tag().unwrap();
                    model.insert_with_values(None, &[0,1,2,3], &[&(i as u32 + 1), &tags.title(), &tags.artist(), &tags.album()]);
                }
        }
        for id in vec![0,1,2,3] {
            let column = gtk::TreeViewColumn::new();
            let cell = gtk::CellRendererText::new();
            column.pack_start(&cell, true);
            // Association of the view's column with the model's `id` column.
            column.add_attribute(&cell, "text", id);
            treeview.append_column(&column);
        }
        treeview.set_model(Some(&model));
    }
    
    
    window.connect_delete_event(clone!(pipeline => move |_, _| {
        let p = pipeline.lock().unwrap();
        (*p).set_state(gstreamer::State::Null);
        gtk::main_quit();
        Inhibit(false)
    }));

    window.show_all();
    gtk::main();
}