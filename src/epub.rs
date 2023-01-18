// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::templates;
use crate::toc::{Toc, TocElement};
use crate::zip::Zip;
use crate::ReferenceType;
use crate::{common, EpubContent};

use std::io;
use std::io::Read;
use std::path::Path;

use eyre::{bail, Context, Result};
use mustache::MapBuilder;

/// Represents the EPUB version.
///
/// Currently, this library supports EPUB 2.0.1 and 3.0.1.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Eq)]
pub enum EpubVersion {
    /// EPUB 2.0.1 format
    V20,
    /// EPUB 3.0.1 format
    V30,
}

/// EPUB Metadata
#[derive(Debug)]
struct Metadata {
    pub title: String,
    pub author: Vec<String>,
    pub lang: String,
    pub generator: String,
    pub toc_name: String,
    pub description: Vec<String>,
    pub subject: Vec<String>,
    pub license: Option<String>,
    pub date_published: Option<chrono::DateTime<chrono::Utc>>,
}

impl Metadata {
    /// Create new default metadata
    pub fn new() -> Metadata {
        Metadata {
            title: String::new(),
            author: vec![],
            lang: String::from("en"),
            generator: String::from("Rust EPUB library"),
            toc_name: String::from("Table Of Contents"),
            description: vec![],
            subject: vec![],
            license: None,
            date_published: None,
        }
    }
}

/// A file added in the EPUB
#[derive(Debug)]
struct Content {
    pub file: String,
    pub mime: String,
    pub itemref: bool,
    pub cover: bool,
    pub reftype: Option<ReferenceType>,
    pub title: String,
}

impl Content {
    /// Create a new content file
    pub fn new<S1, S2>(file: S1, mime: S2) -> Content
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Content {
            file: file.into(),
            mime: mime.into(),
            itemref: false,
            cover: false,
            reftype: None,
            title: String::new(),
        }
    }
}

/// Epub Builder
///
/// The main struct you'll need to use in this library. It is first created using
/// a wrapper to zip files; then you add content to it, and finally you generate
/// the EPUB file by calling the `generate` method.
///
/// ```
/// use epub_builder::EpubBuilder;
/// use epub_builder::ZipCommand;
/// use std::io;
///
/// // "Empty" EPUB file
/// let mut builder = EpubBuilder::new(ZipCommand::new().unwrap()).unwrap();
/// builder.metadata("title", "Empty EPUB").unwrap();
/// builder.metadata("author", "Ann 'Onymous").unwrap();
/// builder.generate(&mut io::stdout()).unwrap();
/// ```
#[derive(Debug)]
pub struct EpubBuilder<Z: Zip> {
    version: EpubVersion,
    zip: Z,
    files: Vec<Content>,
    metadata: Metadata,
    toc: Toc,
    stylesheet: bool,
    inline_toc: bool,
}

impl<Z: Zip> EpubBuilder<Z> {
    /// Create a new default EPUB Builder
    pub fn new(zip: Z) -> Result<EpubBuilder<Z>> {
        let mut epub = EpubBuilder {
            version: EpubVersion::V20,
            zip,
            files: vec![],
            metadata: Metadata::new(),
            toc: Toc::new(),
            stylesheet: false,
            inline_toc: false,
        };

        epub.zip
            .write_file("META-INF/container.xml", templates::CONTAINER)?;
        epub.zip.write_file(
            "META-INF/com.apple.ibooks.display-options.xml",
            templates::IBOOKS,
        )?;

        Ok(epub)
    }

    /// Set EPUB version (default: V20)
    ///
    /// Supported versions are:
    ///
    /// * `V20`: EPUB 2.0.1
    /// * 'V30`: EPUB 3.0.1
    pub fn epub_version(&mut self, version: EpubVersion) -> &mut Self {
        self.version = version;
        self
    }

    /// Set some EPUB metadata
    ///
    /// For most metadata, this function will replace the existing metadata, but for subject, cteator and identifier who
    /// can have multiple values, it will add data to the existing data, unless the empty string "" is passed, in which case
    /// it will delete existing data for this key.
    ///
    /// # Valid keys used by the EPUB builder
    ///
    /// * `author`: author(s) of the book;
    /// * `title`: title of the book;
    /// * `lang`: the language of the book, quite important as EPUB renderers rely on it
    ///   for e.g. hyphenating words.
    /// * `generator`: generator of the book (should be your program name);
    /// * `toc_name`: the name to use for table of contents (by default, "Table of Contents");
    /// * `subject`;
    /// * `description`;
    /// * `license`.

    pub fn metadata<S1, S2>(&mut self, key: S1, value: S2) -> Result<&mut Self>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        match key.as_ref() {
            "author" => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.author = vec![];
                } else {
                    self.metadata.author.push(value);
                }
            }
            "title" => self.metadata.title = value.into(),
            "lang" => self.metadata.lang = value.into(),
            "generator" => self.metadata.generator = value.into(),
            "description" => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.description = vec![];
                } else {
                    self.metadata.description.push(value);
                }
            }
            "subject" => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.subject = vec![];
                } else {
                    self.metadata.subject.push(value);
                }
            }
            "license" => self.metadata.license = Some(value.into()),
            "toc_name" => self.metadata.toc_name = value.into(),
            s => bail!("invalid metadata '{}'", s),
        }
        Ok(self)
    }

    /// Sets the publication date of the EPUB
    /// 
    /// This value is part of the metadata. If this function is not called, the time at the
    /// moment of generation will be used instead.
    pub fn set_publication_date(&mut self, date_published: chrono::DateTime<chrono::Utc>) {
        self.metadata.date_published = Some(date_published);
    }

    /// Sets stylesheet of the EPUB.
    ///
    /// This content will be written in a `stylesheet.css` file; it is used by
    /// some pages (such as nav.xhtml), you don't have use it in your documents though it
    /// makes sense to also do so.
    pub fn stylesheet<R: Read>(&mut self, content: R) -> Result<&mut Self> {
        self.add_resource("stylesheet.css", content, "text/css")?;
        self.stylesheet = true;
        Ok(self)
    }

    /// Adds an inline toc in the document.
    ///
    /// If this method is called it adds a page that contains the table of contents
    /// that appears in the document.
    ///
    /// The position where this table of contents will be inserted depends on when
    /// you call this method: if you call it before adding any content, it will be
    /// at the beginning, if you call it after, it will be at the end.
    pub fn inline_toc(&mut self) -> &mut Self {
        self.inline_toc = true;
        self.toc.add(TocElement::new(
            "toc.xhtml",
            self.metadata.toc_name.as_str(),
        ));
        let mut file = Content::new("toc.xhtml", "application/xhtml+xml");
        file.reftype = Some(ReferenceType::Toc);
        file.title = self.metadata.toc_name.clone();
        file.itemref = true;
        self.files.push(file);
        self
    }

    /// Add a resource to the EPUB file
    ///
    /// This resource can be a picture, a font, some CSS file, .... Unlike
    /// `add_content`, files added this way won't appear in the linear
    /// document.
    ///
    /// Note that these files will automatically be inserted into an `OEBPS` directory,
    /// so you don't need (and shouldn't) prefix your path with `OEBPS/`.
    ///
    /// # Arguments
    ///
    /// * `path`: the path where this file will be written in the EPUB OEBPS structure,
    ///   e.g. `data/image_0.png`
    /// * `content`: the resource to include
    /// * `mime_type`: the mime type of this file, e.g. "image/png".
    pub fn add_resource<R, P, S>(&mut self, path: P, content: R, mime_type: S) -> Result<&mut Self>
    where
        R: Read,
        P: AsRef<Path>,
        S: Into<String>,
    {
        self.zip
            .write_file(Path::new("OEBPS").join(path.as_ref()), content)?;
        log::debug!("Add resource: {:?}", path.as_ref().display());
        self.files.push(Content::new(
            format!("{}", path.as_ref().display()),
            mime_type,
        ));
        Ok(self)
    }

    /// Add a cover image to the EPUB.
    ///
    /// This works similarly to adding the image as a resource with the `add_resource`
    /// method, except, it signals it in the Manifest section so it is displayed as the
    /// cover by Ereaders
    pub fn add_cover_image<R, P, S>(
        &mut self,
        path: P,
        content: R,
        mime_type: S,
    ) -> Result<&mut Self>
    where
        R: Read,
        P: AsRef<Path>,
        S: Into<String>,
    {
        self.zip
            .write_file(Path::new("OEBPS").join(path.as_ref()), content)?;
        let mut file = Content::new(format!("{}", path.as_ref().display()), mime_type);
        file.cover = true;
        self.files.push(file);
        Ok(self)
    }

    /// Add a XHTML content file that will be added to the EPUB.
    ///
    /// # Examples
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary, EpubContent};
    /// let content = "Some content";
    /// let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// // Add a chapter that won't be added to the Table of Contents
    /// builder.add_content(EpubContent::new("intro.xhtml", content.as_bytes())).unwrap();
    /// ```
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary, EpubContent, TocElement};
    /// # let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// # let content = "Some content";
    /// // Sets the title of a chapter so it is added to the Table of contents
    /// // Also add information about its structure
    /// builder.add_content(EpubContent::new("chapter_1.xhtml", content.as_bytes())
    ///                      .title("Chapter 1")
    ///                      .child(TocElement::new("chapter_1.xhtml#1", "1.1"))).unwrap();
    /// ```
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary, EpubContent};
    /// # let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// # let content = "Some content";
    /// // Add a section, by setting the level to 2 (instead of the default value 1)
    /// builder.add_content(EpubContent::new("section.xhtml", content.as_bytes())
    ///                      .title("Section 1")
    ///                      .level(2)).unwrap();
    /// ```
    ///
    /// Note that these files will automatically be inserted into an `OEBPS` directory,
    /// so you don't need (and shouldn't) prefix your path with `OEBPS/`.
    ///
    /// # See also
    ///
    /// * [`EpubContent`](struct.EpubContent.html)
    /// * the `add_resource` method, to add other resources in the EPUB file.
    pub fn add_content<R: Read>(&mut self, content: EpubContent<R>) -> Result<&mut Self> {
        self.zip.write_file(
            Path::new("OEBPS").join(content.toc.url.as_str()),
            content.content,
        )?;
        let mut file = Content::new(content.toc.url.as_str(), "application/xhtml+xml");
        file.itemref = true;
        file.reftype = content.reftype;
        if file.reftype.is_some() {
            file.title = content.toc.title.clone();
        }
        self.files.push(file);
        if !content.toc.title.is_empty() {
            self.toc.add(content.toc);
        }
        Ok(self)
    }

    /// Generate the EPUB file and write it to the writer
    ///
    /// # Example
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, ZipLibrary};
    /// let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// // Write the EPUB file into a Vec<u8>
    /// let mut epub: Vec<u8> = vec!();
    /// builder.generate(&mut epub).unwrap();
    /// ```
    pub fn generate<W: io::Write>(&mut self, to: W) -> Result<()> {
        // If no styleesheet was provided, generate a dummy one
        if !self.stylesheet {
            self.stylesheet(b"".as_ref())?;
        }
        // Render content.opf
        let bytes = self.render_opf()?;
        self.zip.write_file("OEBPS/content.opf", &*bytes)?;
        // Render toc.ncx
        let bytes = self.render_toc()?;
        self.zip.write_file("OEBPS/toc.ncx", &*bytes)?;
        // Render nav.xhtml
        let bytes = self.render_nav(true)?;
        self.zip.write_file("OEBPS/nav.xhtml", &*bytes)?;
        // Write inline toc if it needs to
        if self.inline_toc {
            let bytes = self.render_nav(false)?;
            self.zip.write_file("OEBPS/toc.xhtml", &*bytes)?;
        }

        self.zip.generate(to)?;
        Ok(())
    }

    /// Render content.opf file
    fn render_opf(&mut self) -> Result<Vec<u8>> {
        log::debug!("render_opf...");
        let mut optional: Vec<String> = Vec::new();
        for desc in &self.metadata.description {
            optional.push(format!("<dc:description>{}</dc:description>", desc));
        }
        for subject in &self.metadata.subject {
            optional.push(format!("<dc:subject>{}</dc:subject>", subject));
        }
        if let Some(ref rights) = self.metadata.license {
            optional.push(format!("<dc:rights>{}</dc:rights>", rights));
        }
        let date = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
        let date_published = self.metadata.date_published.map(|date| date.format("%Y-%m-%dT%H:%M:%SZ"));
        let uuid = uuid::fmt::Urn::from_uuid(uuid::Uuid::new_v4()).to_string();

        let mut items: Vec<String> = Vec::new();
        let mut itemrefs: Vec<String> = Vec::new();
        let mut guide: Vec<String> = Vec::new();

        for content in &self.files {
            let id = if content.cover {
                String::from("cover-image")
            } else {
                to_id(&content.file)
            };
            let properties = match (self.version, content.cover) {
                (EpubVersion::V30, true) => "properties=\"cover-image\" ",
                _ => "",
            };
            if content.cover {
                optional.push("<meta name=\"cover\" content=\"cover-image\"/>".to_string());
            }
            log::debug!("id={:?}, mime={:?}", id, content.mime);
            items.push(format!(
                "<item media-type=\"{mime}\" {properties}\
                        id=\"{id}\" href=\"{href}\"/>",
                properties = properties,
                mime = content.mime,
                id = id,
                // in the zip the path is always with forward slashes, on windows it is with backslashes
                href = content.file.replace('\\', "/")
            ));
            if content.itemref {
                itemrefs.push(format!("<itemref idref=\"{id}\"/>", id = id));
            }
            if let Some(reftype) = content.reftype {
                use crate::ReferenceType::*;
                let reftype = match reftype {
                    Cover => "cover",
                    TitlePage => "title-page",
                    Toc => "toc",
                    Index => "index",
                    Glossary => "glossary",
                    Acknowledgements => "acknowledgements",
                    Bibliography => "bibliography",
                    Colophon => "colophon",
                    Copyright => "copyright",
                    Dedication => "dedication",
                    Epigraph => "epigraph",
                    Foreword => "foreword",
                    Loi => "loi",
                    Lot => "lot",
                    Notes => "notes",
                    Preface => "preface",
                    Text => "text",
                };
                log::debug!("content = {:?}", &content);
                guide.push(format!(
                    "<reference type=\"{reftype}\" title=\"{title}\" href=\"{href}\"/>",
                    reftype = reftype,
                    // escape < > symbols by &lt; &gt; using 'encode_text()' in Title
                    title = common::escape_quote(html_escape::encode_text(content.title.as_str())),
                    href = content.file
                ));
            }
        }

        let data = {
            let mut builder = MapBuilder::new()
                .insert_str("lang", self.metadata.lang.as_str())
                .insert_vec("author", |builder| {
                    let mut builder = builder;
                    for (i, author) in self.metadata.author.iter().enumerate() {
                        let author_escaped = html_escape::encode_text(author);
                        builder = builder.push_map(|builder| {
                            builder
                                .insert_str("id".to_string(), i.to_string())
                                .insert_str("name".to_string(), author_escaped.to_string())
                        });
                    }
                    builder
                })
                .insert_str("title", html_escape::encode_text(self.metadata.title.as_str()))
                .insert_str("generator", self.metadata.generator.as_str())
                .insert_str("toc_name", self.metadata.toc_name.as_str())
                .insert_str("optional", common::indent(optional.join("\n"), 2))
                .insert_str("items", common::indent(items.join("\n"), 2))
                .insert_str("itemrefs", common::indent(itemrefs.join("\n"), 2))
                .insert_str("date", date.to_string())
                .insert_str("uuid", uuid)
                .insert_str("guide", common::indent(guide.join("\n"), 2));

            if let Some(date) = date_published {
                builder = builder.insert_str("date_published", date.to_string());
            } else {
                builder = builder.insert_bool("date_published", false);
            }
                
            builder.build()
        };

        let mut content = vec![];
        let res = match self.version {
            EpubVersion::V20 => templates::v2::CONTENT_OPF.render_data(&mut content, &data),
            EpubVersion::V30 => templates::v3::CONTENT_OPF.render_data(&mut content, &data),
        };

        res.wrap_err("could not render template for content.opf")?;

        Ok(content)
    }

    /// Render toc.ncx
    fn render_toc(&mut self) -> Result<Vec<u8>> {
        let mut nav_points = String::new();

        nav_points.push_str(&self.toc.render_epub());

        let data = MapBuilder::new()
            .insert_str("toc_name", self.metadata.toc_name.as_str())
            .insert_str("nav_points", nav_points.as_str())
            .build();
        let mut res: Vec<u8> = vec![];
        templates::TOC_NCX
            .render_data(&mut res, &data)
            .wrap_err("error rendering toc.ncx template")?;
        Ok(res)
    }

    /// Render nav.xhtml
    fn render_nav(&mut self, numbered: bool) -> Result<Vec<u8>> {
        let content = self.toc.render(numbered);
        let mut landmarks: Vec<String> = Vec::new();
        if self.version > EpubVersion::V20 {
            for file in &self.files {
                if let Some(ref reftype) = file.reftype {
                    use ReferenceType::*;
                    let reftype = match *reftype {
                        Cover => "cover",
                        Text => "bodymatter",
                        Toc => "toc",
                        Bibliography => "bibliography",
                        Epigraph => "epigraph",
                        Foreword => "foreword",
                        Preface => "preface",
                        Notes => "endnotes",
                        Loi => "loi",
                        Lot => "lot",
                        Colophon => "colophon",
                        TitlePage => "titlepage",
                        Index => "index",
                        Glossary => "glossary",
                        Copyright => "copyright-page",
                        Acknowledgements => "acknowledgements",
                        Dedication => "dedication",
                    };
                    if !file.title.is_empty() {
                        landmarks.push(format!(
                            "<li><a epub:type=\"{reftype}\" href=\"{href}\">\
                                {title}</a></li>",
                            reftype = reftype,
                            href = file.file,
                            title = file.title
                        ));
                    }
                }
            }
        }

        let data = MapBuilder::new()
            .insert_str("content", content)
            .insert_str("toc_name", self.metadata.toc_name.as_str())
            .insert_str("generator", self.metadata.generator.as_str())
            .insert_str(
                "landmarks",
                if !landmarks.is_empty() {
                    common::indent(
                        format!("<ol>\n{}\n</ol>", common::indent(landmarks.join("\n"), 1)),
                        2,
                    )
                } else {
                    String::new()
                },
            )
            .build();

        let mut res = vec![];
        let eh = match self.version {
            EpubVersion::V20 => templates::v2::NAV_XHTML.render_data(&mut res, &data),
            EpubVersion::V30 => templates::v3::NAV_XHTML.render_data(&mut res, &data),
        };

        eh.wrap_err("error rendering nav.xhtml template")?;
        Ok(res)
    }
}

// The actual rules for ID are here - https://www.w3.org/TR/xml-names11/#NT-NCNameChar
// Ordering to to look as similar as possible to the W3 Recommendation ruleset
// Slightly more permissive, there are some that are invalid start chars, but this is ok.
fn is_id_char(c: char) -> bool {
    ('A'..='Z').contains(&c)
        || c == '_'
        || ('a'..='z').contains(&c)
        || ('\u{C0}'..='\u{D6}').contains(&c)
        || ('\u{D8}'..='\u{F6}').contains(&c)
        || ('\u{F8}'..='\u{2FF}').contains(&c)
        || ('\u{370}'..='\u{37D}').contains(&c)
        || ('\u{37F}'..='\u{1FFF}').contains(&c)
        || ('\u{200C}'..='\u{200D}').contains(&c)
        || ('\u{2070}'..='\u{218F}').contains(&c)
        || ('\u{2C00}'..='\u{2FEF}').contains(&c)
        || ('\u{3001}'..='\u{D7FF}').contains(&c)
        || ('\u{F900}'..='\u{FDCF}').contains(&c)
        || ('\u{FDF0}'..='\u{FFFD}').contains(&c)
        || ('\u{10000}'..='\u{EFFFF}').contains(&c)
        || c == '-'
        || c == '.'
        || ('0'..='9').contains(&c)
        || c == '\u{B7}'
        || ('\u{0300}'..='\u{036F}').contains(&c)
        || ('\u{203F}'..='\u{2040}').contains(&c)
}

// generate an id compatible string, replacing all none ID chars to underscores
fn to_id(s: &str) -> String {
    s.replace(|c: char| !is_id_char(c), "_")
}
