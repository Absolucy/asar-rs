// SPDX-License-Identifier: Apache-2.0 OR MIT
use super::args::ListArgs;
use asar::AsarReader;
use color_eyre::{eyre::WrapErr, Result};
use std::path::{PathBuf, MAIN_SEPARATOR};

pub fn list(args: ListArgs) -> Result<()> {
	let file = std::fs::read(&args.archive)
		.wrap_err_with(|| format!("failed to read archive {}", args.archive.display()))?;
	let reader = AsarReader::new(&file).wrap_err("failed to read archive")?;
	let root = PathBuf::from(MAIN_SEPARATOR.to_string());
	for path in reader.files().keys() {
		let path = root.join(path);
		println!("{}", path.display());
	}

	Ok(())
}
