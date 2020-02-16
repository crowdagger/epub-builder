extern crate epub_builder;

use epub_builder::EpubBuilder;
use epub_builder::Result;
use epub_builder::ZipLibrary;
use epub_builder::EpubContent;
use epub_builder::ReferenceType;
use epub_builder::TocElement;

use std::io;
use std::io::Write;

// Try to print Zip file to stdout
fn run() -> Result<()> {
    // Some dummy content to fill our books
    let dummy_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<body>
<p>Text of the page</p>
</body>
</html>"#;
    let dummy_image = "Not really a PNG image";
    let dummy_css = "body { background-color: pink }";

    // Create a new EpubBuilder using the zip library
    EpubBuilder::new(ZipLibrary::new()?)?
    // Set some metadata
        .metadata("author", "Joan Doe")?
        .metadata("title", "Dummy Book")?
    // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
        .stylesheet(dummy_css.as_bytes())?
    // Add a image cover file
        .add_cover_image("cover.png", dummy_image.as_bytes(), "image/png")?
    // Add a resource that is not part of the linear document structure
        .add_resource("some_image.png", dummy_image.as_bytes(), "image/png")?
    // Add a cover page
        .add_content(EpubContent::new("cover.xhtml", dummy_content.as_bytes())
                     .title("Cover")
                     .reftype(ReferenceType::Cover))?
    // Add a title page
        .add_content(EpubContent::new("title.xhtml", dummy_content.as_bytes())
                     .title("Title")
                     .reftype(ReferenceType::TitlePage))?
    // Add a chapter, mark it as beginning of the "real content"
        .add_content(EpubContent::new("chapter_1.xhtml", dummy_content.as_bytes())
                     .title("Chapter 1")
                     .reftype(ReferenceType::Text))?
    // Add a second chapter; this one has more toc information about its internal structure
        .add_content(EpubContent::new("chapter_2.xhtml", dummy_content.as_bytes())
                     .title("Chapter 2")
                     .child(TocElement::new("chapter_2.xhtml#1", "Chapter 2, section 1")))?
    // Add a section. Since its level is set to 2, it will be attached to the previous chapter.
        .add_content(EpubContent::new("section.xhtml", dummy_content.as_bytes())
                     .title("Chapter 2, section 2")
                     .level(2))?
    // Add a chapter without a title, which will thus not appear in the TOC.
        .add_content(EpubContent::new("notes.xhtml", dummy_content.as_bytes()))?
    // Generate a toc inside of the document, that will be part of the linear structure.
        .inline_toc()
    // Finally, write the EPUB file to stdout
        .generate(&mut io::stdout())?;
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {
            writeln!(&mut io::stderr(),
                     "Successfully wrote epub document to stdout!")
                .unwrap()
        }
        Err(err) => writeln!(&mut io::stderr(), "Error: {}", err).unwrap(),
    };
}
