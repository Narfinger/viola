use directories::ProjectDirs;

pub fn get_config_dir() -> Result<std::path::PathBuf, String> {
    ProjectDirs::from("com", "narfinger", "viola")
        .map(|p| p.config_dir().to_path_buf())
        .ok_or(String::from("Could not find config dir"))
}

pub fn get_config_file() -> Result<std::path::PathBuf, String> {
    get_config_dir().map(|p| p.join("viola_prefs.json"))
}

pub fn format_into_full_duration(i: i64) -> String {
    let mut s = String::new();
    let seconds = i % 60;
    let minutes = (i / 60) % 60;
    let hours = (i / (60 * 60)) % 24;
    let days = i / (60 * 60 * 24);

    if days > 0 {
        s.push_str(&format!("{}:", days));
    }
    if (hours > 0) | (days > 0) {
        s.push_str(&format!("{:02}:", hours));
    }
    if (minutes > 0) | (days > 0) | (hours > 0) {
        s.push_str(&format!("{:02}:", minutes));
    }
    s.push_str(&format!("{:02}", seconds));

    s
}
