use std::{env, sync::Arc};

use color_eyre::Result;
use futures::future;
use sqlx::PgPool;
use tokio::task;
use tracing_subscriber::EnvFilter;
use twitter_sentiment::*;

// TODO:
// - more graphs (e.g. number of tweets) + more data methods (e.g. indepedentent
//   of tweet number for higher performace)
// - multiple keywords in a graph
// - better HTML views
// - add tests

#[tokio::main]
#[tracing::instrument(level = "debug", err, skip_all)]
async fn main() -> Result<()> {
	let config = Arc::new(Settings::read()?);
	let server_addr = config.bind.parse()?;

	let filter = EnvFilter::from_default_env()
		.add_directive(config.log_level.into())
		.add_directive("hyper=info".parse()?)
		.add_directive("mio=info".parse()?)
		.add_directive("want=info".parse()?)
		.add_directive("sqlx=error".parse()?);
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
		.config(config.twitter.clone())
		.token(token)
		.sentiment_classifier(sentiment_classifier)
		.db(db.clone())
		.build()?;

	// Init webserver
	let server = Server::builder().bind(server_addr).db(db).config(config).build()?;

	// Run all tasks/jobs/runners
	let handles = vec![
		task::spawn(twitter_streams.run()),
		task::spawn(server.run()),
		task::spawn_blocking(move || {
			classifier_runner.join().expect("Join error on classifier thread!")
		}),
	];
	let (first_res, _, others) = future::select_all(handles).await;
	for handle in others {
		handle.abort();
	}
	first_res??;
	Ok(())
}
