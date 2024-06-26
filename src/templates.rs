// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use once_cell::sync::Lazy;

pub static IBOOKS: &[u8] = include_bytes!("../templates/ibooks.xml");
pub static CONTAINER: &[u8] = include_bytes!("../templates/container.xml");

static ENGINE: Lazy<::upon::Engine> = Lazy::new(|| {
    let mut engine = ::upon::Engine::new();
    engine.add_filter("eq", str::eq);
    engine
});

pub static TOC_NCX: Lazy<::upon::Template> = Lazy::new(|| {
    ENGINE
        .compile(include_str!("../templates/toc.ncx"))
        .expect("error compiling 'toc.ncx' template'")
});

pub mod v2 {
    use crate::templates::ENGINE;
    use once_cell::sync::Lazy;

    pub static CONTENT_OPF: Lazy<::upon::Template> = Lazy::new(|| {
        ENGINE
            .compile(include_str!("../templates/v2/content.opf"))
            .expect("error compiling 'content.opf' (for EPUB 2.0) template")
    });
    pub static NAV_XHTML: Lazy<::upon::Template> = Lazy::new(|| {
        ENGINE
            .compile(include_str!("../templates/v2/nav.xhtml"))
            .expect("error compiling 'nav.xhtml' (for EPUB 2.0) template")
    });
}
pub mod v3 {
    use crate::templates::ENGINE;
    use once_cell::sync::Lazy;

    pub static CONTENT_OPF: Lazy<::upon::Template> = Lazy::new(|| {
        ENGINE
            .compile(include_str!("../templates/v3/content.opf"))
            .expect("error compiling 'content.opf' (for EPUB 3.0) template")
    });
    pub static NAV_XHTML: Lazy<::upon::Template> = Lazy::new(|| {
        ENGINE
            .compile(include_str!("../templates/v3/nav.xhtml"))
            .expect("error compiling 'nav.xhtml' (for EPUB 3.0) template")
    });
}
