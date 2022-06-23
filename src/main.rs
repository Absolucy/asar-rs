// SPDX-License-Identifier: Apache-2.0 OR MIT
mod app;

use self::app::args::{AppArgs, AppSubcommand};
use clap::Parser;
use color_eyre::{eyre::WrapErr, Result};

fn main() -> Result<()> {
	color_eyre::install().wrap_err("failed to install color-eyre handler")?;
	let args = AppArgs::parse();

	match args.subcommand {
		AppSubcommand::Pack(args) => app::pack::pack(args).wrap_err("failed to pack archive"),
		AppSubcommand::List(args) => app::list::list(args).wrap_err("failed to list archive"),
		AppSubcommand::Extract(args) => {
			app::extract::extract(args).wrap_err("failed to extract archive")
		}
		_ => unimplemented!(),
	}
}
