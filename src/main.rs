use std::{
	env,
	sync::{Arc, Mutex},
};

use color_eyre::Result;
use futures::future::try_join_all;
use rust_bert::pipelines::sentiment::{SentimentConfig, SentimentModel};
use sqlx::PgPool;
use tokio::{runtime::Runtime, task};
use twitter_sentiment::*;

// TODO:
// - add tracing + logging
// - more graphs (e.g. number of tweets) + more data methods (moving average)
// - multiple keywords in a graph
// - better HTML views
// - add lints
// - clean up? (add traits for things?)

fn main() -> Result<()> {
	let config = Settings::read()?;
	let server_addr = config.bind.parse()?;

	color_eyre::install()?;
	dotenv::dotenv()?;

	// must be created outside of the async runtime :(
	let sentiment_classifier =
		Arc::new(Mutex::new(SentimentModel::new(SentimentConfig::default())?));

	Runtime::new()?.block_on(async {
		// Init DB
		let db_url = env::var("DATABASE_URL")?;
		let db_pool = PgPool::connect(&db_url).await?;
		sqlx::migrate!().run(&db_pool).await?;
		let db = Arc::new(SentimentDB::new(db_pool));

		// Init Twitter listener
		let token = twitter_access_token()?;
		let twitter_streams = TwitterStreamRunner::builder()
			.streams(config.track_tweets.as_slice())
			.token(token)
			.sentiment_classifier(sentiment_classifier)
			.db(db.clone())
			.build()?;

		// Init webserver
		let server =
			Server::builder().bind(server_addr).db(db.clone()).config(Arc::new(config)).build()?;

		// Run all tasks/jobs/runners
		let handles = vec![task::spawn(twitter_streams.run()), task::spawn(server.run())];
		for res in try_join_all(handles).await? {
			res?;
		}
		Ok(())
	})
}
