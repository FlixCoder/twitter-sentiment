//! Module for data persistence handling.

use sqlx::{FromRow, PgPool, Result};

/// Database entry for tweet sentiment.
#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct TweetSentiment {
	pub id: i64,
	pub keyword: String,
	pub created: i64,
	pub sentiment: f64,
}

impl TweetSentiment {
	/// Create new entry.
	pub fn new(id: u64, keyword: String, timestamp: i64, sentiment: f64) -> Self {
		TweetSentiment { id: id as i64, keyword, created: timestamp, sentiment }
	}

	/// Save an entry to the database
	async fn insert(self, db: &PgPool) -> Result<()> {
		sqlx::query!(
			r#"INSERT INTO tweet_sentiment
				(id, keyword, created, sentiment)
				VALUES ($1, $2, $3, $4)
			"#,
			self.id,
			self.keyword,
			self.created,
			self.sentiment
		)
		.execute(db)
		.await?;
		Ok(())
	}

	/// Get the entries for a given keyword
	async fn with_keyword(db: &PgPool, keyword: &str) -> Result<Vec<Self>> {
		let entries = sqlx::query_as!(
			TweetSentiment,
			r#"SELECT id, keyword, created, sentiment FROM tweet_sentiment
				WHERE keyword = $1
				ORDER BY created ASC
			"#,
			keyword
		)
		.fetch_all(db)
		.await?;
		Ok(entries)
	}
}

/// Database handler to share
#[derive(Debug)]
pub struct SentimentDB {
	pool: PgPool,
}

impl SentimentDB {
	/// Create ne DB interface for DB pool
	pub fn new(db: PgPool) -> Self {
		SentimentDB { pool: db }
	}

	/// Save an entry to the database
	pub async fn insert(&self, entry: TweetSentiment) -> Result<()> {
		entry.insert(&self.pool).await
	}

	/// Get the entries for a given keyword
	pub async fn get(&self, keyword: &str) -> Result<Vec<TweetSentiment>> {
		TweetSentiment::with_keyword(&self.pool, keyword).await
	}

	/// Checks if a given keyword exists in the database
	pub async fn exists(&self, keyword: &str) -> Result<bool> {
		let exists = sqlx::query_scalar!(
			r#"SELECT EXISTS (
				SELECT keyword FROM tweet_sentiment WHERE keyword = $1
			)"#,
			keyword
		)
		.fetch_one(&self.pool)
		.await?;
		Ok(exists.unwrap_or_default())
	}

	/// List all keywords in the database
	pub async fn keywords(&self) -> Result<Vec<String>> {
		let keywords = sqlx::query_scalar!(
			r#"SELECT DISTINCT keyword FROM tweet_sentiment ORDER BY keyword ASC"#
		)
		.fetch_all(&self.pool)
		.await?;
		Ok(keywords)
	}
}
