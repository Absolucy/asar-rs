// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::header::HashAlgorithm;
use sha2::{
	digest::{typenum::Unsigned, Digest, OutputSizeUser},
	Sha256,
};

impl HashAlgorithm {
	pub const fn hash_len(&self) -> usize {
		match self {
			HashAlgorithm::Sha256 => <Sha256 as OutputSizeUser>::OutputSize::USIZE,
		}
	}

	pub fn hash(&self, data: &[u8]) -> Vec<u8> {
		match self {
			Self::Sha256 => {
				let mut hasher = Sha256::new();
				hasher.update(data);
				hasher.finalize().to_vec()
			}
		}
	}

	pub fn hash_blocks(&self, block_size: usize, data: &[u8]) -> Vec<Vec<u8>> {
		let mut blocks = Vec::with_capacity((0..data.len()).step_by(block_size).size_hint().0);
		data.chunks(block_size).for_each(|block| {
			let hash = self.hash(block);
			blocks.push(hash);
		});
		blocks
	}
}
