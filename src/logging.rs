#![allow(unused_macros)]

/// An enum representing the available verbosity levels of the logger.
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
	/// Disable all our put messages
	///
	/// Designates without information
	Disabled = 0,
	/// The "error" level.
	///
	/// Designates very serious errors.
	Error,
	/// The "warn" level.
	///
	/// Designates hazardous situations.
	Warning,
	/// The "info" level.
	///
	/// Designates useful information.
	Info,
	// The "debug" level.
	///
	/// Designates lower priority information.
	Debug,
}

/// Data structures to filter kernel messages
pub struct KernelLogger {
	pub log_level: LogLevel,
}

/// default logger to handle kernel messages
pub const LOGGER: KernelLogger = KernelLogger {
	log_level: LogLevel::Info,
};

/// Print formatted info text to our console, followed by a newline.
#[macro_export]
macro_rules! info {
	($fmt:expr) => ({
		if $crate::logging::LOGGER.log_level >= $crate::logging::LogLevel::Info {
			println!(concat!("info: ", $fmt));
		}
	});
	($fmt:expr, $($arg:tt)*) => ({
		if $crate::logging::LOGGER.log_level >= $crate::logging::LogLevel::Info {
			println!(concat!("info: ", $fmt), $($arg)*);
		}
	});
}

/// Print formatted warnings to our console, followed by a newline.
#[macro_export]
macro_rules! warn {
	($fmt:expr) => ({
		if $crate::logging::LOGGER.log_level >= $crate::logging::LogLevel::Warning {
			println!(concat!("warning: ", $fmt));
		}
	});
	($fmt:expr, $($arg:tt)*) => ({
		if $crate::logging::LOGGER.log_level >= $crate::logging::LogLevel::Warning {
			println!(concat!("warning: ", $fmt), $($arg)*);
		}
	});
}

/// Print formatted warnings to our console, followed by a newline.
#[macro_export]
macro_rules! error {
	($fmt:expr) => ({
		if $crate::logging::LOGGER.log_level >= $crate::logging::LogLevel::Error {
			println!(concat!("error: ", $fmt));
		}
	});
	($fmt:expr, $($arg:tt)*) => ({
		if $crate::logging::LOGGER.log_level >= $crate::logging::LogLevel::Error {
			println!(concat!("error: ", $fmt), $($arg)*);
		}
	});
}

/// Print formatted debug messages to our console, followed by a newline.
#[macro_export]
macro_rules! debug {
	($fmt:expr) => ({
		if $crate::logging::LOGGER.log_level >= $crate::logging::LogLevel::Debug {
			println!(concat!("debug: ", $fmt));
		}
	});
	($fmt:expr, $($arg:tt)*) => ({
		if $crate::logging::LOGGER.log_level >= $crate::logging::LogLevel::Debug {
			println!(concat!("debug: ", $fmt), $($arg)*);
		}
	});
}
