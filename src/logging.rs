#![allow(unused_macros)]

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
	Disabled = 0,
	Error,
	Warning,
	Info,
	Debug,
}

pub struct Logger {
	pub log_level: LogLevel,
}

pub const LOGGER: Logger = Logger {
	log_level: LogLevel::Info,
};

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
