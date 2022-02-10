//! Sentiment classification module

use std::{
	sync::mpsc,
	thread::{self, JoinHandle},
};

use color_eyre::{eyre::eyre, Result};
use rust_bert::pipelines::sentiment::{Sentiment, SentimentConfig, SentimentModel};
use tokio::{sync::oneshot, task};
use tracing::{info, warn};

/// Message type for internal channel, passing around texts and return value
/// senders
type Message = (Vec<String>, oneshot::Sender<Vec<Sentiment>>);

/// Runner for sentiment classification
#[derive(Debug, Clone)]
pub struct SentimentClassifier {
	sender: mpsc::SyncSender<Message>,
}

impl SentimentClassifier {
	/// Spawn a classifier on a separate thread and return a classifier instance
	/// to interact with it
	pub fn spawn() -> (JoinHandle<Result<()>>, SentimentClassifier) {
		let (sender, receiver) = mpsc::sync_channel(10);
		let handle = thread::spawn(move || Self::runner(receiver));
		(handle, SentimentClassifier { sender })
	}

	/// The classification runner itself
	#[tracing::instrument(level = "debug", err, skip_all)]
	fn runner(receiver: mpsc::Receiver<Message>) -> Result<()> {
		info!("Sentiment classifier runner starting.");
		// Needs to be in sync runtime, async doesn't work
		let model = SentimentModel::new(SentimentConfig::default())?;

		while let Ok((texts, sender)) = receiver.recv() {
			let texts: Vec<&str> = texts.iter().map(String::as_str).collect();
			let sentiments = model.predict(texts);
			let res = sender.send(sentiments);
			if let Err(_lost) = res {
				warn!("Sending sentiments results failed, receiver was closed!");
			}
		}

		info!("Sentiment classifier runner stopped.");
		Ok(())
	}

	/// Make the runner predict a sample and return the result
	#[tracing::instrument(level = "debug", err, skip_all)]
	pub async fn predict(&self, texts: Vec<String>) -> Result<Vec<Sentiment>> {
		let (sender, receiver) = oneshot::channel();
		task::block_in_place(|| self.sender.send((texts, sender)))
			.map_err(|err| eyre!("Sending mpsc message failed: {}", err))?;
		Ok(receiver.await?)
	}
}
