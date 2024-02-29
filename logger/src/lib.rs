//! Custom logger module.
//!
//! A custom logger implementation over [`log`] crate
//! that uses stdout to print logs.
//!
//! To make clear, [`log`] crate is a facade for the logging facilities in
//! Rust. And user ue its macros (such as [log::debug!] or [log::info!]) to
//! log messages. But [`log`] itself will not print out or write any logs.
//! Instead, it will use a specified logger implementation, such as
//! [`Logger`], to do so. It is the actual implementation that defines
//! format, style and destination of the logs.
//!
//! So for binary targets, both the [`log`] crate and the [`Logger`]
//! implementation should be added as dependencies.
//!
//! ``` toml
//! # Cargo.toml for binary targets
//! [dependencies]
//! log = {version = "0.4", features = ["std"]}
//! logger = { path = "/path/to/logger" }
//! ```
//!
//! However, for library targets, only the [`log`] crate should be added
//! as a dependency. This is because the log will not be generated until
//! it's used in a binary target. And the actual logger implementation used
//! in binary will be used in this scenario.
//!
//! So for a library target, this logger implementation is actually
//! not a dependency.
//!
//! ``` toml
//! # Cargo.toml for library targets
//! [dependencies]
//! log = {version = "0.4", features = ["std"]}
//! ```
//!
//! And similarly, doing logging in the library is as simple as using
//! the [`log`] crate.
//!
//! ``` rust
//! log::debug!("This is a debug message in a library");
//! ```
//!
//! Note that `log` crate should be set with `"sdt"` feature.
//! This apply both to library and binary targets.

mod time;
use time::get_formatted_time;

extern crate colored;
use colored::*;

extern crate log;
use log::{LevelFilter, Log, Metadata, Record};

/// A custom logger struct that uses stdout to print logs.
///
/// To use it in a binary target, first initialize the logger
/// with a logging level. Then use the log macros to log messages.
///
/// ``` rust
/// extern crate log;
/// use crate::logger::Logger;
/// fn main() {
///     Logger::init(Some(log::LevelFilter::Debug));
///     log::debug!("This is a debug message");
/// }
/// ```
pub struct Logger {
    /// The default level of the logger.
    default_level: LevelFilter,
}

impl Logger {
    /// Static method to initialize the logger
    /// with an optional logging level.
    pub fn init(level: Option<LevelFilter>) {
        // If the level is not specified, use `Trace` as default level.
        let record_level = level.unwrap_or(LevelFilter::Trace);
        log::set_max_level(record_level);
        let logger = Logger {
            default_level: record_level,
        };
        // Set the logger as the global logger.
        // Note that `log` crate should be set with "sdt" feature.
        // See Cargo.toml for more details.
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::trace!("Logger initialized with level: {:?}", record_level);
    }
}

impl Log for Logger {
    /// Check if current message level is enabled for logging.
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.default_level
    }

    /// Log the message.
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_str = {
                {
                    record.level().to_string().to_uppercase()
                }
            };
            let time_str = get_formatted_time();
            let message = format!(
                "{} {} {}",
                time_str,
                colorize_level_string(level_str),
                record.args()
            );
            println!("{}", message);
        }
    }

    /// Flush the logger.
    /// As stdout is used, no need to flush, so this method is empty.
    fn flush(&self) {}
}

fn colorize_level_string(level: String) -> colored::ColoredString {
    match level.as_str() {
        "ERROR" => level.red().bold(),
        "WARN" => level.magenta().bold(),
        "INFO" => level.green().bold(),
        "DEBUG" => level.cyan(),
        "TRACE" => level.normal(),
        _ => level.normal(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    #[test]
    fn test_colorize_level_string() {
        let level = "ERROR".to_string();
        let level_in_color = colorize_level_string(level);
        assert_eq!(level_in_color.fgcolor(), Some(Color::Red));
        assert!(level_in_color.style().contains(Styles::Bold));

        let random_text = "random".to_string();
        let random_text_in_color = colorize_level_string(random_text);
        assert!(random_text_in_color.is_plain());
    }

    #[test]
    fn test_color() {
        println!("{}", colorize_level_string("ERROR".to_string()));
        println!("{}", colorize_level_string("WARN".to_string()));
        println!("{}", colorize_level_string("INFO".to_string()));
        println!("{}", colorize_level_string("DEBUG".to_string()));
        println!("{}", colorize_level_string("TRACE".to_string()));
        println!("{}", colorize_level_string("random".to_string()));
    }

    #[test]
    fn test_logger_init() {
        Logger::init(Some(LevelFilter::Debug));
        assert_eq!(log::max_level(), LevelFilter::Debug);
    }
}
