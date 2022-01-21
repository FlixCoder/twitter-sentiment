//! Database module.
//!
//! Bad internal file-save should be replaced by a proper database!

use std::{
	collections::HashMap,
	fs,
	path::{Path, PathBuf},
	sync::{Arc, RwLock},
	time::Duration,
};

use bincode::{config::Configuration, Decode, Encode};
use color_eyre::{
	eyre::{bail, eyre},
	Result,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

const BIN_CONF: Configuration = Configuration::standard();

/// Database entry
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Entry {
	pub timestamp: i64,
	pub author: String,
	pub tweet: String,
	pub sentiment: f64,
}

impl Entry {
	pub fn new(author: impl Into<String>, tweet: impl Into<String>, sentiment: f64) -> Self {
		let timestamp = OffsetDateTime::now_utc().unix_timestamp();
		Entry { timestamp, author: author.into(), tweet: tweet.into(), sentiment }
	}

	pub fn set_timestamp(&mut self, timestamp: i64) -> &mut Self {
		self.timestamp = timestamp;
		self
	}
}

/// Database
#[derive(Debug, Encode, Decode, Clone, Default)]
struct DB {
	/// For every tracked keyword, there is a list of entries
	pub entries: HashMap<String, Vec<Entry>>,
}

/// Sentiment database.
#[derive(Debug)]
pub struct SentimentDB {
	db: RwLock<DB>,
	path: PathBuf,
}

impl SentimentDB {
	/// Open the DB
	pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
		let db = if path.as_ref().is_file() {
			let mut file = fs::File::open(path.as_ref())?;
			bincode::decode_from_std_read(&mut file, BIN_CONF)?
		} else {
			DB::default()
		};
		Ok(Self { db: RwLock::new(db), path: path.as_ref().into() })
	}

	/// Write DB to file
	pub fn write(&self) -> Result<()> {
		let db = self.db.read().map_err(|err| eyre!("Lock error: {}", err))?;
		let mut file = fs::File::create(&self.path)?;
		bincode::encode_into_std_write(&*db, &mut file, BIN_CONF)?;
		Ok(())
	}

	/// Write database data at a fixed interval
	pub async fn regular_autosaver(self: Arc<Self>, interval: Duration) -> Result<()> {
		let mut interval = tokio::time::interval(interval);
		loop {
			interval.tick().await;
			self.write()?;
		}
	}

	/// Insert an entry
	pub fn insert(&self, keyword: impl Into<String>, entry: Entry) -> Result<()> {
		let mut db = self.db.write().map_err(|err| eyre!("Lock error: {}", err))?;
		db.entries.entry(keyword.into()).or_default().push(entry);
		Ok(())
	}

	/// Checks if a given keyword exists in the database
	pub fn exists(&self, keyword: &str) -> Result<bool> {
		let db = self.db.read().map_err(|err| eyre!("Lock error: {}", err))?;
		Ok(db.entries.contains_key(keyword))
	}

	/// Sort the entries by timestamp for the given keyword
	pub fn sort(&self, keyword: &str) -> Result<()> {
		let mut db = self.db.write().map_err(|err| eyre!("Lock error: {}", err))?;
		if let Some(vec) = db.entries.get_mut(keyword) {
			vec.sort_unstable_by_key(|entry| entry.timestamp);
		}
		Ok(())
	}

	/// Get the entries for a given keyword
	pub fn get(&self, keyword: &str) -> Result<Vec<Entry>> {
		self.sort(keyword)?;
		let db = self.db.read().map_err(|err| eyre!("Lock error: {}", err))?;
		if let Some(entries) = db.entries.get(keyword) {
			Ok(entries.clone())
		} else {
			bail!("No entries found for {}!", keyword);
		}
	}

	/// List all keywords in the database
	pub fn keywords(&self) -> Result<Vec<String>> {
		let db = self.db.read().map_err(|err| eyre!("Lock error: {}", err))?;
		let keywords = db.entries.keys().cloned().collect();
		Ok(keywords)
	}
}

impl Drop for SentimentDB {
	fn drop(&mut self) {
		self.write().expect("saving DB while dropping");
	}
}
