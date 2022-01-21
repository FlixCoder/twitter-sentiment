//! Server error handling and response

use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub struct ServerError {
	status_code: StatusCode,
	error_msg: String,
}

impl IntoResponse for ServerError {
	fn into_response(self) -> axum::response::Response {
		(self.status_code, self.error_msg).into_response()
	}
}

impl<E: ToString> From<E> for ServerError {
	fn from(err: E) -> Self {
		ServerError { status_code: StatusCode::INTERNAL_SERVER_ERROR, error_msg: err.to_string() }
	}
}

impl ServerError {
	pub fn not_found<T: ToString>(msg: T) -> Self {
		Self { status_code: StatusCode::NOT_FOUND, error_msg: msg.to_string() }
	}
}
