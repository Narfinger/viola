use dbus::{BusType, Connection, NameFlag, Path};
use dbus::tree::{DataType, Factory, Interface, MTFn, MethodErr};
use dbus::arg;
use dbus::tree;
use std::thread;
use std;
use gtk;
use std::rc::Rc;
use std::sync::Arc;
use std::fmt;

use dbus_mpris_player::{OrgMprisMediaPlayer2Player, org_mpris_media_player2_player_server};
use dbus_mpris_mediaplayer::{OrgMprisMediaPlayer2, org_mpris_media_player2_server};
use gstreamer_wrapper::{GStreamerAction, GStreamerExt};
use types::*;

struct DBusAdapter {
    gstreamer: GStreamerPtr,
}

impl std::fmt::Debug for DBusAdapter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Some gstreamer")
    }
}

#[derive(Copy, Clone, Default, Debug)]
struct TData;
impl tree::DataType for TData {
    type Tree = ();
    type ObjectPath = Arc<DBusAdapter>;
    type Property = ();
    type Interface = ();
    type Method = ();
    type Signal = ();
}

impl OrgMprisMediaPlayer2Player for DBusAdapter {
    type Err = MethodErr;

    fn next(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Next);
        Ok(())
    }

    fn previous(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Previous);
        Ok(())
    }

    fn pause(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Pausing);
        Ok(())
    }

    fn play_pause(&self) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Next);
        Ok(())
    }

    fn stop(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Next);
        Ok(())
    }

    fn play(&self) -> Result<(), Self::Err> {
        self.gstreamer.do_gstreamer_action(&GStreamerAction::Playing);
        Ok(())
    }

    fn seek(&self, offset: i64) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn set_position(&self, track_id: Path, position: i64) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn open_uri(&self, uri: &str) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_playback_status(&self) -> Result<String, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        panic!("not yet implemented");
        Ok(String::from("NOTHING"))
    }

    fn get_loop_status(&self) -> Result<String, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(String::from("NOTHING"))
    }

    fn set_loop_status(&self, value: String) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_rate(&self) -> Result<f64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0.0)
    }

    fn set_rate(&self, value: f64) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_shuffle(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(false)
    }

    fn set_shuffle(&self, value: bool) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_metadata(&self) -> Result<std::collections::HashMap<String, arg::Variant<Box<arg::RefArg>>>, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        let hm = std::collections::HashMap::new();
        panic!("not yet implemented");
        Ok(hm)
    }

    fn get_volume(&self) -> Result<f64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0.0)
    }

    fn set_volume(&self, value: f64) -> Result<(), Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(())
    }

    fn get_position(&self) -> Result<i64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0)
    }

    fn get_minimum_rate(&self) -> Result<f64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0.0)
    }

    fn get_maximum_rate(&self) -> Result<f64, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(0.0)
    }

    fn get_can_go_next(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(true)
    }

    fn get_can_go_previous(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(true)
    }

    fn get_can_play(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(true)
    }

    fn get_can_pause(&self) -> Result<bool, Self::Err> {
        //self.gstreamer.do_gstreamer_action(GStreamerAction::Playing);
        Ok(true)
    }

    fn get_can_seek(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

    fn get_can_control(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

}

struct DBusAdapter2 {
    gstreamer: GStreamerPtr,
}

impl std::fmt::Debug for DBusAdapter2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Some gstreamer")
    }
}

#[derive(Copy, Clone, Default, Debug)]
struct TData2;
impl tree::DataType for TData2 {
    type Tree = ();
    type ObjectPath = Arc<DBusAdapter2>;
    type Property = ();
    type Interface = ();
    type Method = ();
    type Signal = ();
}


impl OrgMprisMediaPlayer2 for DBusAdapter2 {
    type Err = MethodErr;
    fn raise(&self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn quit(&self) -> Result<(), Self::Err> {
        Ok(())
    }

    fn get_can_quit(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

    fn get_fullscreen(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

    fn set_fullscreen(&self, value: bool) -> Result<(), Self::Err> {
        Ok(())
    }

    fn get_can_set_fullscreen(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

    fn get_can_raise(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

    fn get_has_track_list(&self) -> Result<bool, Self::Err> {
        Ok(false)
    }

    fn get_identity(&self) -> Result<String, Self::Err> {
        Ok(String::from("Viola"))
    }

    fn get_desktop_entry(&self) -> Result<String, Self::Err> {
        Ok(String::from(""))
    }

    fn get_supported_uri_schemes(&self) -> Result<Vec<String>, Self::Err> {
        Ok(Vec::new())
    }

    fn get_supported_mime_types(&self) -> Result<Vec<String>, Self::Err> {
        Ok(Vec::new())
    }

}

fn create_tree(device: &Arc<DBusAdapter>, iface: &Arc<Interface<MTFn<TData>, TData>>) -> tree::Tree<MTFn<TData>, TData> {
    let f = tree::Factory::new_fn();
    let mut tree = f.tree(());
    tree = tree.add(f.object_path("/Player", device.clone())
        .introspectable()
        .add(iface.clone())
        );
    tree 
}

fn create_tree2(device: &Arc<DBusAdapter2>, iface: &Arc<Interface<MTFn<TData2>, TData2>>) -> tree::Tree<MTFn<TData2>, TData2> {
    let f = tree::Factory::new_fn();
    let mut tree = f.tree(());
    tree = tree.add(f.object_path("/org/viola", device.clone())
        .introspectable()
        .add(iface.clone())
        );
    tree 
}


fn setup_single(gstreamer: GStreamerPtr) -> Result<(),String> {
    println!("setuing up dbus");
    let tree = {
        let dbusadapter = Arc::new(DBusAdapter {gstreamer: gstreamer.clone() });
        let f = tree::Factory::new_fn();
        let iface = org_mpris_media_player2_player_server(&f, (), |m| {
            // Just provide a link from MethodInfo (m) to the &Device
            // we should call.
            let a: &Arc<DBusAdapter> = m.path.get_data();
            let b: &DBusAdapter = &a;
        b
        });
        create_tree(&dbusadapter, &Arc::new(iface))
    };

    let tree2 = {
        let dbusadapter2 = Arc::new(DBusAdapter2 {gstreamer: gstreamer});
        let f = tree::Factory::new_fn();
        let iface2 = org_mpris_media_player2_server(&f, (), |m| {
            let a: &Arc<DBusAdapter2> = m.path.get_data();
            let b: &DBusAdapter2 = &a;
            b
        });
        create_tree2(&dbusadapter2, &Arc::new(iface2))
    };
    let c = Connection::get_private(BusType::Session).map_err(|_| String::from("Error in getting bus"))?;
    c.register_name("org.viola", 0).map_err(|_| String::from("could not register"))?;
    tree.set_registered(&c, true).map_err(|_| String::from("Could not register to tree"))?;
    c.add_handler(tree);
    
    c.register_name("org.viola", 0).map_err(|_| String::from("could not register"))?;
    tree2.set_registered(&c, true).map_err(|_| String::from("Could not register to tree"))?;
    c.add_handler(tree2);
    
    gtk::idle_add(move || {
        c.incoming(10).next();
        gtk::Continue(true)
    });
    println!("Done setting up dbus");
    Ok(())
}

pub fn setup(gui: &MainGuiPtr) {
    let gst = gui.gstreamer.clone();
    setup_single(gst);
}



