//! Runner to receive the twitter streams and put sentiment data into the DB

use std::{sync::Arc, time::Duration};

use color_eyre::Result;
use derive_builder::Builder;
use egg_mode::{search::ResultType, stream::StreamMessage, tweet::Tweet, Token};
use futures::{future, TryStreamExt};
use rust_bert::pipelines::sentiment::{Sentiment, SentimentPolarity};
use tracing::{error, info, trace};

use crate::{
	database::{self, SentimentDB},
	settings::TwitterSettings,
	SentimentClassifier,
};

fn sentiment_to_float(sentiment: &Sentiment) -> f64 {
	match sentiment.polarity {
		SentimentPolarity::Positive => sentiment.score,
		SentimentPolarity::Negative => -sentiment.score,
	}
}

/// Runner to receive the twitter streams and put sentiment data into the DB
#[derive(Debug, Builder)]
pub struct TwitterStreamRunner {
	config: TwitterSettings,
	token: Token,
	sentiment_classifier: SentimentClassifier,
	db: Arc<SentimentDB>,
}

impl TwitterStreamRunner {
	/// Get a builder to create an instance.
	pub fn builder() -> TwitterStreamRunnerBuilder {
		TwitterStreamRunnerBuilder::default()
	}

	/// If the database for a keyword is empty, fill it with some search results
	#[tracing::instrument(level = "debug", err, skip_all)]
	pub async fn save_search_results(&self) -> Result<()> {
		info!("Retrieving Twitter search results.");
		for keyword in self.config.track_tweets.iter() {
			if self.db.exists(keyword).await? {
				continue;
			}

			let search = egg_mode::search::search("rust")
				.lang("en")
				.result_type(ResultType::Mixed)
				.count(10)
				.call(&self.token)
				.await?;
			let tweets =
				egg_mode::tweet::lookup(search.statuses.iter().map(|tweet| tweet.id), &self.token)
					.await?;

			let tweets = tweets.response;
			let sentiments = self.predict_sentiment(&tweets).await?;
			for (tweet, sentiment) in tweets.into_iter().zip(sentiments) {
				let id = tweet.id;
				let created = tweet.created_at.timestamp();

				let entry = database::TweetSentiment::new(
					id,
					keyword.to_owned(),
					created,
					sentiment_to_float(&sentiment),
				);

				self.db.insert(entry).await?;
			}
		}
		Ok(())
	}

	/// Runner for retrieving/receiving tweets from Twitter. Save search results
	/// for new keywords and listen to Twitter's tweet streams for the keywords
	/// and save the entries in the DB.
	#[tracing::instrument(level = "debug", err, skip_all)]
	pub async fn run(self) -> Result<()> {
		self.save_search_results().await?;

		while let Err(err) = self.internal_run().await {
			error!("Reconnecting soon after error in TwitterStreamRunner: {}", err);
			tokio::time::sleep(Duration::from_secs(self.config.secs_reconnect)).await;
		}

		Ok(())
	}

	/// Listen to Twitter's tweet streams for the keywords and save the entries
	/// in the DB
	#[tracing::instrument(level = "debug", err, skip_all)]
	async fn internal_run(&self) -> Result<()> {
		info!("Starting Twitter stream listener.");
		let stream = egg_mode::stream::filter()
			.track(&self.config.track_tweets)
			.language(&["en"])
			.start(&self.token);

		let keywords: Vec<String> =
			self.config.track_tweets.iter().map(|s| s.to_lowercase()).collect();

		stream
			.try_filter_map(|msg| {
				if let StreamMessage::Tweet(tweet) = msg {
					let text = tweet.text.to_lowercase();
					if keywords.iter().any(|keyword| text.contains(keyword)) {
						return future::ready(Ok(Some(tweet)));
					}
				}
				future::ready(Ok(None))
			})
			.try_chunks(self.config.chunk_size)
			.map_err(color_eyre::Report::from)
			.try_for_each_concurrent(self.config.concurrency, |tweets| async {
				trace!("New incoming Tweets: {}", tweets.len());

				let sentiments = self.predict_sentiment(&tweets).await?;
				for (tweet, sentiment) in tweets.into_iter().zip(sentiments) {
					let id = tweet.id;
					let created = tweet.created_at.timestamp();
					let text = tweet.text.to_lowercase();

					for keyword in keywords.iter().filter(|keyword| text.contains(*keyword)) {
						let entry = database::TweetSentiment::new(
							id,
							keyword.to_owned(),
							created,
							sentiment_to_float(&sentiment),
						);

						self.db.insert(entry).await?;
					}
				}
				Ok(())
			})
			.await?;

		info!("Twitter stream listener stopped.");
		Ok(())
	}

	/// Predict sentiment of some tweets
	async fn predict_sentiment(&self, tweets: &[Tweet]) -> Result<Vec<Sentiment>> {
		let texts = tweets.iter().map(|tweet| tweet.text.clone()).collect();
		self.sentiment_classifier.predict(texts).await
	}
}
