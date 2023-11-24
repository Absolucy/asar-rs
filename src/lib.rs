// SPDX-License-Identifier: Apache-2.0 OR MIT
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![forbid(unsafe_code)]
#![warn(
	clippy::perf,
	clippy::complexity,
	clippy::style,
	clippy::correctness,
	clippy::missing_const_for_fn
)]
#![allow(clippy::tabs_in_doc_comments, clippy::too_many_arguments)]

//! This crate allows for the parsing, reading, and writing of [asar](https://github.com/electron/asar) archives,
//! often seen in [Electron](https://www.electronjs.org/)-based applications.
//!
//! # Examples
//!
//! ## Listing the contents of an asar archive
//! ```rust,no_run
//! use asar::{AsarReader, Header, Result};
//! use std::fs;
//!
//! fn main() -> Result<()> {
//! 	let asar_file = fs::read("archive.asar")?;
//! 	let asar = AsarReader::new(&asar_file, None)?;
//!
//! 	println!("There are {} files in archive.asar", asar.files().len());
//! 	for path in asar.files().keys() {
//! 		println!("{}", path.display());
//! 	}
//! 	Ok(())
//! }
//! ```
//!
//! ## Reading a file from an asar archive
//! ```rust,no_run
//! use asar::{AsarReader, Header, Result};
//! use std::{fs, path::PathBuf};
//!
//! fn main() -> Result<()> {
//! 	let asar_file = fs::read("archive.asar")?;
//! 	let asar = AsarReader::new(&asar_file, None)?;
//!
//! 	let path = PathBuf::from("hello.txt");
//! 	let file = asar.files().get(&path).unwrap();
//! 	let contents = std::str::from_utf8(file.data()).unwrap();
//! 	assert_eq!(contents, "Hello, World!");
//! 	Ok(())
//! }
//! ```
//!
//! ## Writing a file to an asar archive
//! ```rust,no_run
//! use asar::{AsarWriter, Result};
//! use std::fs::File;
//!
//! fn main() -> Result<()> {
//! 	let mut asar = AsarWriter::new();
//! 	asar.write_file("hello.txt", b"Hello, World!", false)?;
//! 	asar.finalize(File::create("archive.asar")?)?;
//! 	Ok(())
//! }
//! ```
//!
//! # Features
//!
//!  - `integrity`: Enable integrity checks/calculation.
//!  - `check-integrity-on-read`: Enable integrity checks when reading an
//!    archive, failing if any integrity check fails.
//!  - `write` - Enable writing an asar archive. **Enabled by default**, also
//!    enables `integrity`.
//!
//! # License
//!
//! `asar` is licensed under either the [MIT license](LICENSE-MIT) or the
//! [Apache License 2.0](LICENSE-APACHE), at the choice of the user.

/// Error handling for parsing, reading, and writing asar archives.
pub mod error;
/// Header parsing for asar archives.
pub mod header;
#[cfg(feature = "integrity")]
pub mod integrity;
/// Reading asar archives.
pub mod reader;
#[cfg(feature = "write")]
/// Writing asar archives.
pub mod writer;

pub use error::{Error, Result};
pub use header::{File, FileIntegrity, HashAlgorithm, Header};
pub use reader::AsarReader;
#[cfg(feature = "write")]
pub use writer::AsarWriter;
