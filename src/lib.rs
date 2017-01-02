// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.


//! A library to generate EPUB files.

#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;
extern crate mustache;
extern crate tempdir;
extern crate zip as libzip;
extern crate chrono;
extern crate uuid;

mod errors; 
mod epub;
mod zip;
mod zip_command;
mod zip_library;
mod templates;
mod toc;
mod epub_content;

pub use errors::*;
pub use epub::EpubBuilder;
pub use epub::EpubVersion;
pub use zip_command::ZipCommand;
pub use zip_library::ZipLibrary;
pub use zip::Zip;
pub use toc::Toc;
pub use toc::TocElement;
pub use epub_content::EpubContent;
