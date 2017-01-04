// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use errors::Result;
use errors::ResultExt;
use zip::Zip;

use std::io;
use std::fmt;
use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::io::Cursor;

use libzip::ZipWriter;
use libzip::write::FileOptions;

/// Zip files using the [Rust `zip`](https://crates.io/crates/zip) library.
pub struct ZipLibrary {
    writer: ZipWriter<Cursor<Vec<u8>>>,
}

impl fmt::Debug for ZipLibrary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ZipLibrary")
    }
}


impl ZipLibrary {
    /// Creates a new wrapper for zip library
    pub fn new() -> ZipLibrary {
        ZipLibrary { writer: ZipWriter::new(Cursor::new(vec![])) }
    }
}


impl Zip for ZipLibrary {
    fn write_file<P: AsRef<Path>, R: Read>(&mut self, path: P, mut content: R) -> Result<()> {
        let file = format!("{}", path.as_ref().display());
        let options = FileOptions::default();
        self.writer
            .start_file(format!("{}", file), options)
            .chain_err(|| format!("could not create file '{}' in epub", file))?;
        io::copy(&mut content, &mut self.writer)
            .chain_err(|| format!("could not write file '{}' in epub",
                                  file))?;
        Ok(())
    }

    fn generate<W: Write>(&mut self, mut to: W) -> Result<()> {
        let cursor = self.writer
            .finish()
            .chain_err(|| "error writing zip file")?;
        let bytes = cursor.into_inner();
        to.write_all(bytes.as_ref())
            .chain_err(|| "error writing zip file")?;
        Ok(())
    }
}
