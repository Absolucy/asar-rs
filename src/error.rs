// SPDX-License-Identifier: Apache-2.0 OR MIT
use serde::de::Error as DeError;
use serde_json::Error as JsonError;
use std::io::Error as IoError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
	#[error("I/O error: {0}")]
	Io(#[from] IoError),
	#[error("JSON error: {0}")]
	Json(#[from] JsonError),
	#[error("Archive is truncated")]
	Truncated,
}

impl Clone for Error {
	fn clone(&self) -> Self {
		match self {
			Self::Io(io_err) => Self::Io(IoError::new(io_err.kind(), io_err.to_string())),
			Self::Json(json_err) => Self::Json(JsonError::custom(json_err.to_string())),
			Self::Truncated => Self::Truncated,
		}
	}
}

impl PartialEq for Error {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Io(io_err), Self::Io(other_io_err)) => {
				io_err.kind() == other_io_err.kind()
					&& io_err.raw_os_error() == other_io_err.raw_os_error()
					&& io_err.to_string() == other_io_err.to_string()
			}
			(Self::Json(json_err), Self::Json(other_json_err)) => {
				json_err.line() == other_json_err.line()
					&& json_err.column() == other_json_err.column()
					&& json_err.classify() == other_json_err.classify()
					&& json_err.to_string() == other_json_err.to_string()
			}
			(Self::Truncated, Self::Truncated) => true,
			_ => false,
		}
	}
}

pub type Result<T> = std::result::Result<T, Error>;
