ALTER TABLE tweet_sentiment DROP CONSTRAINT tweet_sentiment_id_keyword_key;
ALTER TABLE tweet_sentiment ADD PRIMARY KEY (keyword, id);
