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
#![allow(clippy::tabs_in_doc_comments)]

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
//! 	let (header, offset) = Header::read(&mut &asar_file[..])?;
//! 	let asar = AsarReader::new(header, offset, &asar_file)?;
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
//! 	let (header, offset) = Header::read(&mut &asar_file[..])?;
//! 	let asar = AsarReader::new(header, offset, &asar_file)?;
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

pub mod error;
pub mod header;
#[cfg(feature = "integrity")]
pub(crate) mod integrity;
pub mod reader;
#[cfg(feature = "write")]
pub mod writer;

pub use error::{Error, Result};
pub use header::{File, FileIntegrity, HashAlgorithm, Header};
pub use reader::AsarReader;
#[cfg(feature = "write")]
pub use writer::AsarWriter;
