// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub static IBOOKS: &[u8] = include_bytes!("../templates/ibooks.xml");
pub static CONTAINER: &[u8] = include_bytes!("../templates/container.xml");

lazy_static! {
    pub static ref TOC_NCX: ::mustache::Template =
        ::mustache::compile_str(include_str!("../templates/toc.ncx"))
            .expect("error compiling 'toc.ncx' template'");
}

pub mod v2 {
    lazy_static! {
        pub static ref CONTENT_OPF: ::mustache::Template =
            ::mustache::compile_str(include_str!("../templates/v2/content.opf"))
                .expect("error compiling 'content.opf' (for EPUB 2.0) template");
        pub static ref NAV_XHTML: ::mustache::Template =
            ::mustache::compile_str(include_str!("../templates/v2/nav.xhtml"))
                .expect("error compiling 'nav.xhtml' (for EPUB 2.0) template");
    }
}
pub mod v3 {
    lazy_static! {
        pub static ref CONTENT_OPF: ::mustache::Template =
            ::mustache::compile_str(include_str!("../templates/v3/content.opf"))
                .expect("error compiling 'content.opf' (for EPUB 3.0) template");
        pub static ref NAV_XHTML: ::mustache::Template =
            ::mustache::compile_str(include_str!("../templates/v3/nav.xhtml"))
                .expect("error compiling 'nav.xhtml' (for EPUB 3.0) template");
    }
}
