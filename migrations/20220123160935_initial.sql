CREATE TABLE tweet_sentiment (
	id BIGINT PRIMARY KEY NOT NULL,
	keyword VARCHAR(101) NOT NULL,
	created BIGINT NOT NULL,
	sentiment DOUBLE PRECISION NOT NULL
);