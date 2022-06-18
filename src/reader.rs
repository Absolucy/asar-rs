// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::{
	error::{Error, Result},
	header::{FileIntegrity, Header},
};
use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
};

/// An AsarReader is a struct that takes an asar [`Header`] and its offset,
/// and reads the files specified in the header from the given byte buffer.
///
/// The lifetime of the [`AsarReader`] is tied to the lifetime of the byte
/// buffer that it reads from.
///
/// If the `check-integrity-on-read` feature is enabled, then the [`AsarReader`]
/// will check file integrity when reading an archive, and error out if any
/// integrity check fails.
///
/// ## Example
///
/// ```rust,no_run
/// use asar::{AsarReader, Header, Result};
/// use std::fs;
///
/// fn main() -> Result<()> {
/// 	let asar_file = fs::read("archive.asar")?;
/// 	let reader = AsarReader::new(&asar_file)?;
///
/// 	println!("There are {} files in archive.asar", reader.files().len());
/// 	Ok(())
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AsarReader<'a> {
	header: Header,
	directories: BTreeMap<PathBuf, Vec<PathBuf>>,
	files: BTreeMap<PathBuf, AsarFile<'a>>,
}

impl<'a> AsarReader<'a> {
	/// Parse and read an asar archive from a byte buffer.
	///
	/// ```rust,no_run
	/// use asar::{AsarReader, Header, Result};
	/// use std::fs;
	///
	/// fn main() -> Result<()> {
	/// 	let asar_file = fs::read("archive.asar")?;
	/// 	let asar = AsarReader::new(&asar_file)?;
	/// 	Ok(())
	/// }
	/// ```
	pub fn new(data: &'a [u8]) -> Result<Self> {
		let (header, offset) = Header::read(&mut &data[..])?;
		Self::new_from_header(header, offset, data)
	}

	/// Read an asar archive from a byte buffer, using the given header and
	/// offset.
	///
	/// ```rust,no_run
	/// use asar::{AsarReader, Header, Result};
	/// use std::fs;
	///
	/// fn main() -> Result<()> {
	/// 	let asar_file = fs::read("archive.asar")?;
	/// 	let (header, offset) = Header::read(&mut &asar_file[..])?;
	/// 	let asar = AsarReader::new_from_header(header, offset, &asar_file)?;
	/// 	Ok(())
	/// }
	/// ```
	pub fn new_from_header(header: Header, offset: usize, data: &'a [u8]) -> Result<Self> {
		let mut files = BTreeMap::new();
		let mut directories = BTreeMap::new();
		recursive_read(
			PathBuf::new(),
			&mut files,
			&mut directories,
			&header,
			offset,
			data,
		)?;
		Ok(Self {
			header,
			files,
			directories,
		})
	}

	/// Gets all files in the asar.
	#[inline]
	pub const fn files(&self) -> &BTreeMap<PathBuf, AsarFile<'a>> {
		&self.files
	}

	/// Gets all directories in the asar.
	#[inline]
	pub const fn directories(&self) -> &BTreeMap<PathBuf, Vec<PathBuf>> {
		&self.directories
	}

	/// Gets information about a file.
	#[inline]
	pub fn read(&self, path: &Path) -> Option<&AsarFile> {
		self.files.get(path)
	}

	/// Gets the contents of a directory.
	#[inline]
	pub fn read_dir(&self, path: &Path) -> Option<&[PathBuf]> {
		self.directories.get(path).map(|paths| paths.as_slice())
	}
}

/// This represents a file in an asar archive, with a byte slice referencing the
/// contents, and the integrity details containing file hashes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AsarFile<'a> {
	data: &'a [u8],
	integrity: FileIntegrity,
}

impl<'a> AsarFile<'a> {
	/// The data of the file.
	#[inline]
	pub const fn data(&self) -> &[u8] {
		self.data
	}

	/// Integrity details of the file, such as hashes.
	#[inline]
	pub const fn integrity(&self) -> &FileIntegrity {
		&self.integrity
	}
}

fn recursive_read<'a>(
	path: PathBuf,
	file_map: &mut BTreeMap<PathBuf, AsarFile<'a>>,
	dir_map: &mut BTreeMap<PathBuf, Vec<PathBuf>>,
	header: &Header,
	begin_offset: usize,
	data: &'a [u8],
) -> Result<()> {
	match header {
		Header::File(file) => {
			let start = begin_offset + file.offset();
			let end = start + file.size();
			if data.len() < end {
				println!(
					"file truncated path='{}', data_len={}, start={}, size={}, end={}",
					path.display(),
					data.len(),
					start,
					file.size(),
					end
				);
				return Err(Error::Truncated);
			}
			let data = &data[start..end];
			#[cfg(feature = "check-integrity-on-read")]
			{
				let integrity = file.integrity();
				let algorithm = integrity.algorithm();
				let block_size = integrity.block_size();
				let blocks = integrity.blocks();
				if block_size > 0 && !blocks.is_empty() {
					for (idx, (block, expected_hash)) in
						data.chunks(block_size).zip(blocks.iter()).enumerate()
					{
						let hash = algorithm.hash(block);
						if hash != *expected_hash {
							return Err(Error::HashMismatch {
								file: path,
								block: Some(idx + 1),
								expected: expected_hash.to_owned(),
								actual: hash,
							});
						}
					}
				}
				let hash = algorithm.hash(data);
				if hash != integrity.hash() {
					return Err(Error::HashMismatch {
						file: path,
						block: None,
						expected: integrity.hash().to_owned(),
						actual: hash,
					});
				}
			}
			file_map.insert(path, AsarFile {
				data,
				integrity: file.integrity().clone(),
			});
		}
		Header::Directory { files } => {
			for (name, header) in files {
				let file_path = path.join(name);
				dir_map
					.entry(path.clone())
					.or_default()
					.push(file_path.clone());
				recursive_read(file_path, file_map, dir_map, header, begin_offset, data)?;
			}
		}
	}
	Ok(())
}

#[cfg(test)]
pub mod test {
	use super::AsarReader;
	use crate::header::TEST_ASAR;
	use include_dir::{include_dir, Dir};

	static ASAR_CONTENTS: Dir = include_dir!("$CARGO_MANIFEST_DIR/data/contents");

	#[test]
	fn test_reading() {
		let reader = AsarReader::new(TEST_ASAR).expect("failed to read asar");
		for (path, file) in reader.files() {
			let real_file = ASAR_CONTENTS
				.get_file(path)
				.unwrap_or_else(|| panic!("test.asar contains invalid file {}", path.display()));
			let real_contents = real_file.contents();
			let asar_contents = file.data();
			assert_eq!(real_contents, asar_contents);
		}
	}
}
