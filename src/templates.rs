// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.


pub static IBOOKS: &'static [u8] = include_bytes!("../templates/ibooks.xml");
pub static CONTAINER: &'static [u8] = include_bytes!("../templates/container.xml");

lazy_static! {
    pub static ref TOC_NCX: ::mustache::Template = ::mustache::compile_str(include_str!("../templates/toc.ncx")).unwrap();
}

pub mod v2 {
    lazy_static! {
        pub static ref CONTENT_OPF: ::mustache::Template = ::mustache::compile_str(include_str!("../templates/v2/content.opf")).unwrap();
        pub static ref NAV_XHTML: ::mustache::Template = ::mustache::compile_str(include_str!("../templates/v2/nav.xhtml")).unwrap();
    }
}
pub mod v3 {
    lazy_static! {
        pub static ref CONTENT_OPF: ::mustache::Template = ::mustache::compile_str(include_str!("../templates/v3/content.opf")).unwrap();
        pub static ref NAV_XHTML: ::mustache::Template = ::mustache::compile_str(include_str!("../templates/v3/nav.xhtml")).unwrap();
    }
}
