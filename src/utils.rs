use directories::ProjectDirs;
use std::fmt::Write;
use std::fs::File;

pub(crate) fn get_config_dir() -> Result<std::path::PathBuf, String> {
    ProjectDirs::from("com", "narfinger", "viola")
        .map(|p| p.config_dir().to_path_buf())
        .ok_or_else(|| String::from("Could not find config dir"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConfigWriteMode {
    Read,
    Write,
}

pub(crate) fn get_config_file(mode: &ConfigWriteMode) -> Result<File, String> {
    info!("Settings file with {:?}", mode);
    get_config_dir()
        .map(|p| p.join("viola_prefs.json"))
        .and_then(|f| {
            if mode == &ConfigWriteMode::Write {
                File::create(f)
            } else {
                File::open(f)
            }
            .map_err(|_| String::from("Could not open file"))
        })
}

#[must_use]
pub(crate) fn format_into_full_duration(i: i64) -> String {
    let mut s = String::new();
    let seconds = i % 60;
    let minutes = (i / 60) % 60;
    let hours = (i / (60 * 60)) % 24;
    let days = i / (60 * 60 * 24);

    if days > 0 {
        write!(s, "{}:", days).expect("Error in conversion");
    }
    if (hours > 0) | (days > 0) {
        write!(s, "{:02}:", hours).expect("Error in conversion");
    }
    if (minutes > 0) | (days > 0) | (hours > 0) {
        write!(s, "{:02}:", minutes).expect("Error in conversion");
    }
    write!(s, "{:02}", seconds).expect("Error in conversion");

    s
}
