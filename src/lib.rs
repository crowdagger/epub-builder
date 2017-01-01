// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.


//! A library to generate EPUB files.

#![deny(missing_docs)]

#[macro_use]
extern crate error_chain;
extern crate tempdir;
extern crate zip as libzip;

mod errors; 
mod epub;
mod zip;
mod zip_command;
mod zip_library;

pub use errors::*;
pub use epub::Epub;
pub use epub::EpubVersion;
pub use zip_command::ZipCommand;
pub use zip_library::ZipLibrary;
pub use zip::Zip;
