use std::path::PathBuf;
use gtk;

use db::Track;
use types::DBPool;

#[derive(Clone, Debug)]
pub struct LoadedPlaylist {
    /// The id we have in the database for it. If none, means this was not yet saved
    pub id: Option<i32>,
    pub name: String,
    pub items: Vec<Track>,
    pub current_position: i32,
}

pub trait LoadedPlaylistExt {
    fn get_current_track(&self) -> &Track;
}

impl LoadedPlaylistExt for LoadedPlaylist {
    fn get_current_track(&self) -> &Track {
        &self.items[self.current_position as usize]
    }
}

pub trait PlaylistControls {
    fn get_current_path(&self) -> PathBuf;
    fn get_current_uri(&self) -> String;
    fn previous(&mut self) -> String;
    fn next(&mut self) -> String;
    fn set(&mut self, i32) -> String;
    fn next_or_eol(&mut self, &DBPool) -> Option<String>;
}

impl PlaylistControls for LoadedPlaylist {
    fn get_current_path(&self) -> PathBuf {
        let mut pb = PathBuf::new();
        pb.push(&self.items[self.current_position as usize].path);
        pb
    }

    fn get_current_uri(&self) -> String {
        println!("loading from playlist with name: {}", self.name);
        format!(
            "file:////{}",
            self.items[self.current_position as usize]
                .path
                .replace(" ", "%20")
        )
    }

    fn previous(&mut self) -> String {
        self.current_position -= 1 % self.items.len() as i32;
        self.get_current_uri()
    }

    fn next(&mut self) -> String {
        self.current_position += 1 % self.items.len() as i32;
        self.get_current_uri()
    }

    fn set(&mut self, i: i32) -> String {
        self.current_position = i;
        self.get_current_uri()
    }

    fn next_or_eol(&mut self, pool: &DBPool) -> Option<String> {
        {
            let track = self.items.get(self.current_position as usize);
            //update playlist counter
            let dbc = pool.clone();
            if let Some(t) = track {
                let id = t.id;
                gtk::idle_add(move ||
                    update_playcount(id, &dbc)
                );
            }
        }

        let next_pos = self.current_position + 1;

        if self.items.get(next_pos as usize).is_some() {
            Some(self.next())
        } else {
            self.current_position = 0;
            None
        }
    }
}

fn update_playcount(t_id: i32, pool: &DBPool) -> gtk::Continue {
    use schema::tracks::dsl::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SaveChangesDsl};
    let db = pool.get().unwrap();

    if let Ok(mut track) = tracks.filter(id.eq(t_id)).first::<Track>(&db) {
        track.playcount += 1;
        if let Err(_) =  track.save_changes::<Track>(&db) {
            println!("Some problem with updating play status (cannot update)");
        }
    } else {
        println!("Some problem with updating play status (gettin track)");
    }
    gtk::Continue(false)
}
