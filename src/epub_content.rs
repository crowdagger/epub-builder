// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use toc::TocElement;

use std::io::Read;

/// Represents a XHTML file that can be added to an EPUB document.
#[derive(Debug)]
pub struct EpubContent<R: Read> {
    /// The title and url, plus sublevels
    pub toc: TocElement,
    /// The content
    pub content: R,
}

impl<R: Read> EpubContent<R> {
    /// Creates a new EpubContent
    pub fn new<S: Into<String>>(href: S, content: R) -> Self {
        EpubContent {
            content: content,
            toc: TocElement::new(href, ""),
        }
    }

    /// Set the title of this content. Used for the TOC. If to title is set,
    /// this fragment will not be displayed in the TOC.
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.toc.title = title.into();
        self
    }

    /// Set the level
    pub fn level(mut self, level: i32) -> Self {
        self.toc = self.toc.level(level);
        self
    }

    /// Adds a sublevel to the toc
    pub fn child(mut self, elem: TocElement) -> Self {
        self.toc = self.toc.child(elem);
        self
    }
}
