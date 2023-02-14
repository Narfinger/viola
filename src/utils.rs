use directories::ProjectDirs;
use log::info;
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
