[See the full library documentation on Docs.rs.](https://docs.rs/epub)

# epub-builder

A library to generate EPUB files.

The purpose for this library is to make it easier to generate EPUB files:
it should take care of most of the boilerplate for you, leaving you only
with the task of filling the actual content.

## Usage

Add this in your `Cargo.toml` file:

```toml, ignore
[dependencies]
epub-builder: "0.1"
```

## Example

```rust
extern crate epub_builder;

use epub_builder::EpubBuilder;
use epub_builder::Result;
use epub_builder::ZipLibrary;
use epub_builder::EpubContent;
use epub_builder::TocElement;

use std::io;
use std::io::Write;

// Try to print Zip file to stdout
fn run() -> Result<()> {
    // Some dummy content to fill our books
    let dummy_content = "Dummy content. This should be valid XHTML if you want a valid EPUB!";
    let dummy_cover = "Not really a PNG image";
    let dummy_css = "body { background-color: pink }";

    // Create a new EpubBuilder using the zip library
    EpubBuilder::new(ZipLibrary::new())?
    // Set some metadata
        .metadata("author", "Joan Doe")?
        .metadata("title", "Dummy Book")?
    // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
        .stylesheet(dummy_css.as_bytes())?
    // Add a resource that is not part of the linear document structure
        .add_resource("cover.png", dummy_cover.as_bytes(), "image/png")?
    // Add a chapter
        .add_content(EpubContent::new("chapter_1.xhtml", dummy_content.as_bytes())
                     .title("Chapter 1"))?
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
        .generate(&mut io::sink())?;
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => writeln!(&mut io::stdout(), "Successfully wrote epub document to stdout!").unwrap(),
        Err(err) => writeln!(&mut io::stderr(), "Error: {}", err).unwrap(),
    };
}
```

## Conditional compilation

EPUB files are Zip files, so we need to zip them. By default, this library features
wrappers around both the [Rust zip library](https://crates.io/crates/zip) and calls
to the `zip` command that may (or may not) be installed on your system.

It is possible to disable the compilation (and the dependencies) of either of this
wrapper, using `no-defaut-features`. (If you don't enable at least one of them this
library will be pretty useless).

## License

This is free software, published under the [Mozilla Public License,
version 2.0](https://www.mozilla.org/en-US/MPL/2.0/).

## ChangeLog

See the [ChangeLog file](ChangeLog.md).
