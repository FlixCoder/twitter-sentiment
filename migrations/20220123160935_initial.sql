CREATE TABLE tweet_sentiment (
	id BIGINT NOT NULL,
	keyword VARCHAR(101) NOT NULL,
	UNIQUE (id, keyword),
	created BIGINT NOT NULL,
	sentiment DOUBLE PRECISION NOT NULL
);
