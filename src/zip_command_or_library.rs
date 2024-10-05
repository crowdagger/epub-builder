// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::zip::Zip;
use crate::Result;
use crate::ZipCommand;
use crate::ZipLibrary;

use std::io::Read;
use std::io::Write;
use std::path::Path;

/// Wrapper around either a ZipCommand or a ZipLibrary
///
/// Allows to create an EpubBuilder that can decide at runtime which to use.
pub enum ZipCommandOrLibrary {
    /// Command variant
    Command(ZipCommand),
    /// Library variant
    Library(ZipLibrary),
}

impl Zip for ZipCommandOrLibrary {
    fn write_file<P: AsRef<Path>, R: Read>(&mut self, path: P, content: R) -> Result<()> {
        match self {
            ZipCommandOrLibrary::Command(ref mut command) => command.write_file(path, content),
            ZipCommandOrLibrary::Library(ref mut library) => library.write_file(path, content),
        }
    }

    fn generate<W: Write>(self, to: W) -> Result<()> {
        match self {
            ZipCommandOrLibrary::Command(command) => command.generate(to),
            ZipCommandOrLibrary::Library(library) => library.generate(to),
        }
    }
}

impl ZipCommandOrLibrary {
    /// Try to create a ZipCommand using `command`. If running `command` fails on the system,
    /// fall back to `ZipLibrary`.
    pub fn new(command: &str) -> Result<ZipCommandOrLibrary> {
        ZipCommand::new()
            .map(|mut z| {
                z.command(command);
                z
            })
            .and_then(|z| z.test().map(|_| z))
            .map(ZipCommandOrLibrary::Command)
            .or_else(|_| ZipLibrary::new().map(ZipCommandOrLibrary::Library))
    }
}
