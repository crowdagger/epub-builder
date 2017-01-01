// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use errors::{Result, Error};
use zip::Zip;

use std::io::Read;

/// Represents a EPUB version
#[derive(Debug)]
pub enum EpubVersion {
    /// EPUB 2 format
    Epub2,
    /// EPUB 3.0 format
    Epub3_0, 
}


/// Epub files generator
#[derive(Debug)]
pub struct Epub<Z:Zip> {
    version: EpubVersion,
    zip: Z,
}

impl<Z:Zip> Epub<Z> {
    /// Create a new default EPUB Generator
    pub fn new(zip: Z) -> Epub<Z> {
        Epub {
            version: EpubVersion::Epub2,
            zip: zip,
        }
    }

    /// Set EPUB version (default: epub2)
    pub fn epub_version(&mut self, version: EpubVersion) -> &mut Self {
        self.version = version;
        self
    }

    /// Set some EPUB metadata
    ///
    /// # Arguments
    ///
    /// * `metadata`: a (possibly empty) list (or other iterator) of (key, value) tuples
    ///
    /// # Metadata that are used by the EPUB generator
    ///
    /// * `author`: author(s) of the book;
    /// * `title`: title of the book;
    /// * `lang`: the language ot the book, quite important as EPUB renderers rely on it
    ///   for e.g. hyphenating words.
    /// * `subject`;
    /// * `description`;
    /// * `generator`: generator of the book (should be your program name);
    pub fn metadata<'a, I>(&mut self, metadata: I) -> Result<&mut Self>
        where I: IntoIterator<Item = &'a (&'a str, &'a str)> {
        Ok(self)
    }

    // /// Use specified zip command to zip the EPUB file
    // pub fn zip_command<S:Into<String>>(&mut self, command: S) -> &mut Self {
    //     self.zip = ZipTool::Command(command.into());
    //     self
    // }

    // /// Use the libzip library to zip the EPUB file.
    // pub fn zip_library(&mut self) -> &mut Self {
    //     self.zip = ZipTool::Library;
    //     self
    // }

    /// Sets a titlepage. If titlepage in not set, a default one will be generated.
    pub fn titlepage<R: Read>(&mut self,
                              content: R) -> Result<&mut Self> {
        Ok(self)
    }

    /// Add a chapter to the EPUB.
    ///
    /// Wraps around `add_content`.
    pub fn add_chapter<R: Read, S: Into<String>>(&mut self,
                                                 title: S,
                                                 content: R) -> Result<&mut Self> {
        self.add_content(1, title, vec!(), content)
    }

    /// Add a resource
    ///
    /// Can be a picture, font, ...
    ///
    /// # Arguments
    ///
    /// * `path`: the path where this file will be writen in the EPUB OEBPS structure,
    ///   e.g. `data/image_0.png`
    /// * `content`: the resource to include
    /// * `mime_type`: the mime type of this file, e.g. "image/png".
    pub fn add_resource<R: Read, S1: Into<String>, S2: Into<String>>(&mut self,
                                                                     path: S1,
                                                                     content: R,
                                                                     mime_type: S2) -> Result<&mut Self> {
        Ok(self)
    }
    
    /// Add a content file that will be added to the EPUB.
    ///
    /// # Arguments
    ///
    /// * `level`: the level this content will be added in the toc;
    /// * `title`: the title of this content, as it should appear in the TOC;
    /// * `inner_toc`: a table of contents descrbing the inner layout of the content;
    /// * `content`: should be the contents of an XHTML file.
    pub fn add_content<R: Read, S:Into<String>>(&mut self,
                                                level: usize,
                                                title: S,
                                                inner_toc: Vec<()>,
                                                content: R) -> Result<&mut Self> {
        Ok(self)
    }

    /// Generate the EPUB to the specified file.
    pub fn generate(self, file: &str) -> Result<()> {
        Ok(())
    }
}


