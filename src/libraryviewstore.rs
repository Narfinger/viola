use gdk;
use gio;
use gtk;
use gtk::prelude::*;
use std::ops::Deref;
use types::*;

fn signalhandler(tv: &gtk::TreeView, event: &gdk::Event) {
    if event.get_event_type() == gdk::EventType::ButtonPress {
        if let Ok(b) = event.clone().downcast::<gdk::EventButton>() {
            if b.get_button() == 3 {
                println!("YEAH");
                let mut menu = gtk::Menu::new();
                let new_pl = gtk::MenuItem::new_with_label("New Playlist");
                menu.append(&new_pl);
                let new_pl2 = gtk::MenuItem::new_with_label("Yeah");
                menu.append(&new_pl2);

                //let (_, iter) = tv.get_selection().get_selected().unwrap();
                menu.attach(tv, 0, 0, 0, 0);
                menu.popup_easy(b.get_button(), b.get_time());
                //menu.show_all();
            }
        }
    }
}

pub fn connect(pool: DBPool, tv: &gtk::TreeView) {
    use diesel::{ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl};
    use schema::tracks::dsl::*;

    let model = gtk::TreeStore::new(&[String::static_type()]);
    let db = pool.get().unwrap();
    let res: Vec<String> = tracks
        .select(artist)
        .order(artist)
        .group_by(artist)
        .load(db.deref())
        .expect("Error in db connection");

    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", 0);
    tv.append_column(&column);

    println!("Running");
    for i in res {
        let st: String = i.chars().take(20).collect();
        model.insert_with_values(None, None, &[0], &[&st]);
    }

    tv.set_model(Some(&model));
    println!("Stopped");

    tv.connect_event_after(signalhandler);
}
