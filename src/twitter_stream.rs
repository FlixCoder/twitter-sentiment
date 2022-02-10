//! Runner to receive the twitter streams and put sentiment data into the DB

use std::{sync::Arc, time::Duration};

use color_eyre::Result;
use derive_builder::Builder;
use egg_mode::{search::ResultType, stream::StreamMessage, tweet::Tweet, Token};
use futures::TryStreamExt;
use rust_bert::pipelines::sentiment::{Sentiment, SentimentPolarity};
use tracing::{error, info, trace};

use crate::{
	database::{self, SentimentDB},
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
	#[builder(setter(into))]
	streams: Vec<String>,
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
		for keyword in self.streams.iter() {
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
			tokio::time::sleep(Duration::from_secs(120)).await;
		}

		Ok(())
	}

	/// Listen to Twitter's tweet streams for the keywords and save the entries
	/// in the DB
	#[tracing::instrument(level = "debug", err, skip_all)]
	async fn internal_run(&self) -> Result<()> {
		info!("Starting Twitter stream listener.");
		let stream =
			egg_mode::stream::filter().track(&self.streams).language(&["en"]).start(&self.token);

		stream
			.try_filter_map(|tweet| async move {
				if let StreamMessage::Tweet(inner) = tweet {
					Ok(Some(inner))
				} else {
					Ok(None)
				}
			})
			.try_chunks(16)
			.map_err(color_eyre::Report::from)
			.try_for_each_concurrent(3, |tweets| async move {
				trace!("New Tweets incoming: {}", tweets.len());

				let sentiments = self.predict_sentiment(&tweets).await?;
				for (tweet, sentiment) in tweets.into_iter().zip(sentiments) {
					let id = tweet.id;
					let created = tweet.created_at.timestamp();
					let text = tweet.text;

					for keyword in self
						.streams
						.iter()
						.filter(|key| text.to_lowercase().contains(&key.to_lowercase()))
					{
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
