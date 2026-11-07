//! The one place a caller-supplied id is turned into CSS or JavaScript text.
//!
//! An id is arbitrary text (`a.b`, `x:y`, `it's`). Formatted raw into a CSS identifier, a CSS
//! attribute selector or a single-quoted script string it ends the token early: the anchor name of
//! `a.b` stops at the dot, an apostrophe closes the script's string. Every site goes through one of
//! these, one per context, and `scripts/check-id-interpolation.sh` fails a site that formats an id
//! by hand:
//!
//! - [`css_ident`]: the characters after `--dxa-` in an `anchor-name` / `position-anchor` value;
//! - [`css_string`]: the inside of a double-quoted CSS string;
//! - [`anchor_selector`]: the selector that finds the element whose `style` declares an id's anchor,
//!   sent to the page script as data so it never concatenates an id into a selector;
//! - [`js_string`]: a complete JavaScript string literal.

/// `id` as the tail of a CSS identifier (the semantics of `CSS.escape`, for a name that already
/// starts with `--dxa-`, so no leading-digit rule applies).
#[cfg(any(feature = "web", test))]
pub(crate) fn css_ident(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for c in id.chars() {
        match c {
            '\0' => out.push('\u{FFFD}'),
            '\u{1}'..='\u{1f}' | '\u{7f}' => out.push_str(&format!("\\{:x} ", c as u32)),
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => out.push(c),
            c if c as u32 >= 0x80 => out.push(c),
            c => {
                out.push('\\');
                out.push(c);
            }
        }
    }
    out
}

/// `text` as the inside of a double-quoted CSS string.
#[cfg(any(feature = "web", test))]
pub(crate) fn css_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\0' => out.push('\u{FFFD}'),
            '\u{1}'..='\u{1f}' | '\u{7f}' => out.push_str(&format!("\\{:x} ", c as u32)),
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
}

/// The selector for the element whose `style` attribute declares the anchor `anchor_name_style(id)`
/// writes. Empty for an empty id (no anchor), which the page scripts read as "none".
#[cfg(any(feature = "web", test))]
pub(crate) fn anchor_selector(id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }
    format!(
        "[style*=\"anchor-name: --dxa-{};\"]",
        css_string(&css_ident(id))
    )
}

/// `text` as a complete JavaScript string literal, quotes included.
pub(crate) fn js_string(text: &str) -> String {
    serde_json::to_string(text).expect("a string always serialises")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_ident_escapes_what_ends_an_identifier() {
        assert_eq!(css_ident("plain_id-1"), "plain_id-1");
        assert_eq!(css_ident("a.b"), r"a\.b");
        assert_eq!(css_ident("x:y z"), r"x\:y\ z");
        assert_eq!(css_ident("it's"), r"it\'s");
        assert_eq!(css_ident("a\nb"), "a\\a b");
        assert_eq!(css_ident("é"), "é");
    }

    #[test]
    fn css_string_escapes_quote_and_backslash() {
        assert_eq!(css_string(r#"a"b\c"#), r#"a\"b\\c"#);
    }

    #[test]
    fn the_selector_matches_the_declaration_text() {
        assert_eq!(
            anchor_selector("a.b"),
            r#"[style*="anchor-name: --dxa-a\\.b;"]"#
        );
        assert_eq!(anchor_selector("x"), r#"[style*="anchor-name: --dxa-x;"]"#);
        assert_eq!(anchor_selector(""), "");
    }

    #[test]
    fn js_string_survives_a_quote_and_a_line_break() {
        assert_eq!(js_string("it's"), r#""it's""#);
        assert_eq!(js_string("a\"b\nc"), r#""a\"b\nc""#);
        assert_eq!(js_string("</script>"), r#""</script>""#);
    }
}
