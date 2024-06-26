// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with
// this file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::borrow::Cow;

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

/// conditionnaly encode HTML
pub fn encode_html(s: &str, encode: bool) -> Cow<'_, str> {
    if encode {
        html_escape::encode_text(s)
    } else {
        Cow::Borrowed(s)
    }
}

#[test]
#[allow(clippy::disallowed_names)]
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
