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
//! epub-builder = "0.7"
//! ```
//!
//! # Example
//!
//! ```rust
//! use std::io::Write;
//!
//! fn run() -> epub_builder::Result<Vec<u8>> {
//!     // Some dummy content to fill our books
//!     let dummy_content = "Dummy content. This should be valid XHTML if you want a valid EPUB!";
//!     let dummy_image = "Not really a PNG image";
//!     let dummy_css = "body { background-color: pink }";
//!
//!     let mut output = Vec::<u8>::new();
//!
//!     // Create a new EpubBuilder using the zip library
//!     let mut builder = epub_builder::EpubBuilder::new(epub_builder::ZipLibrary::new()?)?;
//!     // Set some metadata
//!     builder.metadata("author", "Joan Doe")?
//!         .metadata("title", "Dummy Book")?
//!     // Set epub version to 3.0
//!         .epub_version(epub_builder::EpubVersion::V30)
//!     // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
//!         .stylesheet(dummy_css.as_bytes())?
//!     // Add a image cover file
//!         .add_cover_image("cover.png", dummy_image.as_bytes(), "image/png")?
//!     // Add a resource that is not part of the linear document structure
//!         .add_resource("some_image.png", dummy_image.as_bytes(), "image/png")?
//!     // Add a cover page
//!         .add_content(epub_builder::EpubContent::new("cover.xhtml", dummy_content.as_bytes())
//!                      .title("Cover")
//!                      .reftype(epub_builder::ReferenceType::Cover))?
//!     // Add a title page
//!         .add_content(epub_builder::EpubContent::new("title.xhtml", dummy_content.as_bytes())
//!                      .title("Title")
//!                      .reftype(epub_builder::ReferenceType::TitlePage))?
//!     // Add a chapter, mark it as beginning of the "real content"
//!         .add_content(epub_builder::EpubContent::new("chapter_1.xhtml", dummy_content.as_bytes())
//!                      .title("Chapter 1")
//!                      .reftype(epub_builder::ReferenceType::Text))?
//!     // Add a second chapter; this one has more toc information about its internal structure
//!         .add_content(epub_builder::EpubContent::new("chapter_2.xhtml", dummy_content.as_bytes())
//!                      .title("Chapter 2")
//!                      .child(epub_builder::TocElement::new("chapter_2.xhtml#1", "Chapter 2, section 1")))?
//!     // Add a section. Since its level is set to 2, it will be attached to the previous chapter.
//!         .add_content(epub_builder::EpubContent::new("section.xhtml", dummy_content.as_bytes())
//!                      .title("Chapter 2, section 2")
//!                      .level(2))?
//!     // Add a chapter without a title, which will thus not appear in the TOC.
//!         .add_content(epub_builder::EpubContent::new("notes.xhtml", dummy_content.as_bytes()))?
//!     // Generate a toc inside of the document, that will be part of the linear structure.
//!         .inline_toc();
//!     // Finally, write the EPUB file to a writer. It could be a `Vec<u8>`, a file,
//!     // `stdout` or whatever you like, it just needs to implement the `std::io::Write` trait.
//!     builder.generate(&mut output)?;
//!     Ok(output)
//! }
//!
//! fn main() {
//!     let _output = run().expect("Unable to create an epub document");
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

mod common;
mod epub;
mod epub_content;
mod templates;
mod toc;
mod zip;
#[cfg(feature = "zip-command")]
mod zip_command;
#[cfg(feature = "zip-command")]
#[cfg(feature = "libzip")]
mod zip_command_or_library;
#[cfg(feature = "libzip")]
mod zip_library;

pub use epub::EpubBuilder;
pub use epub::EpubVersion;
pub use epub::MetadataOpf;
pub use epub::PageDirection;
pub use epub_content::EpubContent;
pub use epub_content::ReferenceType;
use libzip::result::ZipError;
pub use toc::Toc;
pub use toc::TocElement;
#[cfg(feature = "zip-command")]
pub use zip_command::ZipCommand;
#[cfg(feature = "zip-command")]
#[cfg(feature = "libzip")]
pub use zip_command_or_library::ZipCommandOrLibrary;
#[cfg(feature = "libzip")]
pub use zip_library::ZipLibrary;

/// Error type of this crate. Each variant represent a type of event that may happen during this crate's operations.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error caused while processing a template or its rendering.
    #[error("{msg}: {cause:?}")]
    TemplateError {
        /// A message explaining what was happening when we recieved this error.
        msg: String,
        /// The root cause of the error.
        // Box the error, since it is quite large (at least 136 bytes, thanks clippy!)
        cause: Box<upon::Error>,
    },
    /// An error returned when encountering an unknown [`PageDirection`].
    #[error("Invalid page direction specification: {0}")]
    PageDirectionError(String),
    /// An error returned when an unknown metadata key has been encountered.
    #[error("Invalid metadata key: {0}")]
    InvalidMetadataError(String),
    /// An error returned when attempting to access the filesystem
    #[error("{msg}: {cause:?}")]
    IoError {
        /// A message explaining what was happening when we recieved this error.
        msg: String,
        /// The root cause of the error.
        cause: std::io::Error,
    },
    /// An error returned when something happened while invoking a zip program. See [`ZipCommand`].
    #[error("Error while executing zip command: {0}")]
    ZipCommandError(String),
    /// An error returned when the zip library itself returned an error. See [`ZipLibrary`].
    #[error(transparent)]
    ZipError(#[from] ZipError),
    /// An error returned when the zip library itself returned an error, but with an additional message. See [`ZipLibrary`].
    #[error("{msg}: {cause:?}")]
    ZipErrorWithMessage {
        /// A message explaining what was happening when we recieved this error.
        msg: String,
        /// The root cause of the error.
        cause: ZipError,
    },
    /// An error returned when an invalid [`Path`] has been encountered during epub processing.
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError {
            msg: format!("{value:?}"),
            cause: value,
        }
    }
}

/// A more convenient shorthand for functions returning an error in this crate.
pub type Result<T> = std::result::Result<T, Error>;
