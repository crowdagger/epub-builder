
//! A library to generate EPUB files.

#![deny(missing_docs)]

#[macro_use]
extern crate error_chain;

mod errors; 
mod epub;

pub use errors::*;
pub use epub::Epub;
pub use epub::EpubVersion;
pub use epub::ZipTool;
