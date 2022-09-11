use epub_builder::EpubBuilder;
use epub_builder::EpubContent;
use epub_builder::ReferenceType;
use epub_builder::Result;
use epub_builder::TocElement;
use epub_builder::ZipLibrary;

use std::env;
use std::fs::File;
use std::io;
use std::io::Write;

const IMAGE_HEX: &str = "\
    89504e470d0a1a0a0000000d49484452\
    00000001000000010100000000376ef9\
    240000001049444154789c6260010000\
    00ffff03000006000557bfabd4000000\
    0049454e44ae426082";

// Try to print Zip file to stdout
fn run() -> Result<()> {
    env_logger::init();
    // Some dummy content to fill our books
    let dummy_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title></title></head>
<body>
<p>Text of the page<T></p>
</body>
</html>"#;
    let dummy_content_with_id = dummy_content.replace("<p>", "<p id=\"p1\">");
    let dummy_image: Vec<u8> = (0..IMAGE_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&IMAGE_HEX[i..i + 2], 16).unwrap())
        .collect();
    let dummy_css = "body { background-color: pink }";

    // temp file to see epub internals
    let _curr_dir = env::current_dir().unwrap();
    let _out_file = _curr_dir.join("temp_epub_file.epub");
    log::debug!("file to write = {}", &_out_file.display());
    let _writer = File::create(_out_file).unwrap();

    // Create a new EpubBuilder using the zip library
    EpubBuilder::new(ZipLibrary::new()?)?
        // Set some metadata
        .metadata("author", "Joan Doe & friends")?
        .metadata("title", "Dummy Book <T>")?
        // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
        .stylesheet(dummy_css.as_bytes())?
        // Add a image cover file
        .add_cover_image("cover.png", dummy_image.as_slice(), "image/png")?
        // Add a resource that is not part of the linear document structure
        .add_resource("some_image.png", dummy_image.as_slice(), "image/png")?
        // Add a cover page
        .add_content(
            EpubContent::new("cover.xhtml", dummy_content.as_bytes())
                .title("Cover")
                .reftype(ReferenceType::Cover),
        )?
        // Add a title page
        .add_content(
            EpubContent::new("title.xhtml", dummy_content.as_bytes())
                .title("Title <T>")
                .reftype(ReferenceType::TitlePage),
        )?
        // Add a chapter, mark it as beginning of the "real content"
        .add_content(
            EpubContent::new("chapter_1.xhtml", dummy_content.as_bytes())
                .title("Chapter 1 <T>")
                .reftype(ReferenceType::Text),
        )?
        // Add a second chapter; this one has more toc information about its internal structure
        .add_content(
            EpubContent::new("chapter_2.xhtml", dummy_content_with_id.as_bytes())
                .title("Chapter 2 <T>")
                .child(TocElement::new(
                    "chapter_2.xhtml#p1",
                    "Chapter 2, section 1",
                )),
        )?
        // Add a section. Since its level is set to 2, it will be attached to the previous chapter.
        .add_content(
            EpubContent::new("section.xhtml", dummy_content.as_bytes())
                .title("Chapter 2 <T>, section 2")
                .level(2),
        )?
        // Add a chapter without a title, which will thus not appear in the TOC.
        .add_content(EpubContent::new("notes.xhtml", dummy_content.as_bytes()))?
        // Generate a toc inside of the document, that will be part of the linear structure.
        .inline_toc()
        // Finally, write the EPUB file to stdout
        .generate(&mut io::stdout())?; // generate into stout
                                       // .generate(&_writer)?; // generate into temp file to see epub internals
    log::debug!("dummy book generation is done");
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => writeln!(
            &mut io::stderr(),
            "Successfully wrote epub document to stdout!"
        )
        .unwrap(),
        Err(err) => writeln!(&mut io::stderr(), "Error: {}", err).unwrap(),
    };
}
