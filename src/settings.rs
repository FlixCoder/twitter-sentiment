//! Configuration module

use config::{ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

/// This app's configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
	/// Webserver bind address and port
	pub bind: String,
	/// Words to track via Twitter streams
	pub track_tweets: Vec<String>,
	/// Default alpha value for exponential moving average
	pub default_alpha: f64,
}

impl Settings {
	/// Read configuration from `config.yaml` by default. Calls [`read_from`]
	#[inline]
	pub fn read() -> Result<Self, ConfigError> {
		Self::read_from("config.yaml")
	}

	/// Read configuration from specified file and merge in environment variable
	/// configuration.
	pub fn read_from(cfg_path: &str) -> Result<Self, ConfigError> {
		let mut config = config::Config::default();
		// config.set_default("key", "value")?;

		config.merge(File::with_name(cfg_path))?;
		config.merge(Environment::with_prefix("app"))?;

		config.try_into()
	}
}
