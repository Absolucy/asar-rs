# asar

This crate allows for the parsing, reading, and writing of [asar](https://github.com/electron/asar) archives,
often seen in [Electron](https://www.electronjs.org/)-based applications.

## Examples

### Listing the contents of an asar archive
```rust
use asar::{AsarReader, Header, Result};
use std::fs;

fn main() -> Result<()> {
	let asar_file = fs::read("archive.asar")?;
	let asar = AsarReader::new(&asar_file)?;

	println!("There are {} files in archive.asar", asar.files().len());
	for path in asar.files().keys() {
		println!("{}", path.display());
	}
	Ok(())
}
```

### Reading a file from an asar archive
```rust
use asar::{AsarReader, Header, Result};
use std::{fs, path::PathBuf};

fn main() -> Result<()> {
	let asar_file = fs::read("archive.asar")?;
	let asar = AsarReader::new(&asar_file)?;

	let path = PathBuf::from("hello.txt");
	let file = asar.files().get(&path).unwrap();
	let contents = std::str::from_utf8(file.data()).unwrap();
	assert_eq!(contents, "Hello, World!");
	Ok(())
}
```

### Writing a file to an asar archive
```rust
use asar::{AsarWriter, Result};
use std::fs::File;

fn main() -> Result<()> {
	let mut asar = AsarWriter::new();
	asar.write_file("hello.txt", b"Hello, World!", false)?;
	asar.finalize(File::create("archive.asar")?)?;
	Ok(())
}
```

## Features

 - `integrity`: Enable integrity checks/calculation.
 - `check-integrity-on-read`: Enable integrity checks when reading an
   archive, failing if any integrity check fails.
 - `write` - Enable writing an asar archive. **Enabled by default**, also
   enables `integrity`.

## License

`asar` is licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License 2.0](LICENSE-APACHE), at the choice of the user.

License: Apache-2.0 OR MIT
