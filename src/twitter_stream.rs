//! Runner to receive the twitter streams and put sentiment data into the DB

use std::sync::{Arc, Mutex};

use color_eyre::Result;
use derive_builder::Builder;
use egg_mode::{search::ResultType, stream::StreamMessage, Token};
use futures::StreamExt;
use rust_bert::pipelines::sentiment::{Sentiment, SentimentModel, SentimentPolarity};
use tokio::task;

use crate::database::{self, SentimentDB};

fn sentiment_to_float(sentiment: &Sentiment) -> f64 {
	match sentiment.polarity {
		SentimentPolarity::Positive => sentiment.score,
		SentimentPolarity::Negative => -sentiment.score,
	}
}

/// Runner to receive the twitter streams and put sentiment data into the DB
#[derive(Builder)]
pub struct TwitterStreamRunner {
	#[builder(setter(into))]
	streams: Vec<String>,
	token: Token,
	// Needs mutex to be Send + Sync
	sentiment_classifier: Arc<Mutex<SentimentModel>>,
	db: Arc<SentimentDB>,
}

impl TwitterStreamRunner {
	pub fn builder() -> TwitterStreamRunnerBuilder {
		TwitterStreamRunnerBuilder::default()
	}

	/// Predict sentiment of some texts
	fn predict_sentiment<'a>(&self, texts: &[&'a str]) -> Vec<Sentiment> {
		task::block_in_place(|| {
			let classifier = self.sentiment_classifier.lock().expect("access sentiment classifier");
			classifier.predict(texts)
		})
	}

	/// If the database for a keyword is empty, fill it with some search results
	pub async fn save_search_results(&self) -> Result<()> {
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

			for tweet in tweets.response {
				let id = tweet.id;
				let created = tweet.created_at.timestamp();
				let sentiment = self.predict_sentiment(&[tweet.text.as_str()]);

				let entry = database::TweetSentiment::new(
					id,
					keyword.to_owned(),
					created,
					sentiment_to_float(&sentiment[0]),
				);

				self.db.insert(entry).await?;
			}
		}
		Ok(())
	}

	/// Listen to Twitter's tweet streams for the keywords and save the entries
	/// in the DB
	pub async fn run(self) -> Result<()> {
		self.save_search_results().await?;

		let mut stream =
			egg_mode::stream::filter().track(&self.streams).language(&["en"]).start(&self.token);
		while let Some(message) = stream.next().await {
			let message = message?;

			if let StreamMessage::Tweet(tweet) = message {
				let id = tweet.id;
				let created = tweet.created_at.timestamp();
				let text = tweet.text;

				for keyword in self.streams.iter().filter(|key| text.contains(*key)) {
					let sentiment = self.predict_sentiment(&[text.as_str()]);

					let entry = database::TweetSentiment::new(
						id,
						keyword.to_owned(),
						created,
						sentiment_to_float(&sentiment[0]),
					);

					self.db.insert(entry).await?;
				}
			}
		}

		Ok(())
	}
}
