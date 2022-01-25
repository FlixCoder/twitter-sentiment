use std::sync::Arc;

use askama::Template;
use axum::{
	extract::{Extension, Path, Query},
	response::Html,
};
use serde::Deserialize;

use super::{error::ServerError, svg::Svg, templates};
use crate::{data, SentimentDB, Settings};

/// Shows page with list of keywords.
#[tracing::instrument(level = "debug", err, skip_all)]
pub async fn list_keywords(
	Extension(db): Extension<Arc<SentimentDB>>,
) -> Result<Html<String>, ServerError> {
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
	let entries = db.get(&keyword).await.map_err(ServerError::not_found)?;
	let alpha = params.alpha.unwrap_or(settings.default_alpha);
	let points = data::exp_moving_avg(&entries, alpha);

	let plot = poloto::plot("Sentiment - Exponential moving average", "Timestamp", "Sentiment")
		.ymarker(-1.0)
		.ymarker(1.0)
		.line(keyword, &points)
		.xinterval_fmt(data::timestamp_fmt)
		.move_into();
	let mut svg = String::new();
	poloto::simple_theme_dark(&mut svg, plot)?;

	Ok(Svg(svg))
}
