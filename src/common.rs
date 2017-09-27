// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use regex::Regex;

use std::borrow::Cow;

/// Escape quotes from the string
pub fn escape_quote<'a, S:Into<Cow<'a, str>>>(s: S) -> Cow<'a, str> {
    lazy_static! {
        static ref REGEX:Regex = Regex::new(r#"""#).unwrap();
    }

    let s = s.into();
    if REGEX.is_match(&s) {
        let res = REGEX.replace_all(&s, "\\\"").into_owned();
        Cow::Owned(res)
    } else {
        s
    }
}


#[test]
fn test_escape() {
    let foo = "Some string with \"quote\"";
    assert_eq!(&escape_quote(foo), "Some string with \\\"quote\\\"");

    let bar= "Some string without quote";
    assert_eq!(&escape_quote(bar), bar);
}
