//! Webserver to serve the Twitter sentiment info

use std::{net::SocketAddr, sync::Arc};

use axum::{routing::get, AddExtensionLayer, Router};
use color_eyre::Result;
use derive_builder::Builder;

mod error;
mod routes;
mod svg;
mod templates;

use crate::{SentimentDB, Settings};

/// Webserver
#[derive(Debug, Clone, Builder)]
pub struct Server {
	/// Adress to bind the server to
	bind: SocketAddr,
	/// Handle to the database
	db: Arc<SentimentDB>,
	/// The app's configuration
	config: Arc<Settings>,
}

impl Server {
	/// Builder for the server
	pub fn builder() -> ServerBuilder {
		ServerBuilder::default()
	}

	/// Webserver routes
	fn routes() -> Router {
		Router::new()
			.route("/", get(routes::list_keywords))
			.route("/svg/:keyword/ema", get(routes::exp_moving_avg))
	}

	/// Run the webserver
	pub async fn run(self) -> Result<()> {
		let app = Self::routes()
			.layer(AddExtensionLayer::new(self.db))
			.layer(AddExtensionLayer::new(self.config));
		axum::Server::bind(&self.bind).serve(app.into_make_service()).await?;
		Ok(())
	}
}
