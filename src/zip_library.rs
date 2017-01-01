// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use errors::Result;
use errors::ResultExt;
use zip::Zip;

use std::path::Path;
use std::io;
use std::io::Read;
use std::io::Cursor;

use libzip::ZipWriter;
use libzip::write::FileOptions;

/// Zip files using the Rust `zip` library.
pub struct ZipLibrary {
    writer: ZipWriter<Cursor<Vec<u8>>>,
}

impl ZipLibrary {
    /// Creates a new ZipLibrary
    pub fn new() -> ZipLibrary {
        ZipLibrary {
            writer: ZipWriter::new(Cursor::new(vec!()))
        }
    }
}


impl Zip for ZipLibrary {
    fn write_file<P: AsRef<Path>, R: Read>(&mut self, path: P, mut content: R) -> Result<()> {
        let file = format!("{}", path.as_ref().display());
        let options = FileOptions::default();
        self.writer.start_file(format!("{}", file), options)
            .chain_err(|| format!("could not create file '{}' in epub",
                                  file))?;
        io::copy(&mut content, &mut self.writer)
            .chain_err(|| format!("could not write file '{}' in epub",
                                  file))?;
        Ok(())
    }

    fn generate(&mut self) -> Result<Vec<u8>> {
        let cursor = self.writer.finish()
            .chain_err(|| "error writing zip file")?;
        Ok(cursor.into_inner())
    }
}
