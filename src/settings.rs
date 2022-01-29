//! Configuration module

use config::{ConfigError, Environment, File};
use serde::{de::Error, Deserialize, Deserializer};
use tracing::{metadata::ParseLevelError, Level};

/// This app's configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
	/// Webserver bind address and port
	pub bind: String,
	/// Logging level
	#[serde(deserialize_with = "deserialize_level")]
	pub log_level: Level,
	/// Words to track via Twitter streams
	pub track_tweets: Vec<String>,
	/// Defaults for server routes
	pub web_defaults: WebDefaults,
}

/// Defaults for webserver
#[derive(Debug, Clone, Deserialize)]
pub struct WebDefaults {
	/// Default alpha value for exponential moving average
	pub alpha: f64,
	/// Default window size for moving average
	pub window: usize,
}

/// Deserialize a Level
fn deserialize_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
	D: Deserializer<'de>,
{
	let log_level = String::deserialize(deserializer)?
		.parse()
		.map_err(|err: ParseLevelError| D::Error::custom(err.to_string()))?;
	Ok(log_level)
}

impl Settings {
	/// Read configuration from `config.yaml` by default. Calls `read_from`.
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
