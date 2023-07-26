use epub_builder::EpubBuilder;
use epub_builder::EpubContent;
use epub_builder::ReferenceType;
use epub_builder::Result;
use epub_builder::ZipLibrary;

use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;


// Try to print Zip file to stdout
fn create_book(outpath: &Path) -> Result<()> {
    let writer = File::create(outpath).unwrap();

    // Create a new EpubBuilder using the zip library
    EpubBuilder::new(ZipLibrary::new()?)?
        // Set some metadata
        .metadata("author", "Wikipedia Contributors")?
        .metadata("title", "Ada Lovelace: first programmer")?
        // // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
        .stylesheet(File::open("examples/book/book.css")?)?
        // Add a image cover file
        .add_cover_image("cover.png", File::open("examples/book/Ada_Lovelace_color.svg")?, "image/svg")?

        // Add a title page
        // .add_content(
        //     EpubContent::new("title.xhtml", File::open("examples/book/title.html")?)
        //         .title("First Programmer")
        //         .reftype(ReferenceType::TitlePage),
        // )?
        // Generate a toc inside of the document, that will be part of the linear structure.
        .inline_toc()
        // add text of first chapter 
        .add_content(
            EpubContent::new("chapter_1.xhtml", File::open("examples/book/ch1.html")?)
                .title("First Programmer")
                .reftype(ReferenceType::Text),
        )?
        .add_content(
            EpubContent::new("chapter_2.xhtml", File::open("examples/book/ch2.html")?)
                .title("First computer program")
                .reftype(ReferenceType::Text),
        )?
        .generate(&writer)?; // generate into file to see epub

    log::debug!("sample book generation is done");
    Ok(())
}

fn main() {
    env_logger::init();

    // output path for ebook
    let curr_dir = env::current_dir().unwrap();
    let outpath = curr_dir.join("book.epub");
    log::debug!("write file to: {}", &outpath.display());

    match create_book(&outpath) {
        Ok(_) => writeln!(
            &mut io::stderr(),
            "Successfully wrote epub document"
        )
        .unwrap(),
        Err(err) => writeln!(&mut io::stderr(), "Error: {:?}", err).unwrap(),
    };
}
