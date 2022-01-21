//! An SVG-wrapper to send an SVG as HTTP response with proper Content-Type
//! header

use axum::response::{Headers, IntoResponse};

/// An SVG-wrapper to send an SVG as HTTP response with proper Content-Type
/// header
pub struct Svg(pub String);

impl IntoResponse for Svg {
	fn into_response(self) -> axum::response::Response {
		let headers = Headers(vec![("content-type", "image/svg+xml")]);
		(headers, self.0).into_response()
	}
}
