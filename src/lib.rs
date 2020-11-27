// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A library to generate EPUB files.
//!
//! The purpose for this library is to make it easier to generate EPUB files:
//! it should take care of most of the boilerplate for you, leaving you only
//! with the task of filling the actual content.
//!
//! # Usage
//!
//! Add this in your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! epub-builder = "0.4"
//! ```
//!
//! # Example
//!
//! ```ignore
//! extern crate epub_builder;
//!
//! use epub_builder::EpubBuilder;
//! use epub_builder::Result;
//! use epub_builder::ZipLibrary;
//! use epub_builder::EpubContent;
//! use epub_builder::ReferenceType;
//! use epub_builder::TocElement;
//!
//! use std::io;
//! use std::io::Write;
//!
//! // Try to print Zip file to stdout
//! fn run() -> Result<()> {
//!     // Some dummy content to fill our books
//!     let dummy_content = "Dummy content. This should be valid XHTML if you want a valid EPUB!";
//!     let dummy_image = "Not really a PNG image";
//!     let dummy_css = "body { background-color: pink }";
//!
//!     // Create a new EpubBuilder using the zip library
//!     EpubBuilder::new(ZipLibrary::new()?)?
//!     // Set some metadata
//!         .metadata("author", "Joan Doe")?
//!         .metadata("title", "Dummy Book")?
//!     // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
//!         .stylesheet(dummy_css.as_bytes())?
//!     // Add a image cover file
//!         .add_cover_image("cover.png", dummy_image.as_bytes(), "image/png")?
//!     // Add a resource that is not part of the linear document structure
//!         .add_resource("some_image.png", dummy_image.as_bytes(), "image/png")?
//!     // Add a cover page
//!         .add_content(EpubContent::new("cover.xhtml", dummy_content.as_bytes())
//!                      .title("Cover")
//!                      .reftype(ReferenceType::Cover))?
//!     // Add a title page
//!         .add_content(EpubContent::new("title.xhtml", dummy_content.as_bytes())
//!                      .title("Title")
//!                      .reftype(ReferenceType::TitlePage))?
//!     // Add a chapter, mark it as beginning of the "real content"
//!         .add_content(EpubContent::new("chapter_1.xhtml", dummy_content.as_bytes())
//!                      .title("Chapter 1")
//!                      .reftype(ReferenceType::Text))?
//!     // Add a second chapter; this one has more toc information about its internal structure
//!         .add_content(EpubContent::new("chapter_2.xhtml", dummy_content.as_bytes())
//!                      .title("Chapter 2")
//!                      .child(TocElement::new("chapter_2.xhtml#1", "Chapter 2, section 1")))?
//!     // Add a section. Since its level is set to 2, it will be attached to the previous chapter.
//!         .add_content(EpubContent::new("section.xhtml", dummy_content.as_bytes())
//!                      .title("Chapter 2, section 2")
//!                      .level(2))?
//!     // Add a chapter without a title, which will thus not appear in the TOC.
//!         .add_content(EpubContent::new("notes.xhtml", dummy_content.as_bytes()))?
//!     // Generate a toc inside of the document, that will be part of the linear structure.
//!         .inline_toc()
//!     // Finally, write the EPUB file to stdout
//!         .generate(&mut io::stdout())?;
//!     Ok(())
//! }
//!
//! fn main() {
//!     match run() {
//!         Ok(_) => {
//!             writeln!(&mut io::stderr(),
//!                      "Successfully wrote epub document to stdout!")
//!                 .unwrap()
//!         }
//!         Err(err) => writeln!(&mut io::stderr(), "Error: {}", err).unwrap(),
//!     };
//! }
//! ```
//!
//! # Features
//!
//! `epub-builder`'s aim is to make EPUB generation simpler. It takes care of zipping
//! the files and generate the following ones:
//!
//! * `mimetype`
//! * `toc.ncx`
//! * `nav.xhtml`
//! * `manifest.xml`
//! * `content.opf`
//! * `com.apple.ibooks.display-options.xml`.
//!
//! It also tries to make it easier to have a correct table of contents, and optionally
//! generate an inline one in the document.
//!
//! Supported EPUB versions:
//!
//! * 2.0.1 (default)
//! * 3.0.1
//!
//! ## Missing features
//!
//! There are various EPUB features that `epub-builder` doesn't handle. Particularly,
//! there are some metadata that could be better
//! handled (e.g. support multiple authors, multiple languages in the document and so on).
//!
//! There are also various things that aren't in the scope of this library: it doesn't
//! provide a default CSS, templates for your XHTML content and so on. This is left to
//! libraries or applications using it.
//!
//! # Conditional compilation
//!
//! EPUB files are Zip files, so we need to zip. By default, this library provides
//! wrappers around both the [Rust zip library](https://crates.io/crates/zip) and calls
//! to the `zip` command that may (or may not) be installed on your system.
//!
//! It is possible to disable the compilation (and the dependencies) of either of these
//! wrappers, using `no-default-features`. (If you don't enable at least one of them this
//! library will be pretty useless).
//!
//! # License
//!
//! This is free software, published under the [Mozilla Public License,
//! version 2.0](https://www.mozilla.org/en-US/MPL/2.0/).
#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate error_chain;
extern crate chrono;
extern crate html_escape;
extern crate mustache;
extern crate regex;
#[cfg(feature = "zip-command")]
extern crate tempdir;
extern crate uuid;
#[cfg(feature = "zip-library")]
extern crate zip as libzip;
#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate log;

mod common;
mod epub;
mod epub_content;
mod errors;
mod templates;
mod toc;
mod zip;
#[cfg(feature = "zip-command")]
mod zip_command;
#[cfg(feature = "zip-command")]
#[cfg(feature = "zip-library")]
mod zip_command_or_library;
#[cfg(feature = "zip-library")]
mod zip_library;

pub use epub::EpubBuilder;
pub use epub::EpubVersion;
pub use epub_content::EpubContent;
pub use epub_content::ReferenceType;
pub use errors::*;
pub use toc::Toc;
pub use toc::TocElement;
#[cfg(feature = "zip-command")]
pub use zip_command::ZipCommand;
#[cfg(feature = "zip-command")]
#[cfg(feature = "zip-library")]
pub use zip_command_or_library::ZipCommandOrLibrary;
#[cfg(feature = "zip-library")]
pub use zip_library::ZipLibrary;
