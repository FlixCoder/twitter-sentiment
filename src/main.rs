use std::{sync::Arc, time::Duration};

use color_eyre::Result;
use futures::future::try_join_all;
use rust_bert::pipelines::sentiment::{SentimentConfig, SentimentModel};
use tokio::{
	runtime::Runtime,
	task::{self, LocalSet},
};
use twitter_sentiment::*;

// TODO:
// - add tracing + logging
// - graceful shutdown
// - more data methods
// - better HTML views
// - add lints
// - add pre-commit?
// - clean up? (add traits for things?)

fn main() -> Result<()> {
	color_eyre::install()?;
	dotenv::dotenv()?;

	let config = Settings::read()?;
	let server_addr = config.bind.parse()?;
	let token = twitter_access_token()?;

	// must be created outside of the async runtime :(
	let sentiment_classifier = SentimentModel::new(SentimentConfig::default())?;
	let db = Arc::new(SentimentDB::open(&config.db_path)?);

	let twitter_streams = TwitterStreamRunner::builder()
		.streams(config.track_tweets.as_slice())
		.token(token)
		.sentiment_classifier(Arc::new(sentiment_classifier))
		.db(db.clone())
		.build()?;

	let server =
		Server::builder().bind(server_addr).db(db.clone()).config(Arc::new(config)).build()?;

	Runtime::new()?.block_on(async {
		let handles = vec![
			task::spawn(db.regular_autosaver(Duration::from_secs(5 * 60))),
			task::spawn(server.run()),
		];

		LocalSet::new().run_until(twitter_streams.run()).await?;

		for res in try_join_all(handles).await? {
			res?;
		}
		Ok(())
	})
}
