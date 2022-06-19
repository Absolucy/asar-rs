// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::header::HashAlgorithm;
use sha2::{
	digest::{typenum::Unsigned, Digest, OutputSizeUser},
	Sha256,
};

impl HashAlgorithm {
	/// Returns the length of the output of the hash function.
	///
	/// ## Example
	/// ```rust
	/// use asar::HashAlgorithm;
	///
	/// assert_eq!(HashAlgorithm::Sha256.hash_len(), 32);
	/// ```
	pub const fn hash_len(&self) -> usize {
		match self {
			HashAlgorithm::Sha256 => <Sha256 as OutputSizeUser>::OutputSize::USIZE,
		}
	}

	/// Hashes the given data and returns the hash.
	///
	/// ## Example
	/// ```rust
	/// use asar::HashAlgorithm;
	///
	/// let data = b"A common mistake that people make when trying to design something completely foolproof is to underestimate the ingenuity of complete fools.";
	/// let hash = HashAlgorithm::Sha256.hash(data);
	/// assert_eq!(hash, b"\x4f\x71\x68\x29\xf5\xd2\x95\xcb\x1a\x24\x33\xb5\x99\x39\xa3\xcf\xf7\x77\x2a\x9c\xb9\x13\x2c\x63\xbe\x56\x10\xfe\x52\x08\x65\x90");
	/// ```
	pub fn hash(&self, data: &[u8]) -> Vec<u8> {
		match self {
			Self::Sha256 => {
				let mut hasher = Sha256::new();
				hasher.update(data);
				hasher.finalize().to_vec()
			}
		}
	}

	/// Splits the given data into blocks of a specific size, and hashes each
	/// block.
	///
	/// ## Example
	/// ```rust
	/// use asar::HashAlgorithm;
	///
	/// let data = "The ships hung in the sky in much the same way that bricks don't.";
	/// let hash = HashAlgorithm::Sha256.hash_blocks(25, data.as_bytes());
	/// assert_eq!(
	/// 	hash,
	/// 	vec![
	/// 		b"\x9d\x84\xeb\x91\x5a\x78\x2c\xc7\x2e\x74\x6d\x41\x62\x59\xe2\x28\xa2\x79\x03\x04\xf7\x6a\xa4\x20\x03\x3c\xf4\x50\xd7\x84\x26\x6c",
	/// 		b"\xdf\x78\xe6\x17\x28\xb6\x61\x8c\x55\x82\xb9\x00\x41\x96\x31\x2c\x24\x85\xe5\x83\xc2\x7b\xba\x8e\x2c\xbb\x1c\x36\x6f\x1a\x73\xad",
	/// 		b"\x7f\xda\x3f\x7b\x0e\x6d\x11\xc0\x61\x23\xff\x52\xd6\x10\xe1\xc3\xa3\xb7\x17\x22\xc0\x8b\xef\x0d\x96\x77\xc0\x46\x1c\x83\xf2\x4e"
	/// 	]
	/// );
	/// ```
	pub fn hash_blocks(&self, block_size: usize, data: &[u8]) -> Vec<Vec<u8>> {
		let mut blocks = Vec::with_capacity((0..data.len()).step_by(block_size).size_hint().0);
		data.chunks(block_size).for_each(|block| {
			let hash = self.hash(block);
			blocks.push(hash);
		});
		blocks
	}
}
