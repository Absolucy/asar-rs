// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::header::HashAlgorithm;
use sha2::{Digest, Sha256};

impl HashAlgorithm {
	pub fn hash(&self, data: &[u8]) -> Vec<u8> {
		match self {
			Self::Sha256 => {
				let mut hasher = Sha256::new();
				hasher.update(data);
				hasher.finalize().to_vec()
			}
		}
	}
}
