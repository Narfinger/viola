use dbus::*;
use dbus::tree::Factory;
use std::rc::Rc;
use crate::gstreamer_wrapper::GStreamer;

struct DBusHandler {
    gstreamer: Rc<GStreamer>,
}

fn setup(gstreamer: Rc<GStreamer>) -> DBusHandler {
    let c = Connection::get_private(BusType::Session).expect("DBus error");
    c.register_name("com.example.dbustest", NameFlag::ReplaceExisting as u32).expect("DBus error");
    let f = Factory::new_fn::<()>();
    let tree = f.tree(()).add(f.object_path("/hello", ()).introspectable().add(
    f.interface("com.example.dbustest", ()).add_m(
        f.method("Hello", (), |m| {
            let n: &str = m.msg.read1()?;
            let s = format!("Hello {}!", n);
            Ok(vec!(m.msg.method_return().append1(s)))
        }).inarg::<&str,_>("name")
          .outarg::<&str,_>("reply")
    )
    ));
    tree.set_registered(&c, true).expect("DBus error");
    c.add_handler(tree);

    DBusHandler {
        gstreamer
    }
    //loop { c.incoming(1000).next(); }
}