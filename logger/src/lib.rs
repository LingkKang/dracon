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
/// use log::LevelFilter::Debug;
///
/// fn main() {
///     Logger::new().set_level(Debug).init();
///     log::debug!("This is a debug message");
/// }
/// ```
///
/// Also, use the [`set_prefix()`] method to enable a prefix for the logger.
///
/// [`set_prefix()`]: #method.set_prefix
pub struct Logger {
    /// The default level of the logger.
    default_level: LevelFilter,

    /// Whether to use a prefix for the logger.
    with_prefix: bool,

    /// The prefix to be used for the logger.
    prefix: String,
}

/// Implement the default trait for the logger.
///
/// See [Clippy Lints :: new_without_default](https://rust-lang.github.io/rust-clippy/master/index.html#/new_without_default)
impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger {
    /// Create a new global logger with default level
    /// set to [`LevelFilter::Trace`].
    ///
    /// ``` rust
    /// use crate::logger::Logger;
    /// Logger::new().init();
    /// log::info!("Logger initialized");
    /// ```
    ///
    /// Remember to call [`init()`] method to initialize the logger
    /// after creating and configuring it.
    ///
    /// [`init()`]: #method.init
    pub fn new() -> Logger {
        Logger {
            default_level: LevelFilter::Trace,
            with_prefix: false,
            prefix: String::from("default"),
        }
    }

    /// Static method to initialize the logger
    /// with an optional logging level.
    ///
    /// If no level is set via [`set_level`], [`LevelFilter::Trace`] will
    /// be used as default level.
    ///
    /// ``` rust
    /// use crate::logger::Logger;
    /// Logger::new().init();
    /// log::info!("Logger initialized");
    /// ```
    ///
    /// [`set_level`]: #method.set_level
    pub fn init(self) {
        let level = self.default_level;
        log::set_max_level(level);
        log::set_boxed_logger(Box::new(self)).unwrap();
        log::trace!("Logger initialized with default level: {:?}", level);
    }

    /// Set the default level of the logger.
    ///
    /// ``` rust
    /// use crate::logger::Logger;
    /// use log::LevelFilter::Warn;
    /// Logger::new().set_level(Warn).init();
    /// log::info!("Logger initialized.");
    /// ```
    ///
    /// Remember to call [`init()`] method to initialize the logger
    /// after creating and configuring it.
    ///
    /// [`init()`]: #method.init
    pub fn set_level(mut self, level: LevelFilter) -> Logger {
        self.default_level = level;
        self
    }

    /// Enable the logger with a prefix after the time.
    ///
    /// ``` rust
    /// use crate::logger::Logger;
    /// use log::LevelFilter::Warn;
    /// let prefix = String::from("test_logger");
    /// Logger::new().set_prefix(prefix).init();
    /// log::info!("Logger initialized.");
    /// ```
    ///
    /// Remember to call [`init()`] method to initialize the logger
    /// after creating and configuring it.
    ///
    /// [`init()`]: #method.init
    pub fn set_prefix(mut self, prefix: String) -> Logger {
        self.with_prefix = true;
        self.prefix = prefix;
        self
    }

    /// Get the current level of the logger.
    pub fn get_level(&self) -> LevelFilter {
        self.default_level
    }

    /// Get the current prefix of the logger.
    /// If no prefix is set, it will return an empty string.
    pub fn get_prefix(&self) -> String {
        self.prefix.clone()
    }

    /// Get the current status of the prefix.
    /// If no prefix is set, it will return false.
    /// Otherwise, it will return true.
    pub fn is_using_prefix(&self) -> bool {
        self.with_prefix
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
            let message = match self.with_prefix {
                true => {
                    format!(
                        "{} {} {} {}",
                        time_str,
                        self.prefix.underline(),
                        colorize_level_string(level_str),
                        record.args()
                    )
                }
                false => {
                    format!(
                        "{} {} {}",
                        time_str,
                        colorize_level_string(level_str),
                        record.args()
                    )
                }
            };
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

    macro_rules! test_colorize_level_string_color {
        ($test_name:ident: $level:expr, $expected:expr) => {
            // As `concat_idents!` is not stable,
            // name of the test function should be typed manually.
            #[test]
            fn $test_name() {
                let level = $level.to_string();
                let level_in_color = colorize_level_string(level);
                assert_eq!(level_in_color.fgcolor(), Some($expected));
            }
        };
    }

    test_colorize_level_string_color!(test_color_error: "ERROR", Color::Red);
    test_colorize_level_string_color!(test_color_warn: "WARN", Color::Magenta);
    test_colorize_level_string_color!(test_color_info: "INFO", Color::Green);
    test_colorize_level_string_color!(test_color_debug: "DEBUG", Color::Cyan);

    macro_rules! test_plain_level {
        ($test_name:ident: $level:expr) => {
            #[test]
            fn $test_name() {
                let level = $level.to_string();
                let level_in_color = colorize_level_string(level);
                assert!(level_in_color.is_plain());
            }
        };
    }

    test_plain_level!(test_plain_trace: "TRACE");
    test_plain_level!(test_plain_random_text: "random_text");

    macro_rules! test_colorize_level_string_bold {
        ($test_name:ident: $level:expr) => {
            #[test]
            fn $test_name() {
                let level = $level.to_string();
                let level_in_color = colorize_level_string(level);
                assert!(level_in_color.style().contains(Styles::Bold));
            }
        };
    }

    test_colorize_level_string_bold!(test_bold_error: "ERROR");
    test_colorize_level_string_bold!(test_bold_warn: "WARN");
    test_colorize_level_string_bold!(test_bold_info: "INFO");

    #[test]
    fn test_logger_default_level() {
        let config = Logger::new();
        assert_eq!(config.get_level(), LevelFilter::Trace);
        assert!(!config.is_using_prefix());
    }

    #[test]
    fn test_logger_set_level() {
        let config = Logger::new().set_level(LevelFilter::Debug);
        assert_eq!(config.get_level(), LevelFilter::Debug);

        let config = Logger::new().set_level(LevelFilter::Warn);
        assert_eq!(config.get_level(), LevelFilter::Warn);
    }

    #[test]
    fn test_logger_set_prefix() {
        let prefix = String::from("test_logger");
        let config = Logger::new().set_prefix(prefix);
        assert_eq!(config.get_level(), LevelFilter::Trace);
        if config.is_using_prefix() {
            assert_eq!(config.get_prefix(), "test_logger");
        }
    }

    #[test]
    fn test_logger_init() {
        let prefix = String::from("test_logger");
        let level = LevelFilter::Debug;
        let config = Logger::new().set_level(level).set_prefix(prefix.clone());
        assert!(config.is_using_prefix());
        assert_eq!(config.get_level(), level);
        assert_eq!(config.get_prefix(), prefix);
        config.init();
        log::debug!("Logger initialized");
        assert_eq!(log::max_level(), level);
        log::info!("This is an info message");
    }
}
