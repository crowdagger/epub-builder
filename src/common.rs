// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use regex::Regex;

use std::borrow::Cow;

/// Escape quotes from the string
pub fn escape_quote<'a, S: Into<Cow<'a, str>>>(s: S) -> Cow<'a, str> {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(r#"""#).unwrap();
    }

    let s = s.into();
    if REGEX.is_match(&s) {
        let res = REGEX.replace_all(&s, "&quot;").into_owned();
        Cow::Owned(res)
    } else {
        s
    }
}

/// Indent lines of the string
pub fn indent<S: AsRef<str>>(s: S, level: usize) -> String {
    s.as_ref()
        .lines()
        .map(|line| {
            if !line.is_empty() {
                format!("{:>spaces$}{}", " ", line, spaces = level * 2)
            } else {
                line.into()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[test]
#[allow(clippy::blacklisted_name)]
fn test_escape() {
    let foo = "Some string with \"quote\"";
    assert_eq!(&escape_quote(foo), "Some string with &quot;quote&quot;");

    let bar = "Some string without quote";
    assert_eq!(&escape_quote(bar), bar);
}

#[test]
#[allow(clippy::blacklisted_name)]
fn test_indent() {
    let foo = "Some string with only one line";
    assert_eq!(indent(foo, 3), "      Some string with only one line");

    let bar = "1. A
  1.1. B
  1.2. C

2. D

3. E

4. F
  4.1 G
    4.1.1 H
  4.2 I";
    assert_eq!(
        indent(bar, 1),
        "  1. A
    1.1. B
    1.2. C

  2. D

  3. E

  4. F
    4.1 G
      4.1.1 H
    4.2 I"
    );
}
