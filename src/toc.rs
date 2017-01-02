// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// An element of the Table of contents
#[derive(Debug, Clone)]
pub struct TocElement {
    /// The level. 0: part, 1: chapter, 2: section, ...
    pub level: i32,
    /// The link
    pub url: String,
    /// Title of this entry
    pub title: String,
    /// Inner elements
    pub children: Vec<TocElement>,
}


impl TocElement {
    /// Creates a new element of the toc
    pub fn new<S1: Into<String>, S2: Into<String>>(url: S1,
                                                   title: S2) -> TocElement {
        TocElement {
            level: 1,
            url: url.into(),
            title: title.into(),
            children: vec!(),
        }
    }

    /// Sets the level of a TocElement
    pub fn level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    /// Change level, recursively, so the structure keeps having some sense
    pub fn level_up(&mut self, level: i32) {
        self.level = level;
        for mut child in self.children.iter_mut() {
            if child.level <= self.level {
                child.level_up(level + 1);
            }
        }
    }

    /// Add a child to this element. Adjust the level of the child to be the level
    /// of its parents, plus 1.
    pub fn child(mut self, mut child: TocElement) -> Self {
        if child.level <= self.level {
            child.level_up(self.level + 1);
        }
        self.children.push(child);
        self
    }

    /// Add element to self or to children, according to level
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
    pub fn render_epub(&self, mut offset: u32) -> (u32, String) {
        offset += 1;
        let id = offset;
        let children = if self.children.is_empty() {
            String::new()
        } else {
            let mut output = String::new();
            for child in &self.children {
                let (n, s) = child.render_epub(offset);
                offset = n;
                output.push_str(&s);
            }
            output
        };
        (offset,
         format!("
<navPoint id = \"navPoint-{id}\">
  <navLabel>
   <text>{title}</text>
  </navLabel>
  <content src = \"{url}\" />
{children}
</navPoint>",
                id = id,
                title = self.title,
                url = self.url,
                children = children))
                
    }

    /// Render element as a list element
    pub fn render(&self, numbered: bool) -> String {
        let children = if self.children.is_empty() {
            String::new()
        } else {
            let mut output = String::new();
            for child in &self.children {
                output.push_str(&child.render(numbered));
            }
            format!("\n<{oul}>{children}\n</{oul}>\n",
                    oul = if numbered { "ol" } else { "ul" },
                    children = output)
        };
        format!("<li><a href = \"{link}\">{title}</a>{children}</li>\n",
                link = self.url,
                title = self.title,
                children = children)

    }
}


/// A structure for manipulating Table Of Contents
#[derive(Debug)]
pub struct Toc {
    /// The elements composing the TOC
    pub elements: Vec<TocElement>,
}

impl Toc {
    /// Create a new, empty, Toc
    pub fn new() -> Toc {
        Toc {
            elements: vec![],
        }
    }

    /// Returns `true` if the toc is empty, `false` else.
    ///
    /// Note that `empty` here means that the the toc has zero *or one*
    /// element, since it's still useless in this case.
    pub fn is_empty(&self) -> bool {
        self.elements.len() <= 1
    }

    /// Adds an element
    pub fn add(&mut self, element: TocElement) {
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
    }

    /// Render the Toc in a toc.ncx compatible way, for EPUB.
    pub fn render_epub(&mut self) -> String {
        let mut output = String::new();
        let mut offset = 0;
        for elem in &self.elements {
            let (n, s) = elem.render_epub(offset);
            offset = n;
            output.push_str(&s);
        }
        output
    }

    /// Render the Toc in either <ul> or <ol> form (according to numbered)
    pub fn render(&mut self, numbered: bool) -> String {
        let mut output = String::new();
        for elem in &self.elements {
            output.push_str(&elem.render(numbered));
        }
        format!("<{oul}>\n{output}\n</{oul}>\n",
                output = output,
                oul = if numbered { "ol" } else { "ul" })
    }
}

/////////////////////////////////////////////////////////////////////////////////
//                                   TESTS                                     //
/////////////////////////////////////////////////////////////////////////////////

#[test]
fn toc_simple() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1", "0.0.1").level(3));
    toc.add(TocElement::new("#2", "1").level(1));
    toc.add(TocElement::new("#3", "1.0.1").level(3));
    toc.add(TocElement::new("#4", "1.1").level(2));
    toc.add(TocElement::new("#5", "2"));
    let actual = toc.render(false);
    let expected = "<ul>
<li><a href = \"#1\">0.0.1</a></li>
<li><a href = \"#2\">1</a>
<ul><li><a href = \"#3\">1.0.1</a></li>
<li><a href = \"#4\">1.1</a></li>

</ul>
</li>
<li><a href = \"#5\">2</a></li>

</ul>
";
    assert_eq!(&actual, expected);
}

#[test]
fn toc_epub_simple() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1", "1"));
    toc.add(TocElement::new("#2", "2"));
    let actual = toc.render_epub();
    let expected = "
<navPoint id = \"navPoint-1\">
  <navLabel>
   <text>1</text>
  </navLabel>
  <content src = \"#1\" />

</navPoint>
<navPoint id = \"navPoint-2\">
  <navLabel>
   <text>2</text>
  </navLabel>
  <content src = \"#2\" />

</navPoint>";
    assert_eq!(&actual, expected);
}

#[test]
fn toc_epub_simple_sublevels() {
    let mut toc = Toc::new();
    toc.add(TocElement::new("#1", "1"));
    toc.add(TocElement::new("#1.1","1.1").level(2));
    toc.add(TocElement::new("#2", "2"));
    toc.add(TocElement::new("#2.1","2.1").level(2));
    let actual = toc.render_epub();
    let expected = "
<navPoint id = \"navPoint-1\">
  <navLabel>
   <text>1</text>
  </navLabel>
  <content src = \"#1\" />

<navPoint id = \"navPoint-2\">
  <navLabel>
   <text>1.1</text>
  </navLabel>
  <content src = \"#1.1\" />

</navPoint>
</navPoint>
<navPoint id = \"navPoint-3\">
  <navLabel>
   <text>2</text>
  </navLabel>
  <content src = \"#2\" />

<navPoint id = \"navPoint-4\">
  <navLabel>
   <text>2.1</text>
  </navLabel>
  <content src = \"#2.1\" />

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
    let actual = toc.render_epub();
    let expected = "
<navPoint id = \"navPoint-1\">
  <navLabel>
   <text>1.1</text>
  </navLabel>
  <content src = \"#1.1\" />

</navPoint>\n<navPoint id = \"navPoint-2\">
  <navLabel>
   <text>2</text>
  </navLabel>
  <content src = \"#2\" />

<navPoint id = \"navPoint-3\">
  <navLabel>
   <text>2.1</text>
  </navLabel>
  <content src = \"#2.1\" />

</navPoint>\n</navPoint>";
    assert_eq!(&actual, expected);
}
