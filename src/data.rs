//! Data retrieval and transformation helpers

use std::fmt::Formatter;

use time::OffsetDateTime;

use crate::database::TweetSentiment;

/// Format the timestamp steps in the graph
pub fn timestamp_fmt(
	f: &mut Formatter<'_>,
	timestamp: f64,
	_step_size: Option<f64>,
) -> std::fmt::Result {
	let time =
		OffsetDateTime::from_unix_timestamp(timestamp as i64).expect("timestamp to OffsetDateTime");
	write!(f, "{}", time.ordinal())
}

/// Transform a vector of entries to exponential moving average values.
///
/// Alpha defines the influence of the previous vs the new value:
/// $$ ema_{i+1} = alpha * ema_i + (1-alpha) * next $$
pub fn exp_moving_avg(entries: &[TweetSentiment], alpha: f64) -> Vec<(f64, f64)> {
	entries
		.iter()
		.scan(0.0, |ema, item| {
			*ema = alpha * *ema + (1.0 - alpha) * item.sentiment;
			Some((item.created as f64, *ema))
		})
		.collect()
}
