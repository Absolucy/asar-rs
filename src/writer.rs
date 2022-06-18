// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::{
	error::{Error, Result},
	header::{File, FileIntegrity, HashAlgorithm, Header},
	reader::AsarReader,
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::{
	collections::{HashMap, VecDeque},
	io::Write,
	path::{Component, Path, PathBuf},
};

const BLOCK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB

pub struct AsarWriter {
	files: HashMap<PathBuf, File>,
	buffer: Vec<u8>,
	offset: usize,
	hasher: HashAlgorithm,
}

impl AsarWriter {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_from_reader(&mut self, reader: &AsarReader) -> Result<()> {
		for (path, file) in reader.files() {
			self.write_file(path, file.data(), false)?;
		}
		Ok(())
	}

	/// Write a file to the archive.
	/// This appends the contents to the writer, adds the file to the header,
	/// and updates the offset.
	pub fn write_file(
		&mut self,
		path: impl AsRef<Path>,
		bytes: impl AsRef<[u8]>,
		executable: bool,
	) -> Result<()> {
		self.write_file_impl(path.as_ref(), bytes.as_ref(), executable)
	}

	fn write_file_impl(&mut self, path: &Path, bytes: &[u8], executable: bool) -> Result<()> {
		if self.files.contains_key(path) {
			return Err(Error::FileAlreadyWritten(path.to_path_buf()));
		}
		let file = File::new(
			self.offset,
			bytes.len(),
			executable,
			FileIntegrity::new(
				self.hasher,
				self.hasher.hash(bytes),
				BLOCK_SIZE,
				self.hasher.hash_blocks(BLOCK_SIZE, bytes),
			),
		);
		self.buffer.extend_from_slice(bytes);
		self.offset += bytes.len();
		self.files.insert(path.to_path_buf(), file);
		Ok(())
	}

	/// Finalizes the archive, writing the header + files to the writer.
	pub fn finalize<FinalWriter>(self, mut final_writer: FinalWriter) -> Result<usize>
	where
		FinalWriter: Write,
	{
		let mut header = Header::new();
		for (path, file) in self.files {
			let path = path_to_reverse_components(&path)?;
			recursive_add_to_header(path, file, &mut header);
		}
		let mut written = 0;
		let json = serde_json::to_string(&header)?;

		let json_size = json.len() as u32;
		let aligned_json_size = json_size + (4 - (json_size % 4)) % 4;
		final_writer.write_u32::<LittleEndian>(4)?;
		written += std::mem::size_of::<u32>();
		final_writer.write_u32::<LittleEndian>(aligned_json_size + 8)?;
		written += std::mem::size_of::<u32>();
		final_writer.write_u32::<LittleEndian>(aligned_json_size + 4)?;
		written += std::mem::size_of::<u32>();
		final_writer.write_u32::<LittleEndian>(json_size)?;
		written += std::mem::size_of::<u32>();
		final_writer.write_all(json.as_bytes())?;
		written += json.len();
		final_writer.write_u16::<LittleEndian>(0)?;
		written += std::mem::size_of::<u16>();
		final_writer.write_all(&self.buffer)?;
		written += self.buffer.len();
		final_writer.flush()?;
		Ok(written)
	}
}

impl Default for AsarWriter {
	fn default() -> Self {
		Self {
			files: HashMap::new(),
			offset: 0,
			buffer: Vec::new(),
			hasher: HashAlgorithm::Sha256,
		}
	}
}

fn path_to_reverse_components(path: &Path) -> Result<VecDeque<String>> {
	Ok(path
		.components()
		.filter_map(|c| match c {
			Component::Prefix(_) | Component::RootDir => None,
			Component::ParentDir | Component::CurDir => unreachable!("path not absolutized"),
			Component::Normal(path) => Some(
				path.to_str()
					.map(str::to_string)
					.unwrap_or_else(|| path.to_string_lossy().into_owned()),
			),
		})
		.collect())
}

fn recursive_add_to_header(mut path: VecDeque<String>, file: File, header: &mut Header) {
	let header_map = match header {
		Header::File(_) => return,
		Header::Directory { files } => files,
	};
	match path.pop_front() {
		Some(name) if path.is_empty() => {
			header_map.insert(name, Header::File(file));
		}
		Some(name) => {
			let new_header = header_map.entry(name).or_insert_with(Header::new);
			recursive_add_to_header(path, file, new_header);
		}
		None => {
			unreachable!("path must have at least one component");
		}
	};
}

#[cfg(test)]
mod test {
	use super::AsarWriter;
	use crate::{
		header::{Header, TEST_ASAR},
		reader::AsarReader,
	};
	use std::io::Cursor;

	#[test]
	pub fn round_trip() {
		let (header_a, offset_a) =
			Header::read(&mut &*TEST_ASAR).expect("failed to read asar header");
		let reader_a = AsarReader::new(header_a, offset_a, TEST_ASAR).expect("failed to read asar");
		let mut writer = AsarWriter::new();
		writer
			.add_from_reader(&reader_a)
			.expect("failed to add asar");
		let mut out = Cursor::new(Vec::<u8>::new());
		writer.finalize(&mut out).expect("failed to finalize asar");
		let out = out.into_inner();
		let out_ref = out.as_ref() as &[u8];
		println!("{}", hex::encode(out_ref));
		let (header_b, offset_b) =
			Header::read(&mut &*out_ref).expect("failed to read asar header");
		let reader_b = AsarReader::new(header_b, offset_b, &out).expect("failed to read new asar");
		assert_eq!(reader_a.files(), reader_b.files());
	}
}
