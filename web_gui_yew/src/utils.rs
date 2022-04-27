use std::time::Duration;

use humantime::format_duration;

pub(crate) fn format_time(i: u64) -> String {
    format_duration(Duration::from_secs(i))
        .to_string()
        .replace(' ', "")
}
