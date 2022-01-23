use std::{env, sync::Arc};

use color_eyre::Result;
use futures::future::try_join_all;
use rust_bert::pipelines::sentiment::{SentimentConfig, SentimentModel};
use sqlx::PgPool;
use tokio::{
	runtime::Runtime,
	task::{self, LocalSet},
};
use twitter_sentiment::*;

// TODO:
// - use real DB and docker
// - add tracing + logging
// - graceful shutdown
// - more data methods
// - multiple keywords in a graph
// - better HTML views
// - add lints
// - add pre-commit?
// - clean up? (add traits for things?)

fn main() -> Result<()> {
	let config = Settings::read()?;
	let server_addr = config.bind.parse()?;

	color_eyre::install()?;
	dotenv::dotenv()?;

	// must be created outside of the async runtime :(
	let sentiment_classifier = SentimentModel::new(SentimentConfig::default())?;

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
			.sentiment_classifier(Arc::new(sentiment_classifier))
			.db(db.clone())
			.build()?;

		// Init webserver
		let server =
			Server::builder().bind(server_addr).db(db.clone()).config(Arc::new(config)).build()?;

		// Run all tasks/jobs/runners
		let handles = vec![task::spawn(server.run())];
		LocalSet::new().run_until(twitter_streams.run()).await?;
		for res in try_join_all(handles).await? {
			res?;
		}
		Ok(())
	})
}
