// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::error::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as, DisplayFromStr};
use std::collections::HashMap;

#[cfg(test)]
pub(crate) static TEST_ASAR: &[u8] = include_bytes!("../data/test.asar");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Header {
	File(File),
	Directory { files: HashMap<String, Self> },
}

impl Header {
	/// Reads the header from a slice.
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
	/// The offset from the end of the header that this file is located at.
	#[inline]
	pub fn offset(&self) -> usize {
		self.offset
	}

	/// The total size of the file.
	#[inline]
	pub fn size(&self) -> usize {
		self.size
	}

	/// Whether this file is executable or not.
	#[inline]
	pub fn executable(&self) -> bool {
		self.executable
	}

	/// Integrity details of the file, such as hashes.
	#[inline]
	pub fn integrity(&self) -> &FileIntegrity {
		&self.integrity
	}
}

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
	/// The hashing algorithm used to calculate the hash.
	#[inline]
	pub fn algorithm(&self) -> HashAlgorithm {
		self.algorithm
	}

	/// The hash of the file
	#[inline]
	pub fn hash(&self) -> &[u8] {
		&self.hash
	}

	/// The size of each "block" to be hashed in a file.
	#[inline]
	pub fn block_size(&self) -> usize {
		self.block_size
	}

	/// The hash of each "block" in a file.
	#[inline]
	pub fn blocks(&self) -> &[Vec<u8>] {
		&self.blocks
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HashAlgorithm {
	/// The SHA-256 hashing algorithm
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
