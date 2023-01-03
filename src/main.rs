// SPDX-License-Identifier: Apache-2.0 OR MIT
mod app;

use self::app::args::{AppArgs, AppSubcommand};
use clap::Parser;
use color_eyre::{eyre::WrapErr, Result};

fn main() -> Result<()> {
	color_eyre::install().wrap_err("failed to install color-eyre handler")?;
	let args = AppArgs::parse();

	match args.subcommand {
		AppSubcommand::Pack(subargs) => app::pack::pack(subargs).wrap_err("failed to pack archive"),
		AppSubcommand::List(subargs) => {
			app::list::list(subargs, args.read_unpacked).wrap_err("failed to list archive")
		}
		AppSubcommand::Extract(subargs) => {
			app::extract::extract(subargs, args.read_unpacked).wrap_err("failed to extract archive")
		}
		AppSubcommand::ExtractFile(subargs) => {
			app::extract_file::extract_file(subargs, args.read_unpacked)
				.wrap_err("failed to extract file from archive")
		}
	}
}
