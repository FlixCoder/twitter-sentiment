//! Server error handling and response

use std::fmt::Display;

use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub struct ServerError {
	status_code: StatusCode,
	error_msg: String,
}

impl Display for ServerError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Error {}: {}", self.status_code, self.error_msg)
	}
}

impl IntoResponse for ServerError {
	fn into_response(self) -> axum::response::Response {
		(self.status_code, self.error_msg).into_response()
	}
}

impl<E: std::error::Error> From<E> for ServerError {
	fn from(err: E) -> Self {
		ServerError { status_code: StatusCode::INTERNAL_SERVER_ERROR, error_msg: err.to_string() }
	}
}

impl ServerError {
	pub fn not_found<T: ToString>(msg: T) -> Self {
		Self { status_code: StatusCode::NOT_FOUND, error_msg: msg.to_string() }
	}

	pub fn bad_request<T: ToString>(msg: T) -> Self {
		Self { status_code: StatusCode::BAD_REQUEST, error_msg: msg.to_string() }
	}
}
