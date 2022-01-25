use std::{env, sync::Arc};

use color_eyre::{eyre::eyre, Result};
use futures::future::try_join_all;
use sqlx::PgPool;
use tokio::task;
use tracing_subscriber::EnvFilter;
use twitter_sentiment::*;

// TODO:
// - add logging
// - proper error handling
// - add lints
// - clean up? (add traits for things?)
// - more graphs (e.g. number of tweets) + more data methods (moving average)
// - multiple keywords in a graph
// - better HTML views

#[tokio::main]
#[tracing::instrument(level = "debug", err, skip_all)]
async fn main() -> Result<()> {
	let config = Settings::read()?;
	let server_addr = config.bind.parse()?;

	let filter = EnvFilter::from_default_env()
		.add_directive(config.log_level.into())
		.add_directive("hyper=info".parse()?)
		.add_directive("sqlx=warn".parse()?);
	tracing_subscriber::fmt().with_env_filter(filter).init();

	color_eyre::install()?;
	dotenv::dotenv()?;

	// Init DB
	let db_url = env::var("DATABASE_URL")?;
	let db_pool = PgPool::connect(&db_url).await?;
	sqlx::migrate!().run(&db_pool).await?;
	let db = Arc::new(SentimentDB::new(db_pool));

	// Init Twitter listener
	let token = twitter_access_token()?;
	let (classifier_runner, sentiment_classifier) = SentimentClassifier::spawn();
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
	task::block_in_place(|| classifier_runner.join())
		.map_err(|_| eyre!("Join error on classifier thread!"))??;
	Ok(())
}
