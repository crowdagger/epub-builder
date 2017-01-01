extern crate epub;

use epub::Epub;
use epub::ZipCommand;
use epub::ZipLibrary;
use epub::Zip;

use std::io;
use std::io::Write;

fn main() {
    let mut zip = ZipLibrary::new();
    let foo = "Coin! Coin!";
    zip.write_file("test.txt", foo.as_bytes()).unwrap();
    match zip.generate() {
        Ok(res) =>  io::stdout().write_all(&res).unwrap(),
        Err(err) => println!("{}", err)
    }
}
