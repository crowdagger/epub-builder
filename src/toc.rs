// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.
use crate::common;

/// An element of the [Table of contents](struct.Toc.html)
///
/// # Example
///
/// ```
/// use epub_builder::TocElement;
/// TocElement::new("chapter_1.xhtml", "Chapter 1")
///     .child(TocElement::new("chapter_1.xhtml#1", "Chapter 1, section 1")
///               .child(TocElement::new("chapter_1.xhtml#1-1", "Chapter 1, section 1, subsection 1")));
/// ```
#[derive(Debug, Clone)]
pub struct TocElement {
    /// The level. 0: part, 1: chapter, 2: section, ...
    pub level: i32,
    /// The link
    pub url: String,
    /// Title of this entry
    pub title: String,
    /// Title of this entry without HTML tags (if None, defaults to the main one)
    pub raw_title: Option<String>,
    /// Inner elements
    pub children: Vec<TocElement>,
}

impl TocElement {
    /// Creates a new element of the toc
    ///
    /// By default, the element's level is `1` and it has no children.
    pub fn new<S1: Into<String>, S2: Into<String>>(url: S1, title: S2) -> TocElement {
        TocElement {
            level: 1,
            url: url.into(),
            title: title.into(),
            raw_title: None,
            children: vec![],
        }
    }

    /// Adds an alternate version of the title without HTML tags.
    ///
    /// Useful only if you disable escaping of HTML fields.
    pub fn raw_title<S: Into<String>>(mut self, title: S) -> TocElement {
        self.raw_title = Option::Some(title.into());
        self
    }

    /// Sets the level of a TocElement
    pub fn level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    /// Change level, recursively, so the structure keeps having some sense
    fn level_up(&mut self, level: i32) {
        self.level = level;
        for child in &mut self.children {
            if child.level <= self.level {
                child.level_up(level + 1);
            }
        }
    }

    /// Add a child to this element.
    ///
    /// This adjust the level of the child to be the level of its parents, plus 1;
    /// this means that there is no point in manually setting the level to elements
    /// added with this method.
    ///
    /// # Example
    ///
    /// ```
    /// use epub_builder::TocElement;
    /// let elem = TocElement::new("foo.xhtml", "Foo")
    ///     .child(TocElement::new("bar.xhtml", "Bar")
    ///          .level(42));
    ///
    /// // `Bar`'s level wiss still be `2`.
    /// ```
    pub fn child(mut self, mut child: TocElement) -> Self {
        if child.level <= self.level {
            child.level_up(self.level + 1);
        }
        self.children.push(child);
        self
    }

    /// Add element to self or to children, according to its level
    ///
    /// This will adds `element` directly to `self` if its level is equal or less
    /// to the last children element; else it will insert it to the last child.
    ///
    /// See the `add` method of [`Toc](struct.toc.html).
    pub fn add(&mut self, element: TocElement) {
        let mut inserted = false;
        if let Some(ref mut last_elem) = self.children.last_mut() {
            if element.level > last_elem.level {
                last_elem.add(element.clone());
                inserted = true;
            }
        }
        if !inserted {
            self.children.push(element);
        }
    }

    /// Render element for Epub's toc.ncx format
    #[doc(hidden)]
    pub fn render_epub(&self, mut offset: u32, escape_html: bool) -> (u32, String) {
        offset += 1;
        let id = offset;
        let children = if self.children.is_empty() {
            String::new()
        } else {
            let mut output: Vec<String> = Vec::new();
            for child in &self.children {
                let (n, s) = child.render_epub(offset, escape_html);
                offset = n;
                output.push(s);
            }
            format!("\n{}", common::indent(output.join("\n"), 1))
        };
        // Try to use the raw title of all HTML elements; if it doesn't exist, insert escaped title
        let mut title = html_escape::encode_text(&self.title);
        if let Some(ref raw_title) = &self.raw_title {
            title = std::borrow::Cow::Borrowed(raw_title);
        }
        (
            offset,
            format!(
                "\
<navPoint playOrder=\"{id}\" id=\"navPoint-{id}\">
  <navLabel>
   <text>{title}</text>
  </navLabel>
  <content src=\"{url}\"/>{children}
</navPoint>",
                id = html_escape::encode_double_quoted_attribute(&id.to_string()),
                title = title.trim(),
                url = html_escape::encode_double_quoted_attribute(&self.url),
                children = children, // Not escaped: XML content
            ),
        )
    }

    /// Render element as a list element
    #[doc(hidden)]
    pub fn render(&self, numbered: bool, escape_html: bool) -> String {
        if self.title.is_empty() {
            return String::new();
        }
        if self.children.is_empty() {
            format!(
                "<li><a href=\"{link}\">{title}</a></li>",
                link = html_escape::encode_double_quoted_attribute(&self.url),
                title = common::encode_html(&self.title, escape_html),
            )
        } else {
            let mut output: Vec<String> = Vec::new();
            for child in &self.children {
                output.push(child.render(numbered, escape_html));
            }
            let children = format!(
                "<{oul}>\n{children}\n</{oul}>",
                oul = if numbered { "ol" } else { "ul" }, // Not escaped: Static string
                children = common::indent(output.join("\n"), 1), // Not escaped: XML content
            );
            format!(
                "\
<li>
  <a href=\"{link}\">{title}</a>
{children}
</li>",
                link = html_escape::encode_double_quoted_attribute(&self.url),
                title = common::encode_html(&self.title, escape_html),
                children = common::indent(children, 1), // Not escaped: XML content
            )
        }
    }
}

/// A Table Of Contents
///
/// It basically contains a list of [`TocElement`](struct.TocElement.html)s.
///
/// # Example
///
/// Creates a Toc, fills it, and render it to HTML:
///
/// ```
/// use epub_builder::{Toc, TocElement};
/// Toc::new()
///    // add a level-1 element
///    .add(TocElement::new("intro.xhtml", "Introduction"))
///    // add a level-1 element with children
///    .add(TocElement::new("chapter_1.xhtml", "Chapter 1")
///            .child(TocElement::new("chapter_1.xhtml#section1", "1.1: Some section"))
///            .child(TocElement::new("chapter_1.xhtml#section2", "1.2: another section")))
///    // add a level-2 element, which will thus get "attached" to previous level-1 element
///    .add(TocElement::new("chapter_1.xhtml#section3", "1.3: yet another section")
///            .level(2))
///    // render the toc (non-numbered list, escape html) and returns a string
///    .render(false, true);
/// ```
#[derive(Debug, Default)]
pub struct Toc {
    /// The elements composing the TOC
    pub elements: Vec<TocElement>,
}

impl Toc {
    /// Creates a new, empty, Toc
    pub fn new() -> Toc {
        Toc { elements: vec![] }
    }

    /// Returns `true` if the toc is empty, `false` else.
    ///
    /// Note that `empty` here means that the the toc has zero *or one*
    /// element, since it's still not worth displaying it in this case.
    pub fn is_empty(&self) -> bool {
        self.elements.len() <= 1
    }

    /// Adds a [`TocElement`](struct.TocElement.html) to the Toc.
    ///
    /// This will look at the element's level and will insert it as a child of the last
    /// element of the Toc that has an inferior level.
    ///
    /// # Example
    ///
    /// ```
    /// # use epub_builder::{Toc, TocElement};
    /// let mut toc = Toc::new();
    /// // Insert an element at default level (1)
    /// toc.add(TocElement::new("chapter_1.xhtml", "Chapter 1"));
    ///
    /// // Insert an element at level 2
    /// toc.add(TocElement::new("section_1.xhtml", "Section 1")
    ///           .level(2));
    /// // "Section 1" is now a child of "Chapter 1"
    /// ```
    ///
    /// There are some cases where this behaviour might not be what you want; however,
    /// it makes sure that the TOC can still be renderer correctly for HTML and EPUB.
    pub fn add(&mut self, element: TocElement) -> &mut Self {
        let mut inserted = false;
        if let Some(ref mut last_elem) = self.elements.last_mut() {
            if element.level > last_elem.level {
                last_elem.add(element.clone());
                inserted = true;
            }
        }
        if !inserted {
            self.elements.push(element);
        }

        self
    }

    /// Render the Toc in a toc.ncx compatible way, for EPUB.
    ///
    /// * `escape_html`: whether titles should be HTML-encoded or not (only applies to titles)
    pub fn render_epub(&mut self, escape_html: bool) -> String {
        let mut output: Vec<String> = Vec::new();
        let mut offset = 0;
        for elem in &self.elements {
            let (n, s) = elem.render_epub(offset, escape_html);
            offset = n;
            output.push(s);
        }
        common::indent(output.join("\n"), 2)
    }

    /// Render the Toc in either <ul> or <ol> form (according to numbered)
    pub fn render(&mut self, numbered: bool, escape_html: bool) -> String {
        let mut output: Vec<String> = Vec::new();
        for elem in &self.elements {
            log::debug!("rendered elem: {:?}", &elem.render(numbered, escape_html));
            output.push(elem.render(numbered, escape_html));
        }
        common::indent(
            format!(
                "<{oul}>\n{output}\n</{oul}>",
                output = common::indent(output.join("\n"), 1), // Not escaped: XML content
                oul = if numbered { "ol" } else { "ul" }       // Not escaped: Static string
            ),
            2,
        )
    }
}

/////////////////////////////////////////////////////////////////////////////////
///                                  TESTS                                     //
/////////////////////////////////////////////////////////////////////////////////

#[test]
fn toc_simple() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1", "0.0.1").level(3));
    toc.add(TocElement::new("#2", "1").level(1));
    toc.add(TocElement::new("#3", "1.0.1").level(3));
    toc.add(TocElement::new("#4", "1.1").level(2));
    toc.add(TocElement::new("#5", "2"));
    let actual = toc.render(false, true);
    let expected = "    <ul>
      <li><a href=\"#1\">0.0.1</a></li>
      <li>
        <a href=\"#2\">1</a>
        <ul>
          <li><a href=\"#3\">1.0.1</a></li>
          <li><a href=\"#4\">1.1</a></li>
        </ul>
      </li>
      <li><a href=\"#5\">2</a></li>
    </ul>";
    assert_eq!(&actual, expected);
}

#[test]
fn toc_epub_simple() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1", "1"));
    toc.add(TocElement::new("#2", "2"));
    let actual = toc.render_epub(true);
    let expected = "    <navPoint playOrder=\"1\" id=\"navPoint-1\">
      <navLabel>
       <text>1</text>
      </navLabel>
      <content src=\"#1\"/>
    </navPoint>
    <navPoint playOrder=\"2\" id=\"navPoint-2\">
      <navLabel>
       <text>2</text>
      </navLabel>
      <content src=\"#2\"/>
    </navPoint>";
    assert_eq!(&actual, expected);
}

#[test]
fn toc_epub_simple_sublevels() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1", "1"));
    toc.add(TocElement::new("#1.1", "1.1").level(2));
    toc.add(TocElement::new("#2", "2"));
    toc.add(TocElement::new("#2.1", "2.1").level(2));
    let actual = toc.render_epub(true);
    let expected = "    <navPoint playOrder=\"1\" id=\"navPoint-1\">
      <navLabel>
       <text>1</text>
      </navLabel>
      <content src=\"#1\"/>
      <navPoint playOrder=\"2\" id=\"navPoint-2\">
        <navLabel>
         <text>1.1</text>
        </navLabel>
        <content src=\"#1.1\"/>
      </navPoint>
    </navPoint>
    <navPoint playOrder=\"3\" id=\"navPoint-3\">
      <navLabel>
       <text>2</text>
      </navLabel>
      <content src=\"#2\"/>
      <navPoint playOrder=\"4\" id=\"navPoint-4\">
        <navLabel>
         <text>2.1</text>
        </navLabel>
        <content src=\"#2.1\"/>
      </navPoint>
    </navPoint>";
    assert_eq!(&actual, expected);
}

#[test]
fn toc_epub_broken_sublevels() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1.1", "1.1").level(2));
    toc.add(TocElement::new("#2", "2"));
    toc.add(TocElement::new("#2.1", "2.1").level(2));
    let actual = toc.render_epub(true);
    let expected = "    <navPoint playOrder=\"1\" id=\"navPoint-1\">
      <navLabel>
       <text>1.1</text>
      </navLabel>
      <content src=\"#1.1\"/>
    </navPoint>
    <navPoint playOrder=\"2\" id=\"navPoint-2\">
      <navLabel>
       <text>2</text>
      </navLabel>
      <content src=\"#2\"/>
      <navPoint playOrder=\"3\" id=\"navPoint-3\">
        <navLabel>
         <text>2.1</text>
        </navLabel>
        <content src=\"#2.1\"/>
      </navPoint>
    </navPoint>";
    assert_eq!(&actual, expected);
}

#[test]
fn toc_epub_title_escaped() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1", "D&D"));
    let actual = toc.render_epub(true);
    let expected = "    <navPoint playOrder=\"1\" id=\"navPoint-1\">
      <navLabel>
       <text>D&amp;D</text>
      </navLabel>
      <content src=\"#1\"/>
    </navPoint>";
    assert_eq!(&actual, expected);
}

#[test]
fn toc_epub_title_not_escaped() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1", "<em>D&amp;D<em>").raw_title("D&amp;D"));
    let actual = toc.render_epub(false);
    let expected = "    <navPoint playOrder=\"1\" id=\"navPoint-1\">
      <navLabel>
       <text>D&amp;D</text>
      </navLabel>
      <content src=\"#1\"/>
    </navPoint>";
    assert_eq!(&actual, expected);
}
