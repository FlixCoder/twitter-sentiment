#![deny(trivial_casts, trivial_numeric_casts, unused_extern_crates)]
#![warn(missing_debug_implementations, unused_qualifications, missing_docs, dead_code)]

//! All essential functionality used for this service, split into modules.
//!
//! - The webserver is in `server`.
//! - Database access is defined in `database`.
//! - Sentiment classification is in `classifier`.
//! - Data handling and transformation is in `data`.
//! - Settings are in `settings`.

mod classifier;
mod data;
mod database;
mod server;
mod settings;
mod twitter_stream;

use std::env;

use color_eyre::Result;
use egg_mode::{KeyPair, Token};

pub use self::{
	classifier::SentimentClassifier, database::SentimentDB, server::Server, settings::Settings,
	twitter_stream::TwitterStreamRunner,
};

/// Obtains a bearer token to use for egg-mode from the secrets in the env
/// variables
pub async fn twitter_app_token() -> Result<Token> {
	let con_key = env::var("TWITTER_API_KEY")?;
	let con_secret = env::var("TWITTER_API_KEY_SECRET")?;
	let keys = KeyPair::new(con_key, con_secret);
	let token = egg_mode::auth::bearer_token(&keys).await?;
	Ok(token)
}

/// Returns the bearer token from the env variables
pub fn twitter_token() -> Result<Token> {
	let bearer = env::var("TWITTER_BEARER_TOKEN")?;
	let token = Token::Bearer(bearer);
	Ok(token)
}

/// Returns the twitter full read-only access token from the env variables
pub fn twitter_access_token() -> Result<Token> {
	let con_key = env::var("TWITTER_API_KEY")?;
	let con_secret = env::var("TWITTER_API_KEY_SECRET")?;
	let consumer = KeyPair::new(con_key, con_secret);

	let access_key = env::var("TWITTER_ACCESS_TOKEN")?;
	let access_secret = env::var("TWITTER_ACCESS_TOKEN_SECRET")?;
	let access = KeyPair::new(access_key, access_secret);

	let token = Token::Access { consumer, access };
	Ok(token)
}
