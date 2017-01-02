// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use toc::TocElement;

use std::io::Read;

/// Represents a XHTML file that can be added to an EPUB document.
///
/// This struct is designed to be used with the `add_content` method
/// of the `[EpubBuilder](struct.EpubBuilder.html).
///
/// # Example
///
/// ```
/// use epub_builder::{EpubContent, TocElement};
///
/// let page_content = "Some XHTML content";
///
/// // Creates a new EpubContent
/// let content = EpubContent::new("intro.xhtml", page_content.as_bytes())
/// // ... and sets a title so it is added to the TOC
///     .title("Introduction")
/// // ... and add some toc information on the document structure
///     .child(TocElement::new("intro.xhtml#1", "Section 1"))
///     .child(TocElement::new("intro.xhtml#2", "Section 2"));
/// ```
#[derive(Debug)]
pub struct EpubContent<R: Read> {
    /// The title and url, plus sublevels
    pub toc: TocElement,
    /// The content
    pub content: R,
}

impl<R: Read> EpubContent<R> {
    /// Creates a new EpubContent
    ///
    /// By default, this element is at level 1, and it has no title
    /// (meaning it won't be added to the [`Table of Contents`](struct.Toc.html).
    pub fn new<S: Into<String>>(href: S, content: R) -> Self {
        EpubContent {
            content: content,
            toc: TocElement::new(href, ""),
        }
    }

    /// Set the title of this content. If no title is set,
    /// this part of the book will not be displayed in the table of content.
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
