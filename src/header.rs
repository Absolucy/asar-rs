// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::error::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as, DisplayFromStr};
use std::collections::HashMap;

#[cfg(test)]
pub(crate) static TEST_ASAR: &[u8] = include_bytes!("../data/test.asar");

/// The [`Header`] represents the data structure found in asar archives. It can
/// either be a [`File`], or a Directory containing other [`Header`]s.
///
/// It is a recursive structure, and a massive pain in the ass as a result. You
/// really don't want to manually mess with these — use
/// [`AsarReader`](crate::reader::AsarReader) instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Header {
	File(File),
	Directory { files: HashMap<String, Self> },
}

impl Header {
	pub(crate) fn new() -> Self {
		Self::Directory {
			files: HashMap::new(),
		}
	}

	/// Reads the header from a reader.
	/// ```rust,no_run
	/// use asar::{Header, Result};
	/// use std::fs;
	///
	/// fn main() -> Result<()> {
	/// 	let asar_file = fs::read("archive.asar")?;
	/// 	let (header, offset) = Header::read(&mut &asar_file[..])?;
	///
	/// 	println!("Header ends at offset {offset}");
	/// 	println!("Header: {header:#?}");
	/// 	Ok(())
	/// }
	/// ```
	pub fn read<Read: ReadBytesExt>(data: &mut Read) -> Result<(Self, usize)> {
		data.read_u32::<LittleEndian>()?; // magic number or something idk
		let header_size = data.read_u32::<LittleEndian>()? as usize;
		data.read_u32::<LittleEndian>()?;
		let json_size = data.read_u32::<LittleEndian>()? as usize;
		let mut bytes = vec![0_u8; json_size];
		data.read_exact(&mut bytes)?;
		Ok((serde_json::from_slice(&bytes)?, header_size + 8))
	}
}

/// This struct contains details about a file in an asar archive, such as
/// where it is located in the archive, its size, whether its executable or not,
/// and integrity details such as cryptographic hashes.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct File {
	/// The offset from the end of the header that this file is located at.
	#[serde_as(as = "DisplayFromStr")]
	offset: usize,
	/// The total size of the file.
	size: usize,
	/// Whether this file is executable or not.
	#[serde(skip_serializing_if = "is_false", default = "default_false")]
	executable: bool,
	/// Integrity details of the file, such as hashes.
	integrity: FileIntegrity,
}

impl File {
	pub(crate) const fn new(
		offset: usize,
		size: usize,
		executable: bool,
		integrity: FileIntegrity,
	) -> Self {
		Self {
			offset,
			size,
			executable,
			integrity,
		}
	}

	/// The offset from the end of the header that this file is located at.
	///
	/// Note that this is represented as a [`String`] in the JSON format,
	/// but we convert it to/from a [`usize`] when we read/write the JSON.
	#[inline]
	pub const fn offset(&self) -> usize {
		self.offset
	}

	/// The total size of the file, in bytes.
	#[inline]
	pub const fn size(&self) -> usize {
		self.size
	}

	/// Whether this file is executable or not.
	#[inline]
	pub const fn executable(&self) -> bool {
		self.executable
	}

	/// Integrity details of the file, such as hashes.
	#[inline]
	pub const fn integrity(&self) -> &FileIntegrity {
		&self.integrity
	}
}

/// This struct contains the integrity details of a file, such as
/// a hash of the file's contents, and hashes of "blocks" of the file, which is
/// split according to the `block_size` specified in it.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIntegrity {
	/// The hashing algorithm used to calculate the hash.
	algorithm: HashAlgorithm,
	/// The hash of the file, in hex format.
	#[serde_as(as = "Hex")]
	hash: Vec<u8>,
	/// The size of each "block" to be hashed in a file.
	block_size: usize,
	/// The hash of each "block" in a file.
	#[serde_as(as = "Vec<Hex>")]
	blocks: Vec<Vec<u8>>,
}

impl FileIntegrity {
	pub(crate) fn new(
		algorithm: HashAlgorithm,
		hash: Vec<u8>,
		block_size: usize,
		blocks: Vec<Vec<u8>>,
	) -> Self {
		Self {
			algorithm,
			hash,
			block_size,
			blocks,
		}
	}

	/// The hashing algorithm used to calculate the hash.
	#[inline]
	pub const fn algorithm(&self) -> HashAlgorithm {
		self.algorithm
	}

	/// The hash of the file.
	#[inline]
	pub fn hash(&self) -> &[u8] {
		&self.hash
	}

	/// The size of each "block" to be hashed in a file.
	///
	/// Defaults to 4 MiB.
	#[inline]
	pub const fn block_size(&self) -> usize {
		self.block_size
	}

	/// The hash of each "block" in a file.
	#[inline]
	pub fn blocks(&self) -> &[Vec<u8>] {
		&self.blocks
	}
}

/// This struct specifies which cryptographic hashing algorithm is used to
/// calculate the hash of a file in the archive.
///
/// Currently, only [SHA-256](https://en.wikipedia.org/wiki/SHA-2) is supported.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HashAlgorithm {
	/// The [SHA-256](https://en.wikipedia.org/wiki/SHA-2) hashing algorithm
	#[serde(rename = "SHA256")]
	Sha256,
}

const fn is_false(b: &bool) -> bool {
	!*b
}

const fn default_false() -> bool {
	false
}

#[cfg(test)]
mod test {
	use super::{Header, TEST_ASAR};

	static TEST_ASAR_JSON: &str = include_str!("../data/test.asar.json");

	#[test]
	pub fn test_read() {
		let mut asar = TEST_ASAR;
		let (header, _) = Header::read(&mut asar).expect("failed to read header");
		let expected =
			serde_json::from_str::<Header>(TEST_ASAR_JSON).expect("failed to decode expected");
		assert_eq!(header, expected);
	}
}
