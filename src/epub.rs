// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::templates;
use crate::toc::{Toc, TocElement};
use crate::zip::Zip;
use crate::ReferenceType;
use crate::Result;
use crate::{common, EpubContent};

use core::fmt::Debug;
use std::io;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;
use upon::Engine;

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
    /// EPUB 3.3.0 format
    V33,
}

pub trait MetadataRenderer: Send + Sync {
    fn render_opf(&self, escape_html: bool) -> String;
}

impl Debug for dyn MetadataRenderer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MetadataRenderer{{{}}}", self.render_opf(true))
    }
}

/// Represents the EPUB `<meta>` content inside `content.opf` file.
///
/// <meta dir="" id="" property="" refines="" scheme="" xml:lang="">content</meta>
/// https://www.w3.org/TR/epub-33/#sec-meta-elem
#[derive(Debug)]
pub struct MetadataOpfV3 {
    /// The property attribute takes a property data type value that defines the statement
    /// made in the expression, and the text content of the element represents the assertion. 
    /// https://www.w3.org/TR/epub-33/#attrdef-meta-property
    pub property: String,

    /// The content of the metadata tag very much based on what you put in property
    pub content: String,

    /// Specifies the base direction [bidi] of the textual content and attribute values
    /// of the carrying element and its descendants.
    /// https://www.w3.org/TR/epub-33/#attrdef-dir
    pub dir: Option<String>,

    /// The ID xml of the element, which MUST be unique within the document scope.
    /// https://www.w3.org/TR/epub-33/#attrdef-id
    pub id: Option<String>,

    /// Establishes an association between the current expression and the element or resource
    /// identified by its value. 
    /// https://www.w3.org/TR/epub-33/#attrdef-refines
    pub refines: Option<String>,

    /// The scheme attribute identifies the system or scheme the EPUB creator obtained the
    /// element's value from.
    /// https://www.w3.org/TR/epub-33/#attrdef-scheme
    pub scheme: Option<String>,

    /// Specifies the language of the textual content and attribute values of the
    /// carrying element and its descendants.
    /// https://www.w3.org/TR/epub-33/#attrdef-xml-lang
    pub xml_lang: Option<String>,
}

impl MetadataOpfV3 {
    /// Create instance of MetadataOpfV3
    ///
    pub fn new(property: String, content: String) -> MetadataOpfV3 {
        MetadataOpfV3{
            property: property,
            content: content,
            dir: None,
            id: None,
            refines: None,
            scheme: None,
            xml_lang: None,
        }
    }

    /// Add reading direction metadata
    pub fn add_direction(&mut self, direction: String) -> &mut Self {
        self.dir = Some(direction);
        self
    }

    /// Add id metadata
    pub fn add_id(&mut self, id: String) -> &mut Self {
        self.id = Some(id);
        self
    }

    /// Add refines metadata
    pub fn add_refines(&mut self, refines: String) -> &mut Self {
        self.id = Some(refines);
        self
    }

    /// Add scheme metadata
    pub fn add_scheme(&mut self, scheme: String) -> &mut Self {
        self.scheme = Some(scheme);
        self
    }

    /// Add xml_lang metadata
    pub fn add_xml_lang(&mut self, xml_lang: String) -> &mut Self {
        self.xml_lang = Some(xml_lang);
        self
    }
}

impl MetadataRenderer for MetadataOpfV3 {
    /// Create instance of MetadataOpfV3
    fn render_opf(&self, escape_html: bool) -> String {
        let mut meta_tag = String::from("<meta ");

        if let Some(dir) = &self.dir {
            meta_tag.push_str(&format!(
                    "dir=\"{}\" ", common::encode_html(dir, escape_html),
            ));
        }

        if let Some(id) = &self.id {
            meta_tag.push_str(&format!(
                    "id=\"{}\" ", common::encode_html(id, escape_html),
            ));
        }

        if let Some(refines) = &self.refines {
            meta_tag.push_str(&format!(
                    "refines=\"{}\" ", common::encode_html(refines, escape_html)
            ));
        }

        if let Some(scheme) = &self.scheme {
            meta_tag.push_str(&format!(
                    "scheme=\"{}\" ", common::encode_html(scheme, escape_html),
            ));
        }

        if let Some(xml_lang) = &self.xml_lang {
            meta_tag.push_str(&format!(
                    "xml:lang=\"{}\" ", common::encode_html(xml_lang, escape_html) 
            ));
        }

        meta_tag.push_str(&format!(
                "property=\"{}\">{}</meta>",
                common::encode_html(&self.property, escape_html),
                &self.content,
        ));

        meta_tag
    }
}

/// Represents the EPUB `<meta>` content inside `content.opf` file.
///
/// <meta name="" content="">
/// 
#[derive(Debug)]
pub struct MetadataOpf {
    /// Name of the `<meta>` tag
    pub name: String,
    /// Content of the `<meta>` tag
    pub content: String
}

impl MetadataOpf {
    /// Create new instance
    /// 
    /// 
    pub fn new(&self, meta_name: String, meta_content: String) -> Self {
        Self { name: meta_name, content: meta_content }
    }
}

impl MetadataRenderer for MetadataOpf {
    fn render_opf(&self, escape_html: bool) -> String {
        format!(
            "<meta name=\"{}\" content=\"{}\"/>", 
            common::encode_html(&self.name, escape_html),
            common::encode_html(&self.content, escape_html),
        )
    }
}

/// The page-progression-direction attribute of spine is a global attribute and
/// therefore defines the pagination flow of the book as a whole.
#[derive(Debug, Copy, Clone, Default)]
pub enum PageDirection {
    /// Left to right
    #[default]
    Ltr,
    /// Right to left
    Rtl,
}

impl ToString for PageDirection {
    fn to_string(&self) -> String {
        match &self {
            PageDirection::Rtl => "rtl".into(),
            PageDirection::Ltr => "ltr".into(),
        }
    }
}

impl FromStr for PageDirection {
    type Err = crate::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_ref() {
            "rtl" => Ok(PageDirection::Rtl),
            "ltr" => Ok(PageDirection::Ltr),
            _ => Err(crate::Error::PageDirectionError(s)),
        }
    }
}

/// A metadata key accepted by [`EpubBuilder::metadata`].
///
/// Passing a `MetadataKey` instead of a string lets the compiler check the key: a typo is
/// a compilation error, instead of a runtime
/// [`Error::InvalidMetadataError`](crate::Error::InvalidMetadataError).
///
/// ```
/// use epub_builder::{EpubBuilder, MetadataKey, ZipLibrary};
///
/// let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
/// builder
///     .metadata(MetadataKey::Title, "My book").unwrap()
///     .metadata(MetadataKey::Author, "Joan Doe").unwrap()
///     .metadata(MetadataKey::Lang, "en").unwrap();
/// ```
///
/// An invalid key does not compile:
///
/// ```compile_fail,E0599
/// use epub_builder::MetadataKey;
///
/// let key = MetadataKey::Titel;
/// ```
///
/// Each key corresponds to the string that `metadata` also accepts (see
/// [`as_str`](MetadataKey::as_str)); the conversion between both works in either direction,
/// through [`AsRef<str>`], [`Display`](core::fmt::Display) and [`FromStr`].
///
/// This enum is `#[non_exhaustive]`: new keys may be added in the future.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MetadataKey {
    /// `author`: author of the book.
    ///
    /// Each call adds an author; the empty string removes all of them.
    Author,
    /// `title`: title of the book.
    ///
    /// Each call replaces the previous title.
    Title,
    /// `lang`: language of the book, quite important as EPUB renderers rely on it
    /// for e.g. hyphenating words.
    ///
    /// Each call adds a language; the empty string removes all of them.
    Lang,
    /// `direction`: page progression direction of the book, `ltr` or `rtl`
    /// (case-insensitive). Any other value is rejected with
    /// [`Error::PageDirectionError`](crate::Error::PageDirectionError).
    ///
    /// Each call replaces the previous direction. It is only used when generating EPUB 3.
    Direction,
    /// `generator`: generator of the book (should be your program name).
    ///
    /// Each call replaces the previous generator.
    Generator,
    /// `description`: description of the book.
    ///
    /// Each call adds a description; the empty string removes all of them.
    Description,
    /// `subject`: subject of the book.
    ///
    /// Each call adds a subject; the empty string removes all of them.
    Subject,
    /// `license`: license under which the book is distributed.
    ///
    /// Each call replaces the previous license.
    License,
    /// `toc_name`: name to use for the table of contents (by default, "Table Of Contents").
    ///
    /// Each call replaces the previous name.
    TocName,
}

impl MetadataKey {
    /// Every key, in order of declaration. [`FromStr`] relies on it to find a key from its string.
    const ALL: [MetadataKey; 9] = [
        MetadataKey::Author,
        MetadataKey::Title,
        MetadataKey::Lang,
        MetadataKey::Direction,
        MetadataKey::Generator,
        MetadataKey::Description,
        MetadataKey::Subject,
        MetadataKey::License,
        MetadataKey::TocName,
    ];

    /// Returns the string corresponding to this key, as accepted by [`EpubBuilder::metadata`].
    ///
    /// ```
    /// use epub_builder::MetadataKey;
    ///
    /// assert_eq!(MetadataKey::TocName.as_str(), "toc_name");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            MetadataKey::Author => "author",
            MetadataKey::Title => "title",
            MetadataKey::Lang => "lang",
            MetadataKey::Direction => "direction",
            MetadataKey::Generator => "generator",
            MetadataKey::Description => "description",
            MetadataKey::Subject => "subject",
            MetadataKey::License => "license",
            MetadataKey::TocName => "toc_name",
        }
    }
}

impl AsRef<str> for MetadataKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl core::fmt::Display for MetadataKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MetadataKey {
    type Err = crate::Error;

    /// Parses a key from its string. The match is exact (case-sensitive); anything else
    /// is an [`Error::InvalidMetadataError`](crate::Error::InvalidMetadataError).
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|key| key.as_str() == s)
            .ok_or_else(|| crate::Error::InvalidMetadataError(s.to_string()))
    }
}

/// EPUB Metadata
#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub author: Vec<String>,
    pub lang: Vec<String>,
    pub direction: PageDirection,
    pub generator: String,
    pub toc_name: String,
    pub description: Vec<String>,
    pub subject: Vec<String>,
    pub license: Option<String>,
    pub date_published: Option<chrono::DateTime<chrono::Utc>>,
    pub date_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub uuid: Option<uuid::Uuid>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: vec![],
            lang: vec![],
            direction: PageDirection::default(),
            generator: String::from("Rust EPUB library"),
            toc_name: String::from("Table Of Contents"),
            description: vec![],
            subject: vec![],
            license: None,
            date_published: None,
            date_modified: None,
            uuid: None,
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
    direction: PageDirection,    
    zip: Z,
    files: Vec<Content>,
    metadata: Metadata,
    toc: Toc,
    stylesheet: bool,
    inline_toc: bool,
    escape_html: bool,
    meta_opf: Vec<Box<dyn MetadataRenderer>>,
}

impl<Z: Zip> EpubBuilder<Z> {
    /// Create a new default EPUB Builder
    pub fn new(zip: Z) -> Result<EpubBuilder<Z>> {
        let mut epub = EpubBuilder {
            version: EpubVersion::V20,
            direction: PageDirection::Ltr,
            zip,
            files: vec![],
            metadata: Metadata::default(),
            toc: Toc::new(),
            stylesheet: false,
            inline_toc: false,
            escape_html: true,
            meta_opf: vec![],
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
    /// * 'V33`: EPUB 3.3
    pub fn epub_version(&mut self, version: EpubVersion) -> &mut Self {
        self.version = version;
        self
    }
    
    /// Set EPUB Direction (default: Ltr)
    ///
    /// * `Ltr`: Left-To-Right 
    /// * `Rtl`: Right-To-Left 
    /// 
    /// 
    pub fn epub_direction(&mut self, direction: PageDirection) -> &mut Self {
        self.direction = direction;
        self
    }
    
    /// Add custom <meta> to `content.opf`
    /// Syntax: `self.add_metadata_opf(name, content)`
    /// 
    /// ### Example
    /// If you wanna add `<meta name="primary-writing-mode" content="vertical-rl"/>` into `content.opf`
    /// 
    /// ```rust
    /// use epub_builder::EpubBuilder;
    /// use epub_builder::ZipCommand;
    /// use epub_builder::MetadataOpf;
    /// use epub_builder::MetadataOpfV3;
    /// let mut builder = EpubBuilder::new(ZipCommand::new().unwrap()).unwrap();
    ///
    /// builder.add_metadata_opf(Box::new(MetadataOpf{
    ///         name: String::from("dcterms:modified"),
    ///         content: String::from("2016-02-29T12:34:56Z")
    ///     }
    /// ));
    ///
    /// builder.add_metadata_opf(Box::new(MetadataOpfV3::new(
    ///         String::from("dcterms:modified"),
    ///         String::from("2016-02-29T12:34:56Z")
    /// )));
    ///
    /// ```
    /// 
    pub fn add_metadata_opf(&mut self, item: Box<dyn MetadataRenderer>) -> &mut Self {
        self.meta_opf.push(item);
        self
    }

    /// Set some EPUB metadata
    ///
    /// For most metadata, this function will replace the existing metadata, but for author, lang, description and
    /// subject, which can have multiple values, it will add data to the existing data, unless the empty string ""
    /// is passed, in which case it will delete existing data for this key.
    ///
    /// # Valid keys used by the EPUB builder
    ///
    /// The `key` is preferably a [`MetadataKey`], which cannot be invalid. For compatibility,
    /// the string corresponding to a key ([`MetadataKey::as_str`]) is still accepted. See
    /// [`MetadataKey`] for the list of keys and how each of them is handled.
    ///   for e.g. hyphenating words.
    ///
    /// # Errors
    ///
    /// * [`Error::InvalidMetadataError`](crate::Error::InvalidMetadataError) if `key` is a string
    ///   that is not the string of a [`MetadataKey`] (this cannot happen with a `MetadataKey`);
    /// * [`Error::PageDirectionError`](crate::Error::PageDirectionError) if `key` is
    ///   [`MetadataKey::Direction`] and `value` is neither `ltr` nor `rtl`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use epub_builder::{EpubBuilder, MetadataKey, ZipLibrary};
    /// let mut builder = EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap();
    /// builder.metadata(MetadataKey::Title, "My book").unwrap();
    /// // A string is still accepted, but it can only be checked when running
    /// builder.metadata("author", "John Doe").unwrap();
    /// assert!(builder.metadata("autor", "John Doe").is_err());
    /// ```
    pub fn metadata<S1, S2>(&mut self, key: S1, value: S2) -> Result<&mut Self>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        match key.as_ref().parse::<MetadataKey>()? {
            MetadataKey::Author => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.author = vec![];
                } else {
                    self.metadata.author.push(value);
                }
            }
            MetadataKey::Title => self.metadata.title = value.into(),
            MetadataKey::Lang => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.lang = vec![];
                } else {
                    self.metadata.lang.push(value.into())
                }
            }
            MetadataKey::Direction => self.metadata.direction = PageDirection::from_str(&value.into())?,
            MetadataKey::Generator => self.metadata.generator = value.into(),
            MetadataKey::Description => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.description = vec![];
                } else {
                    self.metadata.description.push(value);
                }
            }
            MetadataKey::Subject => {
                let value = value.into();
                if value.is_empty() {
                    self.metadata.subject = vec![];
                } else {
                    self.metadata.subject.push(value);
                }
            }
            MetadataKey::License => self.metadata.license = Some(value.into()),
            MetadataKey::TocName => self.metadata.toc_name = value.into(),
        }
        Ok(self)
    }

    /// Sets the authors of the EPUB
    pub fn set_authors(&mut self, value: Vec<String>) {
        self.metadata.author = value;
    }

    /// Add an author to the EPUB
    pub fn add_author<S: Into<String>>(&mut self, value: S) {
        self.metadata.author.push(value.into());
    }

    /// Remove all authors from EPUB
    pub fn clear_authors<S: Into<String>>(&mut self) {
        self.metadata.author.clear();
    }

    /// Sets the title of the EPUB
    pub fn set_title<S: Into<String>>(&mut self, value: S) {
        self.metadata.title = value.into();
    }

    /// Tells whether fields should be HTML-escaped.
    ///
    /// * `true`: fields such as titles, description, and so on will be HTML-escaped everywhere (default)
    /// * `false`: fields will be left as is (letting you in charge of making
    /// sure they do not contain anything illegal, e.g. < and > characters)
    pub fn escape_html(&mut self, val: bool) {
        self.escape_html = val;
    }

    /// Sets a single language of the EPUB
    #[deprecated(since = "0.8.3", note = "Use set_languages or add_language instead")]
    pub fn set_lang<S: Into<String>>(&mut self, value: S) {
        self.clear_languages();
        self.add_language(value);
    }

    /// Sets the languages of the EPUB
    pub fn set_languages(&mut self, value: Vec<String>) {
        self.metadata.lang = value;
    }

    /// Adds a language of the EPUB
    ///
    /// This is quite important as EPUB renderers rely on it
    /// for e.g. hyphenating words.
    pub fn add_language<S: Into<String>>(&mut self, value: S) {
        self.metadata.lang.push(value.into());
    }

    /// Remove all languages from EPUB
    pub fn clear_languages(&mut self) {
        self.metadata.lang.clear();
    }

    /// Sets the generator of the book (should be your program name)
    pub fn set_generator<S: Into<String>>(&mut self, value: S) {
        self.metadata.generator = value.into();
    }

    /// Sets the name to use for table of contents. This is by default, "Table of Contents"
    pub fn set_toc_name<S: Into<String>>(&mut self, value: S) {
        self.metadata.toc_name = value.into();
    }

    /// Sets and replaces the description of the EPUB
    pub fn set_description(&mut self, value: Vec<String>) {
        self.metadata.description = value;
    }

    /// Adds a line to the EPUB description
    pub fn add_description<S: Into<String>>(&mut self, value: S) {
        self.metadata.description.push(value.into());
    }

    /// Remove all description paragraphs from EPUB
    pub fn clear_description(&mut self) {
        self.metadata.description.clear();
    }

    /// Sets and replaces the subjects of the EPUB
    pub fn set_subjects(&mut self, value: Vec<String>) {
        self.metadata.subject = value;
    }

    /// Adds a value to the subjects
    pub fn add_subject<S: Into<String>>(&mut self, value: S) {
        self.metadata.subject.push(value.into());
    }

    /// Remove all the subjects from EPUB
    pub fn clear_subjects(&mut self) {
        self.metadata.subject.clear();
    }

    /// Sets the license under which this EPUB is distributed
    pub fn set_license<S: Into<String>>(&mut self, value: S) {
        self.metadata.license = Some(value.into());
    }

    /// Sets the publication date of the EPUB
    pub fn set_publication_date(&mut self, date_published: chrono::DateTime<chrono::Utc>) {
        self.metadata.date_published = Some(date_published);
    }
    /// Sets the date on which the EPUB was last modified.
    ///
    /// This value is part of the metadata. If this function is not called, the time at the
    /// moment of generation will be used instead.
    pub fn set_modified_date(&mut self, date_modified: chrono::DateTime<chrono::Utc>) {
        self.metadata.date_modified = Some(date_modified);
    }
    /// Sets the uuid used for the EPUB.
    ///
    /// This is useful for reproducibly generating epubs.
    pub fn set_uuid(&mut self, uuid: uuid::Uuid) {
        self.metadata.uuid = Some(uuid);
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
    pub fn generate<W: io::Write>(mut self, to: W) -> Result<()> {
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
            optional.push(format!(
                "<dc:description>{}</dc:description>",
                common::encode_html(desc, self.escape_html),
            ));
        }
        for subject in &self.metadata.subject {
            optional.push(format!(
                "<dc:subject>{}</dc:subject>",
                common::encode_html(subject, self.escape_html),
            ));
        }
        if let Some(ref rights) = self.metadata.license {
            optional.push(format!(
                "<dc:rights>{}</dc:rights>",
                common::encode_html(rights, self.escape_html),
            ));
        }

        for meta in &self.meta_opf {
            optional.push(meta.render_opf(self.escape_html))
        }

        let date_modified = self
            .metadata
            .date_modified
            .unwrap_or_else(chrono::Utc::now)
            .format("%Y-%m-%dT%H:%M:%SZ");
        let date_published = self
            .metadata
            .date_published
            .map(|date| date.format("%Y-%m-%dT%H:%M:%SZ"));
        let uuid = uuid::fmt::Urn::from_uuid(self.metadata.uuid.unwrap_or_else(uuid::Uuid::new_v4))
            .to_string();

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
                (EpubVersion::V33, true) => "properties=\"cover-image\" ",
                _ => "",
            };
            if content.cover {
                optional.push("<meta name=\"cover\" content=\"cover-image\"/>".to_string());
            }
            log::debug!("id={:?}, mime={:?}", id, content.mime);
            items.push(format!(
                "<item media-type=\"{mime}\" {properties}\
                        id=\"{id}\" href=\"{href}\"/>",
                properties = properties, // Not escaped: XML attributes above
                mime = html_escape::encode_double_quoted_attribute(&content.mime),
                id = html_escape::encode_double_quoted_attribute(&id),
                // in the zip the path is always with forward slashes, on windows it is with backslashes
                href =
                    html_escape::encode_double_quoted_attribute(&content.file.replace('\\', "/")),
            ));
            if content.itemref {
                itemrefs.push(format!(
                    "<itemref idref=\"{id}\"/>",
                    id = html_escape::encode_double_quoted_attribute(&id),
                ));
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
                    reftype = html_escape::encode_double_quoted_attribute(&reftype),
                    title = html_escape::encode_double_quoted_attribute(&content.title),
                    href = html_escape::encode_double_quoted_attribute(&content.file),
                ));
            }
        }

        let data = {
            let mut authors: Vec<_> = vec![];
            for (i, author) in self.metadata.author.iter().enumerate() {
                let author = upon::value! {
                    id_attr: html_escape::encode_double_quoted_attribute(&i.to_string()),
                    name: common::encode_html(author, self.escape_html)
                };
                authors.push(author);
            }
            let mut languages: Vec<_> = vec![];
            for (i, lang) in self.metadata.lang.iter().enumerate() {
                let lang = upon::value! {
                    id_attr: html_escape::encode_double_quoted_attribute(&i.to_string()),
                    name: common::encode_html(lang, self.escape_html)
                };
                languages.push(lang);
            }
            upon::value! {
                author: authors,
                lang: languages,
                direction: self.metadata.direction.to_string(),
                title: common::encode_html(&self.metadata.title, self.escape_html),
                generator_attr: html_escape::encode_double_quoted_attribute(&self.metadata.generator),
                toc_name: common::encode_html(&self.metadata.toc_name, self.escape_html),
                toc_name_attr: html_escape::encode_double_quoted_attribute(&self.metadata.toc_name),
                optional: common::indent(optional.join("\n"), 2),
                items: common::indent(items.join("\n"), 2), // Not escaped: XML content
                itemrefs: common::indent(itemrefs.join("\n"), 2), // Not escaped: XML content
                date_modified: html_escape::encode_text(&date_modified.to_string()),
                uuid: html_escape::encode_text(&uuid),
                guide: common::indent(guide.join("\n"), 2), // Not escaped: XML content
                date_published: if let Some(date) = date_published { date.to_string() } else { String::new() },
            }
        };

        let mut res: Vec<u8> = vec![];
        match self.version {
            EpubVersion::V20 => templates::v2::CONTENT_OPF.render(&Engine::new(), &data).to_writer(&mut res),
            EpubVersion::V30 => templates::v3::CONTENT_OPF.render(&Engine::new(), &data).to_writer(&mut res),
            EpubVersion::V33 => templates::v3::CONTENT_OPF.render(&Engine::new(), &data).to_writer(&mut res),
        }
        .map_err(|e| crate::Error::TemplateError {
            msg: "could not render template for content.opf".to_string(),
            cause: e.into(),
        })?;
        //.wrap_err("could not render template for content.opf")?;

        Ok(res)
    }

    /// Render toc.ncx
    fn render_toc(&mut self) -> Result<Vec<u8>> {
        let mut nav_points = String::new();

        nav_points.push_str(&self.toc.render_epub(self.escape_html));

        let data = upon::value! {
            toc_name: common::encode_html(&self.metadata.toc_name, self.escape_html),
            nav_points: nav_points
        };
        let mut res: Vec<u8> = vec![];
        templates::TOC_NCX
            .render(&Engine::new(), &data)
            .to_writer(&mut res)
            .map_err(|e| crate::Error::TemplateError {
                msg: "error rendering toc.ncx template".to_string(),
                cause: e.into(),
            })?;
        Ok(res)
    }

    /// Render nav.xhtml
    fn render_nav(&mut self, numbered: bool) -> Result<Vec<u8>> {
        let content = self.toc.render(numbered, self.escape_html);
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
                            reftype = html_escape::encode_double_quoted_attribute(&reftype),
                            href = html_escape::encode_double_quoted_attribute(&file.file),
                            title = common::encode_html(&file.title, self.escape_html),
                        ));
                    }
                }
            }
        }

        let data = upon::value! {
            content: content, // Not escaped: XML content
            toc_name: common::encode_html(&self.metadata.toc_name, self.escape_html),
            generator_attr: html_escape::encode_double_quoted_attribute(&self.metadata.generator),
            landmarks: if !landmarks.is_empty() {
                common::indent(
                    format!(
                        "<ol>\n{}\n</ol>",
                        common::indent(landmarks.join("\n"), 1), // Not escaped: XML content
                    ),
                    2,
                )
            } else {
                String::new()
            },
        };

        let mut res: Vec<u8> = vec![];
        match self.version {
            EpubVersion::V20 => templates::v2::NAV_XHTML.render(&Engine::new(), &data).to_writer(&mut res),
            EpubVersion::V30 => templates::v3::NAV_XHTML.render(&Engine::new(), &data).to_writer(&mut res),
            EpubVersion::V33 => templates::v3::NAV_XHTML.render(&Engine::new(), &data).to_writer(&mut res),
        }
        .map_err(|e| crate::Error::TemplateError {
            msg: "error rendering nav.xhtml template".to_string(),
            cause: e.into(),
        })?;
        Ok(res)
    }
}

// The actual rules for ID are here - https://www.w3.org/TR/xml-names11/#NT-NCNameChar
// Ordering to to look as similar as possible to the W3 Recommendation ruleset
// Slightly more permissive, there are some that are invalid start chars, but this is ok.
fn is_id_char(c: char) -> bool {
    c.is_ascii_uppercase()
        || c == '_'
        || c.is_ascii_lowercase()
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
        || c.is_ascii_digit()
        || c == '\u{B7}'
        || ('\u{0300}'..='\u{036F}').contains(&c)
        || ('\u{203F}'..='\u{2040}').contains(&c)
}

// generate an id compatible string, replacing all none ID chars to underscores
fn to_id(s: &str) -> String {
    "id_".to_string() + &s.replace(|c: char| !is_id_char(c), "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A `Zip` that discards everything: these tests only look at the rendered metadata.
    #[derive(Debug)]
    struct NullZip;

    impl Zip for NullZip {
        fn write_file<P: AsRef<Path>, R: Read>(&mut self, _file: P, _content: R) -> Result<()> {
            Ok(())
        }

        fn generate<W: Write>(self, _to: W) -> Result<()> {
            Ok(())
        }
    }

    /// Every key with the string `metadata()` accepts for it. This is deliberately written out
    /// literally, not derived from `as_str()`, so that it is an independent specification.
    const KEYS: [(MetadataKey, &str); 9] = [
        (MetadataKey::Author, "author"),
        (MetadataKey::Title, "title"),
        (MetadataKey::Lang, "lang"),
        (MetadataKey::Direction, "direction"),
        (MetadataKey::Generator, "generator"),
        (MetadataKey::Description, "description"),
        (MetadataKey::Subject, "subject"),
        (MetadataKey::License, "license"),
        (MetadataKey::TocName, "toc_name"),
    ];

    fn builder(version: EpubVersion) -> EpubBuilder<NullZip> {
        let mut builder = EpubBuilder::new(NullZip).unwrap();
        builder.epub_version(version);
        builder
    }

    fn opf(builder: &mut EpubBuilder<NullZip>) -> String {
        String::from_utf8(builder.render_opf().unwrap()).unwrap()
    }

    /// Renders every file that contains metadata, with the parts that vary between runs fixed.
    fn render_all(builder: &mut EpubBuilder<NullZip>) -> String {
        builder.set_uuid(uuid::Uuid::nil());
        builder.set_modified_date(chrono::DateTime::UNIX_EPOCH);
        let nav = builder.render_nav(true).unwrap();
        let ncx = builder.render_toc().unwrap();
        format!(
            "{}\n{}\n{}",
            opf(builder),
            String::from_utf8(nav).unwrap(),
            String::from_utf8(ncx).unwrap()
        )
    }

    #[test]
    fn metadata_key_strings() {
        for (key, string) in KEYS {
            assert_eq!(key.as_str(), string);
            assert_eq!(AsRef::<str>::as_ref(&key), string);
            assert_eq!(key.to_string(), string);
            assert_eq!(string.parse::<MetadataKey>().unwrap(), key);
        }
    }

    #[test]
    fn metadata_key_all_is_in_sync_with_the_variants() {
        // No wildcard on purpose: adding a variant breaks the build here, as a reminder to also
        // add it to `KEYS` above and to `MetadataKey::ALL`, which `FromStr` relies on.
        for (key, _) in KEYS {
            match key {
                MetadataKey::Author
                | MetadataKey::Title
                | MetadataKey::Lang
                | MetadataKey::Direction
                | MetadataKey::Generator
                | MetadataKey::Description
                | MetadataKey::Subject
                | MetadataKey::License
                | MetadataKey::TocName => {}
            }
        }
        let keys: Vec<MetadataKey> = KEYS.iter().map(|(key, _)| *key).collect();
        assert_eq!(MetadataKey::ALL.to_vec(), keys);
    }

    #[test]
    fn metadata_key_from_str_rejects_unknown_strings() {
        for s in [
            "",
            "foo",
            "identifier",
            "Title",
            "TITLE",
            " title",
            "title ",
            "toc-name",
            "tocname",
        ] {
            match s.parse::<MetadataKey>() {
                Err(crate::Error::InvalidMetadataError(got)) => assert_eq!(got, s),
                other => panic!("{s:?} should be rejected, got {other:?}"),
            }
        }
    }

    #[test]
    fn metadata_typed_and_string_keys_are_equivalent() {
        for (key, string) in KEYS {
            let value = if key == MetadataKey::Direction {
                "rtl"
            } else {
                "A value"
            };
            let mut typed = builder(EpubVersion::V30);
            typed.metadata(key, value).unwrap();
            let mut untyped = builder(EpubVersion::V30);
            untyped.metadata(string, value).unwrap();
            assert_eq!(
                render_all(&mut typed),
                render_all(&mut untyped),
                "key {key}"
            );
        }
    }

    // The borrowed keys are what is being tested here, so the borrows are not needless.
    #[allow(clippy::needless_borrows_for_generic_args)]
    #[test]
    fn metadata_accepts_any_key_that_is_as_ref_str() {
        let mut builder = builder(EpubVersion::V20);
        let title_is = |builder: &mut EpubBuilder<NullZip>, title: &str| {
            assert!(opf(builder).contains(&format!("<dc:title>{title}</dc:title>")));
        };

        builder.metadata("title", "from str").unwrap();
        title_is(&mut builder, "from str");
        builder
            .metadata(String::from("title"), "from String")
            .unwrap();
        title_is(&mut builder, "from String");
        builder
            .metadata(&String::from("title"), "from ref String")
            .unwrap();
        title_is(&mut builder, "from ref String");
        builder
            .metadata(std::borrow::Cow::Borrowed("title"), "from Cow")
            .unwrap();
        title_is(&mut builder, "from Cow");
        builder
            .metadata(MetadataKey::Title, "from MetadataKey")
            .unwrap();
        title_is(&mut builder, "from MetadataKey");
        builder
            .metadata(&MetadataKey::Title, "from ref MetadataKey")
            .unwrap();
        title_is(&mut builder, "from ref MetadataKey");
    }

    #[test]
    fn metadata_invalid_string_key_is_still_an_error() {
        let mut builder = builder(EpubVersion::V20);
        builder.metadata(MetadataKey::Title, "Kept").unwrap();
        match builder.metadata("titel", "Typo").err() {
            Some(crate::Error::InvalidMetadataError(key)) => assert_eq!(key, "titel"),
            other => panic!("expected InvalidMetadataError, got {other:?}"),
        }
        assert!(opf(&mut builder).contains("<dc:title>Kept</dc:title>"));
    }

    #[test]
    fn metadata_single_value_keys_replace_the_previous_value() {
        let mut builder = builder(EpubVersion::V20);
        builder
            .metadata(MetadataKey::Title, "First title")
            .unwrap()
            .metadata(MetadataKey::Title, "Second title")
            .unwrap()
            .metadata(MetadataKey::License, "First license")
            .unwrap()
            .metadata(MetadataKey::License, "Second license")
            .unwrap()
            .metadata(MetadataKey::Generator, "First generator")
            .unwrap()
            .metadata(MetadataKey::Generator, "Second generator")
            .unwrap()
            .metadata(MetadataKey::TocName, "First contents")
            .unwrap()
            .metadata(MetadataKey::TocName, "Second contents")
            .unwrap();
        let opf = opf(&mut builder);
        let nav = String::from_utf8(builder.render_nav(true).unwrap()).unwrap();
        assert!(opf.contains("<dc:title>Second title</dc:title>"));
        assert!(opf.contains("<dc:rights>Second license</dc:rights>"));
        assert!(opf.contains("title=\"Second contents\""));
        assert!(nav.contains("<meta name=\"generator\" content=\"Second generator\" />"));
        assert!(nav.contains("<h1 id=\"toc-title\">Second contents</h1>"));
        for first in ["First title", "First license", "First contents"] {
            assert!(!opf.contains(first), "{first} should have been replaced");
        }
        assert!(!nav.contains("First"));
    }

    #[test]
    fn metadata_multiple_value_keys_accumulate_until_cleared() {
        let cases = [
            (
                MetadataKey::Author,
                "<dc:creator opf:role=\"aut\">",
                "</dc:creator>",
            ),
            (MetadataKey::Lang, "<dc:language>", "</dc:language>"),
            (
                MetadataKey::Description,
                "<dc:description>",
                "</dc:description>",
            ),
            (MetadataKey::Subject, "<dc:subject>", "</dc:subject>"),
        ];
        for (key, open, close) in cases {
            let rendered = |value: &str| format!("{open}{value}{close}");
            let mut builder = builder(EpubVersion::V20);
            builder
                .metadata(key, "one")
                .unwrap()
                .metadata(key, "two")
                .unwrap();
            let opf_two_values = opf(&mut builder);
            let (one, two) = (
                opf_two_values.find(&rendered("one")),
                opf_two_values.find(&rendered("two")),
            );
            assert!(one.is_some() && two.is_some(), "{key}: {opf_two_values}");
            assert!(one < two, "{key}: values should keep their order");

            // The empty string removes every value...
            builder.metadata(key, "").unwrap();
            let opf_cleared = opf(&mut builder);
            assert!(
                !opf_cleared.contains(&rendered("one")),
                "{key}: {opf_cleared}"
            );
            assert!(
                !opf_cleared.contains(&rendered("two")),
                "{key}: {opf_cleared}"
            );

            // ... and the key can be used again afterwards.
            builder.metadata(key, "three").unwrap();
            assert!(opf(&mut builder).contains(&rendered("three")), "{key}");
        }
    }

    #[test]
    fn metadata_direction_is_validated() {
        let mut builder = builder(EpubVersion::V30);
        assert!(opf(&mut builder).contains("page-progression-direction=\"ltr\""));

        builder.metadata(MetadataKey::Direction, "rtl").unwrap();
        assert!(opf(&mut builder).contains("page-progression-direction=\"rtl\""));

        // Case does not matter
        builder.metadata(MetadataKey::Direction, "LTR").unwrap();
        assert!(opf(&mut builder).contains("page-progression-direction=\"ltr\""));

        // An invalid value is an error, and leaves the direction as it was
        builder.metadata(MetadataKey::Direction, "rtl").unwrap();
        match builder.metadata(MetadataKey::Direction, "up").err() {
            Some(crate::Error::PageDirectionError(value)) => assert_eq!(value, "up"),
            other => panic!("expected PageDirectionError, got {other:?}"),
        }
        assert!(opf(&mut builder).contains("page-progression-direction=\"rtl\""));

        // EPUB 2 has no page progression direction
        let mut builder = self::builder(EpubVersion::V20);
        builder.metadata(MetadataKey::Direction, "rtl").unwrap();
        assert!(!opf(&mut builder).contains("page-progression-direction"));
    }
}
