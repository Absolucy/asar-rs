// SPDX-License-Identifier: Apache-2.0 OR MIT
use super::args::PackArgs;
use asar::AsarWriter;
use color_eyre::{
	eyre::{eyre, WrapErr},
	Result,
};
use std::{
	fs::{self, File},
	io::BufWriter,
};
use walkdir::WalkDir;
use wax::{Glob, Pattern};

pub fn pack(args: PackArgs) -> Result<()> {
	let mut asar = AsarWriter::new();
	let unpack = args
		.unpack
		.as_deref()
		.map(Glob::new)
		.transpose()
		.map_err(|err| eyre!("failed to parse --unpack glob: {}", err))?;
	let unpack_dir = args
		.unpack_dir
		.as_deref()
		.map(Glob::new)
		.transpose()
		.map_err(|err| eyre!("failed to parse --unpack-dir glob: {}", err))?;
	for entry in WalkDir::new(&args.dir) {
		let entry = entry.wrap_err("failed to get directory entry")?;
		let path = entry.path();
		if !path.is_file() {
			continue;
		}
		let stripped_path = path.strip_prefix(&args.dir).wrap_err_with(|| {
			format!(
				"'{}' is not a prefix of '{}'",
				args.dir.display(),
				path.display()
			)
		})?;
		let file_name = path
			.file_name()
			.ok_or_else(|| eyre!("failed to get file name for {}", path.display()))?
			.to_string_lossy();
		if args.exclude_hidden && file_name.starts_with('.') {
			continue;
		}
		if let (Some(parent), Some(unpack_dir_glob)) = (stripped_path.parent(), &unpack_dir) {
			if unpack_dir_glob.is_match(parent) {
				continue;
			}
		}
		if let Some(unpack_glob) = &unpack {
			if unpack_glob.is_match(stripped_path) {
				continue;
			}
		}

		if path.is_symlink() {
			let link = std::fs::read_link(path).unwrap();
			let stripped_link = link.strip_prefix(&args.dir).wrap_err_with(|| {
				format!(
					"'{}' is not a prefix of '{}'",
					args.dir.display(),
					link.display()
				)
			})?;
			asar.write_symlink(stripped_path, stripped_link)
				.wrap_err_with(|| format!("failed to write {} to asar", path.display()))?;
			continue;
		}

		let file = fs::read(path).wrap_err_with(|| format!("failed to read {}", path.display()))?;

		asar.write_file(stripped_path, &file, is_executable::is_executable(path))
			.wrap_err_with(|| format!("failed to write {} to asar", path.display()))?;
	}

	let mut out = BufWriter::new(
		File::create(&args.output)
			.wrap_err_with(|| format!("failed to create {}", args.output.display()))?,
	);
	asar.finalize(&mut out)
		.wrap_err_with(|| format!("failed to write asar to {}", args.output.display()))?;
	out.into_inner()
		.wrap_err("failed to de-buf writer")?
		.sync_all()
		.wrap_err_with(|| format!("failed to sync {} to disk", args.output.display()))?;

	Ok(())
}
