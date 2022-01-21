use std::env;

use color_eyre::Result;
use egg_mode::{KeyPair, Token};

mod data;
mod database;
mod server;
mod settings;
mod twitter_stream;

pub use database::SentimentDB;
pub use server::Server;
pub use settings::Settings;
pub use twitter_stream::TwitterStreamRunner;

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
