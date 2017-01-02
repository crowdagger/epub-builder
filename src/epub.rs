// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use templates;
use errors::Result;
use errors::ResultExt;
use zip::Zip;
use toc::Toc;
use toc::TocElement;
use epub_content::EpubContent;

use std::io::Read;
use std::io::Write;
use std::path::Path;

use chrono;
use uuid;
use mustache::MapBuilder;

/// Represents a EPUB version
#[derive(Debug, Copy, Clone)]
pub enum EpubVersion {
    /// EPUB 2 format
    V2,
    /// EPUB 3.0 format
    V3_0, 
}

/// EPUB Metadata
#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub author: String,
    pub lang: String,
    pub generator: String,
    pub toc_name: String,
    pub description: Option<String>,
    pub subject: Option<String>,
    pub license: Option<String>,
}

impl Metadata {
    /// Create new default metadata
    pub fn new() -> Metadata {
        Metadata {
            title: String::new(),
            author: String::new(),
            lang: String::from("en"),
            generator: String::from("Rust EPUB library"),
            toc_name: String::from("Table Of Contents"),
            description: None,
            subject: None,
            license: None,
        }
    }
}

/// A file added in an EPUB
#[derive(Debug)]
struct Content {
    pub file: String,
    pub mime: String,
    pub itemref: bool,
}

impl Content {
    /// Create a new content file
    pub fn new<S1:Into<String>, S2: Into<String>>(file: S1, mime: S2) -> Content {
        Content {
            file: file.into(),
            mime: mime.into(),
            itemref: false,
        }
    }
}

/// Epub files generator
#[derive(Debug)]
pub struct Epub<Z:Zip> {
    version: EpubVersion,
    zip: Z,
    files: Vec<Content>,
    metadata: Metadata,
    toc: Toc,
    stylesheet: bool,
    inline_toc: bool,
}

impl<Z:Zip> Epub<Z> {
    /// Create a new default EPUB Generator
    pub fn new(zip: Z) -> Result<Epub<Z>> {
        let mut epub = Epub {
            version: EpubVersion::V2,
            zip: zip,
            files: vec!(),
            metadata: Metadata::new(),
            toc: Toc::new(),
            stylesheet: false,
            inline_toc: false,
        };
        
        // Write mimetype upfront
        epub.zip.write_file("mimetype", "application/epub+zip".as_bytes())?;
        epub.zip.write_file("META-INF/container.xml", templates::CONTAINER)?;
        epub.zip.write_file("META-INF/com.apple.ibooks.display-options.xml", templates::IBOOKS)?;

        Ok(epub)
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
    /// * `license`
    /// * `toc_name`
    pub fn metadata<S1: AsRef<str>, S2: Into<String>>(&mut self, key: S1, value: S2) -> Result<&mut Self> {
        match key.as_ref() {
            "author" => self.metadata.author = value.into(),
            "title" => self.metadata.title = value.into(),
            "lang" => self.metadata.lang = value.into(),
            "generator" => self.metadata.generator = value.into(),
            "description" => self.metadata.description = Some(value.into()),
            "subject" => self.metadata.subject = Some(value.into()),
            "license" => self.metadata.license = Some(value.into()),
            "toc_name" => self.metadata.toc_name = value.into(),
            s => bail!("invalid metadata '{}'", s),
        }
        Ok(self)
    }

    /// Sets stylesheet of the EPUB.
    ///
    /// This content will be written in a `stylesheet.css` file; it is used by
    /// some pages (such as nav.xhtml), you don't have use it in your documents though it
    /// makes sense to also do so.
    pub fn stylesheet<R:Read>(&mut self, content: R) -> Result<&mut Self> {
        self.add_resource("stylesheet.css", content, "text/css")?;
        self.stylesheet = true;
        Ok(self)
    }

    /// Adds an inline toc in the document.
    ///
    /// If this method is called it adds a TOC similar to nav.xhtml but listed in the inline document.
    pub fn inline_toc(&mut self) -> &mut Self {
        self.inline_toc = true;
        self.toc.add(TocElement::new("toc.xhtml", self.metadata.toc_name.as_ref()));
        let mut file = Content::new("toc.xhtml", "application/xhtml+xml");
        file.itemref = true;
        self.files.push(file);
        self
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
    pub fn add_resource<R: Read, P: AsRef<Path>, S: Into<String>>(&mut self,
                                                                     path: P,
                                                                     content: R,
                                                                     mime_type: S) -> Result<&mut Self> {
        self.zip.write_file(Path::new("OEBPS").join(path.as_ref()), content)?;
        self.files.push(Content::new(format!("{}", path.as_ref().display()), mime_type));
        Ok(self)
    }
    
    /// Add a XHTML content file that will be added to the EPUB.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// epub.add_content(EpubContent::new("intro.xhtml", file));
    /// ```
    ///
    /// ```ignore
    /// epub.add_content(EpubContent::new("chapter_1.xhtml", File::open("chapter_1.xhtml")?)
    ///                      .title("Chapter 1")
    ///                      .child(TocElement::new("chapter_1.xhtml#1", "1.1")));
    /// ```
    ///
    /// * `level`: the level this content will be added in the toc;
    /// * `title`: the title of this content, as it should appear in the TOC;
    /// * `inner_toc`: a table of contents descrbing the inner layout of the content;
    /// * `content`: should be the contents of an XHTML file.
    pub fn add_content<R: Read>(&mut self, content: EpubContent<R>)-> Result<&mut Self> {
        self.zip.write_file(Path::new("OEBPS").join(content.toc.url.as_str()),
                            content.content)?;
        let mut file = Content::new(content.toc.url.as_ref(),
                                "application/xhtml+xml");
        file.itemref = true;
        self.files.push(file);
        if !content.toc.title.is_empty() {
            self.toc.add(content.toc);
        }
        Ok(self)
    }

    /// Generate the EPUB file and write it to the writer
    pub fn generate<W: Write>(&mut self, mut to: W) -> Result<()> {
        // If no styleesheet was provided, generate a dummy one
        if !self.stylesheet {
            self.stylesheet("".as_bytes())?;
        }
        /// Render content.opf
        let bytes = self.render_opf()?;
        self.zip.write_file("OEBPS/content.opf", &bytes as &[u8])?;
        /// Render toc.ncx
        let bytes = self.render_toc()?;
        self.zip.write_file("OEBPS/toc.ncx", &bytes as &[u8])?;
        // Render nav.xhtml
        let bytes = self.render_nav(true)?;
        self.zip.write_file("OEBPS/nav.xhtml", &bytes as &[u8])?;
        // Write inline toc if it needs to
        if self.inline_toc {
            let bytes = self.render_nav(false)?;
            self.zip.write_file("OEBPS/toc.xhtml", &bytes as &[u8])?;
        }

        
        let res = self.zip.generate()?;
        to.write_all(res.as_ref())
            .chain_err(|| "error writing final result of EPUB content")?;
        
        Ok(())
    }

    /// Render content.opf file
    fn render_opf(&mut self) -> Result<Vec<u8>> {
        let mut optional = String::new();
        if let Some(ref desc) = self.metadata.description {
            optional.push_str(&format!("<dc:description>{}</dc:description>\n", desc));
        }
        if let Some(ref subject) = self.metadata.subject {
            optional.push_str(&format!("<dc:subject>{}</dc:subject>\n", subject));
        }
        if let Some(ref rights) = self.metadata.license {
            optional.push_str(&format!("<dc:rights>{}</dc:rights>\n", rights));
        }
        let date = chrono::UTC::now().format("%Y-%m-%dT%H:%M:%SZ");
        let uuid = uuid::Uuid::new_v4().urn().to_string();

        let mut items = String::new();
        let mut itemrefs = String::new();

        for content in self.files.iter() {
            items.push_str(&format!("<item media-type = \"{mime}\" \
                                     id = \"{id}\" \
                                     href = \"{href}\" />\n",
                                    mime = content.mime,
                                    id = to_id(&content.file),
                                    href = content.file));
            if content.itemref {
                itemrefs.push_str(&format!("<itemref idref = \"{id}\" />\n",
                                           id = to_id(&content.file)));
                                            
            }
        }

        let data = MapBuilder::new()
            .insert_str("lang", &self.metadata.lang)
            .insert_str("author", &self.metadata.author)
            .insert_str("title", &self.metadata.title)
            .insert_str("generator", &self.metadata.generator)
            .insert_str("optional", optional)
            .insert_str("items", items)
            .insert_str("itemrefs", itemrefs)
            .insert_str("date", date)
            .insert_str("uuid", uuid)
            .build();

        let mut content = vec!();
        let res = match self.version {
            EpubVersion::V2 => templates::v2::CONTENT_OPF.render_data(&mut content, &data),
            EpubVersion::V3_0 => templates::v3::CONTENT_OPF.render_data(&mut content, &data),
        };
        

        res
            .chain_err(|| "could not render template for content.opf")?;

        Ok(content)
    }

        /// Render toc.ncx
    fn render_toc(&mut self) -> Result<Vec<u8>> {
        let mut nav_points = String::new();

        nav_points.push_str(&self.toc.render_epub());

        let data = MapBuilder::new()
            .insert_str("toc_name", &self.metadata.toc_name)
            .insert_str("nav_points", nav_points)
            .build();
        let mut res: Vec<u8> = vec![];
        templates::TOC_NCX.render_data(&mut res, &data)
            .chain_err(|| "error rendering toc.ncx template")?;
        Ok(res)
    }


    /// Render nav.xhtml
    fn render_nav(&mut self, numbered: bool) -> Result<Vec<u8>> {
        let content = self.toc.render(numbered);
        let data = MapBuilder::new()
            .insert_str("content", content)
            .insert_str("toc_name", &self.metadata.toc_name)
            .insert_str("generator", &self.metadata.generator)
            .build();

        let mut res = vec!();
        let eh = match self.version {
            EpubVersion::V2 => templates::v2::NAV_XHTML.render_data(&mut res, &data),
            EpubVersion::V3_0 => templates::v3::NAV_XHTML.render_data(&mut res, &data)
        };
        
        eh.chain_err(|| "error rendering nav.xhtml template")?;
        Ok(res)
    }
}


// generate an id compatible string, replacing / and . by _
fn to_id(s: &str) -> String {
    s.replace(".", "_").replace("/", "_")
}
