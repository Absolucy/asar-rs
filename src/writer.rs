// SPDX-License-Identifier: Apache-2.0 OR MIT
use crate::{
	error::{Error, Result},
	header::{File, FileIntegrity, HashAlgorithm, Header},
	reader::AsarReader,
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::{
	collections::{BTreeMap, VecDeque},
	io::Write,
	path::{Component, Path, PathBuf},
};

const BLOCK_SIZE: usize = 4 * 1024 * 1024; // 4 MiB

pub struct AsarWriter {
	files: BTreeMap<PathBuf, File>,
	buffer: Vec<u8>,
	offset: usize,
	hasher: HashAlgorithm,
}

impl AsarWriter {
	/// Creates a new [`AsarWriter`], with an empty buffer and the default
	/// [`HashAlgorithm`]
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new [`AsarWriter`], with an empty buffer and the given
	/// [`HashAlgorithm`]
	///
	/// Currently useless, as only one [`HashAlgorithm`] —
	/// [`HashAlgorithm::Sha256`] — is supported
	pub fn new_with_algorithm(hasher: HashAlgorithm) -> Self {
		Self {
			files: BTreeMap::new(),
			buffer: Vec::new(),
			offset: 0,
			hasher,
		}
	}

	/// Adds all the files from an [`AsarReader`] to the [`AsarWriter`].
	pub fn add_from_reader(&mut self, reader: &AsarReader) -> Result<()> {
		for (path, file) in reader.files() {
			self.write_file(path, file.data(), false)?;
		}
		Ok(())
	}

	/// Write a file to the archive.
	/// This appends the contents to the buffer, adds the file to the header,
	/// and updates the offset.
	///
	/// ## Errors
	///
	///  - If the file already exists in the archive, returns an
	///    [`Error::FileAlreadyWritten`]
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

	/// Finalizes the archive, writing the [`Header`] and the files to the
	/// writer.
	///
	/// The buffer is also flushed before returning.
	///
	/// Returns the amount of bytes written.
	///
	/// ## Errors
	///
	///  - If writing fails, an [std::io::Error] is returned.
	///  - This can **panic** if an invalid path (such as one containing `.` or
	///    `..`) was added to the archive.
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
	/// Creates a new [`AsarWriter`], with an empty buffer and the default
	/// [`HashAlgorithm`]
	fn default() -> Self {
		Self {
			files: BTreeMap::new(),
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
		let reader_a = AsarReader::new_from_header(header_a, offset_a, TEST_ASAR)
			.expect("failed to read asar");
		let mut writer = AsarWriter::new();
		writer
			.add_from_reader(&reader_a)
			.expect("failed to add asar");
		let mut out = Cursor::new(Vec::<u8>::new());
		writer.finalize(&mut out).expect("failed to finalize asar");
		let out = out.into_inner();
		let out_ref = out.as_ref() as &[u8];
		let (header_b, offset_b) =
			Header::read(&mut &*out_ref).expect("failed to read asar header");
		let reader_b =
			AsarReader::new_from_header(header_b, offset_b, &out).expect("failed to read new asar");
		let files_a = reader_a.files();
		let files_b = reader_b.files();
		let mut missing = Vec::new();
		let mut differs = Vec::new();
		assert_eq!(files_a.len(), files_b.len());
		for (k, v) in files_a {
			match files_b.get(k) {
				Some(v2) => {
					if v != v2 {
						differs.push((k.to_owned(), v.data(), v2.data()));
					}
				}
				None => {
					missing.push(k.to_owned());
				}
			}
		}
		if !missing.is_empty() || !differs.is_empty() {
			for m in missing {
				println!("missing: {}", m.display());
			}
			for (path, correct, incorrect) in differs {
				println!("differs: {}", path.display());
				let correct = std::str::from_utf8(correct).unwrap();
				let incorrect = std::str::from_utf8(incorrect).unwrap();
				println!("correct: {}", correct);
				println!("incorrect: {}", incorrect);
				println!();
			}
			panic!("ASAR archives differ!");
		}
	}
}
