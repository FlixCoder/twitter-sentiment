use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Extension, Path, Query},
	response::Html,
};
use serde::Deserialize;
use tracing::info;

use super::{error::ServerError, svg::Svg, templates};
use crate::{data, SentimentDB, Settings};

/// Shows page with list of keywords.
#[tracing::instrument(level = "debug", err, skip_all)]
pub async fn list_keywords(
	Extension(db): Extension<Arc<SentimentDB>>,
) -> Result<Html<String>, ServerError> {
	info!("List of keywords is being retrieved.");

	let mut keywords = db.keywords().await?;
	keywords.sort();

	let keywords = templates::ListKeywords { keywords };
	Ok(Html(keywords.render()?))
}

#[derive(Debug, Deserialize)]
pub struct QueryAlpha {
	alpha: Option<f64>,
}

/// Responds with a SVG graph for the given keyword and parameters.
#[tracing::instrument(level = "debug", err, skip_all)]
pub async fn exp_moving_avg(
	Extension(db): Extension<Arc<SentimentDB>>,
	Extension(settings): Extension<Arc<Settings>>,
	Path(keyword): Path<String>,
	Query(params): Query<QueryAlpha>,
) -> Result<Svg, ServerError> {
	info!("SVG graph of exponential moving average is retrieved.");
	let alpha = params.alpha.unwrap_or(settings.web_defaults.alpha);

	let entries = db.get(&keyword).await.map_err(ServerError::not_found)?;
	let points = data::exp_moving_avg(&entries, alpha);

	let plot = data::plot("Sentiment - Exponential moving average", &keyword, &points)?;
	Ok(Svg(plot))
}

#[derive(Debug, Deserialize)]
pub struct QueryWindow {
	window: Option<usize>,
}

/// Responds with a SVG graph for the given keyword and parameters.
#[tracing::instrument(level = "debug", err, skip_all)]
pub async fn moving_avg(
	Extension(db): Extension<Arc<SentimentDB>>,
	Extension(settings): Extension<Arc<Settings>>,
	Path(keyword): Path<String>,
	Query(params): Query<QueryWindow>,
) -> Result<Svg, ServerError> {
	info!("SVG graph of moving average is retrieved.");
	let window = params.window.unwrap_or(settings.web_defaults.window);
	if window == 0 {
		return Err(ServerError::bad_request("Window size of 0 is not allowed!"));
	}

	let entries = db.get(&keyword).await.map_err(ServerError::not_found)?;
	let points = data::moving_avg(&entries, window);

	let plot = data::plot("Sentiment - Moving average", &keyword, &points)?;
	Ok(Svg(plot))
}
