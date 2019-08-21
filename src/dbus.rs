use dbus::*;
use dbus::tree::Factory;
use std::rc::Rc;
use crate::gstreamer_wrapper::GStreamer;

struct DBusHandler {
    gstreamer: Rc<GStreamer>,
}

fn setup(gstreamer: Rc<GStreamer>) -> DBusHandler {
    let c = Connection::get_private(BusType::Session).expect("DBus error");
    c.register_name("org.mpris.MediaPlayer2.viola", NameFlag::ReplaceExisting as u32).expect("DBus error");
    let f = Factory::new_fn::<()>();
    let tree = f.tree(()).add(f.object_path("/org/mpris/MediaPlayer2", ()).introspectable().add(
    f.interface("org.mpris.MediaPlayer2", ())
    .add_m(f.method("Raise", (), |m| { Ok(vec!(m.msg.method_return())) }))
    .add_m(f.method("Quit", (), |m|  { Ok(vec!(m.msg.method_return())) }))
    ));
    tree.set_registered(&c, true).expect("DBus error");
    c.add_handler(tree);

    DBusHandler {
        gstreamer
    }
    //loop { c.incoming(1000).next(); }
}