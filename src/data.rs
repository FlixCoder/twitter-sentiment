//! Data retrieval and transformation helpers

use std::fmt::Formatter;

use time::OffsetDateTime;

use crate::database::TweetSentiment;

/// Make a plot of data points. Returns a string with a SVG.
pub fn plot(
	title: &str,
	line_name: &str,
	points: &[(f64, f64)],
) -> Result<String, std::fmt::Error> {
	let plot = poloto::plot(title, "Days (/ Timestamp)", "Sentiment")
		.ymarker(-1.0)
		.ymarker(1.0)
		.line(line_name, points)
		.xinterval_fmt(timestamp_fmt)
		.move_into();
	let mut svg = String::new();
	poloto::simple_theme_dark(&mut svg, plot)?;
	Ok(svg)
}

/// Format the timestamp steps in the graph.
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

/// Transform a vector of entries to moving average values with variable window
/// size.
pub fn moving_avg(entries: &[TweetSentiment], mut window: usize) -> Vec<(f64, f64)> {
	if entries.len() < 5 * window && window > 2 {
		// adjust window size for too small set, but must be at least 1
		window = (entries.len() / 5).max(1);
	}
	entries
		.windows(window)
		.map(|values| {
			let (sum_time, sum_val) = values.iter().fold((0, 0.0), |(time, value), item| {
				(time + item.created, value + item.sentiment)
			});
			(sum_time as f64 / window as f64, sum_val / window as f64)
		})
		.collect()
}
