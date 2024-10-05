// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::zip::Zip;

use std::fmt;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use crate::Result;
use libzip::CompressionMethod;
use libzip::ZipWriter;

/// Zip files using the [Rust `zip`](https://crates.io/crates/zip) library.
///
/// While this has the advantage of not requiring an external `zip` command, I have
/// run into some issues when trying to export EPUB generated with this method to
/// ereaders (e.g. Kobo).
///
/// Note that these takes care of adding the mimetype (since it must not be deflated), it
/// should not be added manually.
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
    ///
    /// Also add mimetype at the beginning of the EPUB file.
    pub fn new() -> Result<ZipLibrary> {
        let mut writer = ZipWriter::new(Cursor::new(vec![]));
        writer.set_comment(""); // Fix issues with some readers
        writer.start_file(
            "mimetype",
            libzip::write::SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )?;
        writer
            .write(b"application/epub+zip")
            .map_err(|e| crate::Error::IoError {
                msg: "could not write mimetype in epub".to_string(),
                cause: e,
            })?;

        Ok(ZipLibrary { writer })
    }
}

impl Zip for ZipLibrary {
    fn write_file<P: AsRef<Path>, R: Read>(&mut self, path: P, mut content: R) -> Result<()> {
        let mut file = format!("{}", path.as_ref().display());
        if cfg!(target_os = "windows") {
            // Path names should not use backspaces in zip files
            file = file.replace('\\', "/");
        }
        let options = libzip::write::SimpleFileOptions::default();
        self.writer.start_file(file.clone(), options).map_err(|e| {
            crate::Error::ZipErrorWithMessage {
                msg: format!("could not create file '{}' in epub", file),
                cause: e,
            }
        })?;
        io::copy(&mut content, &mut self.writer).map_err(|e| crate::Error::IoError {
            msg: format!("could not write file '{}' in epub", file),
            cause: e,
        })?;
        Ok(())
    }

    fn generate<W: Write>(self, mut to: W) -> Result<()> {
        let cursor = self
            .writer
            .finish()
            .map_err(|e| crate::Error::ZipErrorWithMessage {
                msg: "error writing zip file".to_string(),
                cause: e,
            })?;
        let bytes = cursor.into_inner();
        to.write_all(bytes.as_ref())
            .map_err(|e| crate::Error::IoError {
                msg: "error writing to file".to_string(),
                cause: e,
            })?;
        Ok(())
    }
}
