use gdk;
use gio;
use gtk;
use gtk::prelude::*;
use std::string::String;
use std::ops::Deref;
use playlist::LoadedPlaylist;

use types::*;

/* fn signalhandler(pool: DBPool, plm: PlaylistManagerPtr, tv: &gtk::TreeView, event: &gdk::Event) {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use schema::playlists::dsl::*;
    use schema::playlisttracks::dsl::*;
    use schema::tracks::dsl::*;

    if event.get_event_type() == gdk::EventType::DoubleButtonPress {
        if let Ok(b) = event.clone().downcast::<gdk::EventButton>() {
            if b.get_button() == 1 {
                let (model, iter) = tv.get_selection().get_selected().unwrap();
                let artist_name = model.get_value(&iter, 0).get::<String>().unwrap();
                let db = pool.get().expect("DB problem");
                let results = tracks
                    .filter(artist.eq(&artist_name))
                    .order(path)
                    .load(db.deref())
                    .expect("Problem loading playlist");

                let pl = LoadedPlaylist {
                    id: None,
                    name: artist_name,
                    items: results,
                    current_position: 0,
                };

                plm.write().unwrap().put_playlist_in_GuiPtr(pl);
                /* println!("YEAH");
                let mut menu = gtk::Menu::new();
                let new_pl = gtk::MenuItem::new_with_label("New Playlist");
                menu.append(&new_pl);
                let new_pl2 = gtk::MenuItem::new_with_label("Yeah");
                menu.append(&new_pl2);

                //let (_, iter) = tv.get_selection().get_selected().unwrap();
                menu.attach(tv, 0, 0, 0, 0);
                menu.popup_easy(b.get_button(), b.get_time());
                //menu.show_all(); */
            }
        }
    }
}

pub fn connect(pool: DBPool, plm: PlaylistManagerPtr, tv: &gtk::TreeView) {
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

    tv.connect_event_after(move |s,e| { signalhandler(pool.clone(), plm.clone(), s, e) });
}
 */