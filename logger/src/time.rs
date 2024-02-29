//! Time utility for the logger.

extern crate chrono;
use chrono::{DateTime, Utc};

/// Get the current time in a formatted string.
/// Here `Z` means Zulu time, which is the same as UTC.
pub fn get_formatted_time() -> String {
    let local: DateTime<Utc> = Utc::now();
    local.format("%Y-%m-%d_%H:%M:%S%.6fZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_time_to_date() {
        println!("{}", get_formatted_time());
    }
}
