use std::path::PathBuf;
use db::Track;

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
    fn get_current_track<'a>(&'a self) -> &'a Track {
        &self.items[self.current_position as usize]
    }
}

pub trait PlaylistControls {
    fn get_current_path(&self) -> PathBuf;
    fn get_current_uri(&self) -> String;
    fn previous(&mut self) -> String;
    fn next(&mut self) -> String;
    fn set(&mut self, i32) -> String;
    fn next_or_eol(&mut self) -> Option<String>;
}

impl PlaylistControls for LoadedPlaylist {
    fn get_current_path(&self) -> PathBuf {
        let mut pb = PathBuf::new();
        pb.push(
            &self.items[self.current_position as usize]
            .path);
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

    fn next_or_eol(&mut self) -> Option<String> {
        if self.current_position >= self.items.len() as i32 {
            self.current_position = 0;
            None
        } else {
            Some(self.next())
        }
    }
}

