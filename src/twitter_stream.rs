//! Runner to receive the twitter streams and put sentiment data into the DB

use std::sync::Arc;

use color_eyre::Result;
use derive_builder::Builder;
use egg_mode::{search::ResultType, stream::StreamMessage, Token};
use futures::StreamExt;
use rust_bert::pipelines::sentiment::{Sentiment, SentimentModel, SentimentPolarity};

use crate::{
	database::{self},
	SentimentDB,
};

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
	sentiment_classifier: Arc<SentimentModel>,
	db: Arc<SentimentDB>,
}

impl TwitterStreamRunner {
	pub fn builder() -> TwitterStreamRunnerBuilder {
		TwitterStreamRunnerBuilder::default()
	}

	/// If the database for a keyword is empty, fill it with some search results
	pub async fn save_search_results(&self) -> Result<()> {
		for keyword in self.streams.iter() {
			if self.db.exists(keyword)? {
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
				let author = tweet.user.expect("no tweet author").name;
				let text = tweet.text;
				let sentiment = self.sentiment_classifier.predict(&[text.as_str()]);

				let mut entry =
					database::Entry::new(author, text, sentiment_to_float(&sentiment[0]));
				entry.set_timestamp(tweet.created_at.timestamp());

				self.db.insert(keyword, entry)?;
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
				let author = tweet.user.expect("no tweet author").name;
				let text = tweet.text;
				let keyword = self.streams.iter().find(|key| text.contains(*key));

				if let Some(keyword) = keyword {
					let sentiment = self.sentiment_classifier.predict(&[text.as_str()]);

					let mut entry =
						database::Entry::new(author, text, sentiment_to_float(&sentiment[0]));
					entry.set_timestamp(tweet.created_at.timestamp());

					self.db.insert(keyword, entry)?;
				}
			}
		}

		Ok(())
	}
}
