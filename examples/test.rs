extern crate epub;

use epub::Epub;
use epub::ZipCommand;
use epub::ZipLibrary;
use epub::Zip;

use std::io;
use std::io::Write;

fn main() {
    let mut epub = Epub::new(ZipLibrary::new()).unwrap();
    epub.metadata("author", "Lise").unwrap()
        .metadata("title", "Test").unwrap();
    epub.add_resource("foo.txt", "coin coin".as_bytes(), "text");
//    println!("{:?}", epub);
    epub.generate(&mut io::stdout()).unwrap();
}
