extern crate epub;

use epub::Epub;
use epub::ZipCommand;
use epub::ZipLibrary;
use epub::Zip;
use epub::EpubContent;
use epub::TocElement;

use std::io;
use std::io::Write;

fn main() {
    let mut epub = Epub::new(ZipLibrary::new()).unwrap();
    epub.metadata("author", "Lise").unwrap()
        .metadata("title", "Test").unwrap()
        .add_resource("foo.txt", "coin coin".as_bytes(), "text").unwrap()
        .add_content(EpubContent::new("chapter_1.xhtml", "A false chapter".as_bytes())
                     .title("Chapter 1")).unwrap()
        .add_content(EpubContent::new("chapter_2.xhtml", "Other false chapter".as_bytes())
                     .title("Chapter 2")
                     .child(TocElement::new("chapter_2.xhtml#1", "2-1"))).unwrap()
        .add_content(EpubContent::new("section.xhtml", "false section".as_bytes())
                     .title("2-2")
                     .level(2)).unwrap()
        .add_content(EpubContent::new("notes.xhtml",
                                      "false unnamed chapter".as_bytes())).unwrap()
        .inline_toc();
        
//    println!("{:?}", epub);
    epub.generate(&mut io::stdout()).unwrap();
}
