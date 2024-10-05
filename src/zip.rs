// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::io::Read;
use std::io::Write;
use std::path::Path;

use crate::Result;

/// An abstraction over possible Zip implementations.
///
/// The actual implementations are `ZipCommand` (uses the system command zip) or
/// `ZipLibrary` (uses the [Rust zip library](https://crates.io/crates/zip)).
pub trait Zip {
    /// Write the source content to a file in the archive
    fn write_file<P: AsRef<Path>, R: Read>(&mut self, file: P, content: R) -> Result<()>;

    /// Generate the ZIP file
    fn generate<W: Write>(self, _: W) -> Result<()>;
}
