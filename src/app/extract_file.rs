// SPDX-License-Identifier: Apache-2.0 OR MIT
use super::args::ExtractFileArgs;
use asar::AsarReader;
use color_eyre::{
	eyre::{eyre, WrapErr},
	Result,
};
use std::{ffi::OsStr, fs, path::Path};

pub fn extract_file(args: ExtractFileArgs) -> Result<()> {
	let file = fs::read(&args.archive)
		.wrap_err_with(|| format!("failed to open archive {}", args.archive.display()))?;
	let reader = AsarReader::new(&file).wrap_err("failed to read archive")?;
	let path = args
		.filename
		.strip_prefix("/")
		.map(Path::to_path_buf)
		.unwrap_or_else(|_| args.filename.to_path_buf());
	let file_name = path
		.file_name()
		.map(OsStr::to_string_lossy)
		.ok_or_else(|| eyre!("failed to get file name for {}", path.display()))?
		.into_owned();
	let file = reader
		.files()
		.get(&path)
		.ok_or_else(|| eyre!("failed to find file {}", path.display()))?;

	fs::write(&file_name, file.data())
		.wrap_err_with(|| format!("failed to write contents to {file_name}"))?;

	Ok(())
}
