// SPDX-License-Identifier: Apache-2.0 OR MIT
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(author, version, about, long_about = None, propagate_version = true)]
pub struct AppArgs {
	#[clap(subcommand)]
	pub subcommand: AppSubcommand,
}

#[derive(Subcommand)]
pub enum AppSubcommand {
	Pack(PackArgs),
	List(ListArgs),
	Extract(ExtractArgs),
	ExtractFile(ExtractFileArgs),
}

/// Create asar archive
#[derive(Args)]
pub struct PackArgs {
	/// Path to a text file for ordering contents
	#[clap(long)]
	pub ordering: Option<PathBuf>,
	/// Do not pack files matching glob <expression>
	#[clap(long)]
	pub unpack: Option<String>,
	/// Do not pack dirs matching glob <expression> or starting with literal
	/// <expression>
	#[clap(long)]
	pub unpack_dir: Option<String>,
	/// Exclude hidden files
	#[clap(long)]
	pub exclude_hidden: bool,
	/// The directory to pack
	#[clap(value_parser)]
	pub dir: PathBuf,
	/// The output asar archive
	#[clap(value_parser)]
	pub output: PathBuf,
}

/// List files of asar archive
#[derive(Args)]
pub struct ListArgs {
	/// The asar archive to list
	#[clap(value_parser)]
	pub archive: PathBuf,
}

/// Extract an asar archive
#[derive(Args)]
pub struct ExtractArgs {
	/// Archive to extract
	#[clap(value_parser)]
	pub archive: PathBuf,
	/// The directory to extract to
	#[clap(value_parser)]
	pub destination: PathBuf,
}

/// Extract one file from an asar archive
#[derive(Args)]
pub struct ExtractFileArgs {
	/// Archive to extract
	#[clap(value_parser)]
	pub archive: PathBuf,
	/// The file to extract from the archive
	#[clap(value_parser)]
	pub filename: PathBuf,
}
