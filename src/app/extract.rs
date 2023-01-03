// SPDX-License-Identifier: Apache-2.0 OR MIT
use super::args::ExtractArgs;
use asar::AsarReader;
use color_eyre::{eyre::WrapErr, Result};
use std::fs;

pub fn extract(args: ExtractArgs, read_unpacked: bool) -> Result<()> {
	let file = fs::read(&args.archive)
		.wrap_err_with(|| format!("failed to open archive {}", args.archive.display()))?;
	let asar_path = if read_unpacked {
		Some(args.archive)
	} else {
		None
	};
	let reader = AsarReader::new(&file, asar_path).wrap_err("failed to read archive")?;
	for (path, file) in reader.files() {
		let out_path = args.destination.join(path);
		if !out_path.starts_with(&args.destination) {
			panic!("asar archive attempted to escape destination");
		}
		fs::write(&out_path, file.data())
			.wrap_err_with(|| format!("failed to write file {}", out_path.display()))?;
	}

	Ok(())
}
