// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::header::HashAlgorithm;
use sha2::{digest::FixedOutputReset, Digest, Sha256};
use std::cell::RefCell;

thread_local! {
	pub static SHA256: RefCell<Sha256> = RefCell::new(Sha256::new());
}

impl HashAlgorithm {
	pub fn hash(&self, data: &[u8]) -> Vec<u8> {
		match self {
			Self::Sha256 => SHA256.with(|hasher| {
				let mut hasher = hasher.borrow_mut();
				hasher.update(data);
				hasher.finalize_fixed_reset().to_vec()
			}),
		}
	}
}
