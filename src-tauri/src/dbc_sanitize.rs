//! Pre-processor that makes non-spec-compliant DBC files parseable by dbc-rs.
//!
//! ## Encoding tolerance
//!
//! [`decode_tolerant`] reads raw bytes and converts them to a UTF-8 `String`.
//! Valid UTF-8 sequences are kept as-is.  Any byte that is not part of a valid
//! UTF-8 sequence is interpreted as **ISO 8859-1 / Latin-1**: its Unicode code
//! point equals its byte value (U+0080..U+00FF), so e.g. the Windows-1252
//! degree symbol 0xB0 becomes `°` rather than `?`.  A warning is returned when
//! any such substitution occurs.
//!
//! ## Comment truncation
//!
//! dbc-rs enforces two limits that cause hard parse failures on real-world files:
//!
//! * `MAX_NAME_SIZE` (default 32, raised to 1024 via `.cargo/config.toml`) —
//!   applies to VERSION strings, message/signal/node names, signal units, and VAL_ labels.
//! * `MAX_COMMENT_SIZE = 256` (hardcoded, not env-overridable) —
//!   applies to every CM_ quoted string.
//!
//! This module reads the raw file content, truncates any CM_ quoted strings that
//! exceed 255 bytes, and returns both the sanitized text and a list of human-readable
//! warnings for every truncation it made.
//!
//! ## Multi-line comment support
//!
//! Real-world DBC files sometimes contain CM_ comment strings whose opening `"`
//! is not closed on the same line.  The content continues across subsequent lines
//! (newlines are part of the comment text) until the next unescaped `"` is found.
//!
//! If no closing `"` is found within **31 lines** of the opening quote, the file
//! is considered malformed and [`sanitize`] returns an `Err`.

/// Maximum length dbc-rs accepts for CM_ comment bodies (hardcoded at 256 incl. NUL).
const MAX_COMMENT_BODY: usize = 255;

/// Maximum number of extra lines a CM_ multi-line comment may span.
const MAX_COMMENT_LINES: usize = 31;

// ─── Encoding ────────────────────────────────────────────────────────────────

/// Decode raw file bytes to a UTF-8 `String` tolerantly.
///
/// * Valid UTF-8 sequences are kept unchanged.
/// * Any byte that is **not** part of a valid UTF-8 sequence is treated as a
///   Latin-1 / ISO 8859-1 code unit and converted to its matching Unicode
///   scalar (U+0080–U+00FF).  This is lossless for Latin-1 files and "best
///   effort" for Windows-1252 files.
///
/// Returns `(string, had_non_utf8)`.
pub fn decode_tolerant(bytes: &[u8]) -> (String, bool) {
    // Fast path: the file is already valid UTF-8 (the common case).
    if let Ok(s) = std::str::from_utf8(bytes) {
        return (s.to_owned(), false);
    }

    // Slow path: walk byte-by-byte, substituting invalid sequences.
    let mut out = String::with_capacity(bytes.len() + 32);
    let mut i   = 0;
    let mut had_non_utf8 = false;

    while i < bytes.len() {
        let expected_len = utf8_char_len(bytes[i]);
        let slice        = &bytes[i..bytes.len().min(i + expected_len)];

        if slice.len() == expected_len {
            if let Ok(s) = std::str::from_utf8(slice) {
                out.push_str(s);
                i += expected_len;
                continue;
            }
        }
        // Invalid or truncated sequence: treat the leading byte as Latin-1.
        // `bytes[i] as char` is always a valid Unicode scalar (U+0000..U+00FF).
        out.push(bytes[i] as char);
        had_non_utf8 = true;
        i += 1;
    }

    (out, had_non_utf8)
}

/// Result of the sanitizer: cleaned content + warning messages.
#[derive(Debug)]
pub struct SanitizeResult {
    pub content:  String,
    pub warnings: Vec<String>,
}

/// Sanitize `raw` DBC text so dbc-rs can parse it.
///
/// * Truncates CM_ comment bodies longer than 255 bytes.
/// * Tolerates multi-line CM_ quoted strings (newlines become part of the
///   comment text).  If a closing `"` is not found within [`MAX_COMMENT_LINES`]
///   lines, returns `Err` describing the malformed location.
/// * Correctly handles multi-byte UTF-8 characters in non-CM_ content.
///
/// Returns the sanitized text and any warnings collected during the process,
/// or an `Err` if the file is structurally malformed.
pub fn sanitize(raw: &str) -> Result<SanitizeResult, String> {
    let mut warnings: Vec<String> = Vec::new();
    let mut out = String::with_capacity(raw.len());

    let bytes = raw.as_bytes();
    let len   = bytes.len();
    let mut i = 0;

    while i < len {
        // Fast-forward to the next CM_ keyword.
        // Everything before it is copied verbatim.
        if i + 3 <= len && &bytes[i..i + 3] == b"CM_" {
            // Copy "CM_" itself.
            out.push_str("CM_");
            i += 3;

            // Copy everything up to (not including) the opening quote.
            let pre_quote_start = i;
            while i < len && bytes[i] != b'"' {
                i += 1;
            }
            // pre_quote_start..i is all ASCII (whitespace + keyword chars), safe as UTF-8.
            out.push_str(&raw[pre_quote_start..i]);

            if i >= len {
                break;
            }
            // Consume and emit the opening quote.
            out.push('"');
            i += 1;

            // ── Collect the comment body ─────────────────────────────────
            // dbc-rs's quoted-string parser (non-c_identifier mode, used for
            // CM_ strings) natively handles literal newlines and tracks line
            // numbers.  What it rejects are:
            //   • \t  (tab) — very common in real-world DBC files as a column
            //         separator; we replace each one with a space.
            //   • \r  (carriage return) — present in Windows-style files;
            //         we drop it (the matching \n is kept).
            //   • \   (backslash) — not valid in DBC quoted strings; replaced
            //         with a space.
            //
            // Newlines (\n) are kept as-is; dbc-rs stores and re-emits them
            // transparently, so multi-line comments survive the round-trip.
            // We count them only to detect a missing closing quote.
            let body_start          = out.len();
            let mut body_byte_len: usize = 0;
            let mut truncated       = false;
            let mut newline_count: usize = 0;
            let mut had_tab_or_cr   = false;
            let mut found_closing   = false;

            // Record approximate file position for error messages.
            let open_quote_pos = i.saturating_sub(1);

            while i < len {
                if bytes[i] == b'"' {
                    // Unescaped closing quote — done.
                    i += 1;
                    found_closing = true;
                    break;
                }

                // Carriage return: drop it (the \n that follows will be kept).
                if bytes[i] == b'\r' {
                    had_tab_or_cr = true;
                    i += 1;
                    continue;
                }

                // Tab: replace with four spaces (preserves rough visual alignment).
                // Backslash: replace with a single space (not valid in DBC strings).
                if bytes[i] == b'\t' || bytes[i] == b'\\' {
                    had_tab_or_cr = true;
                    let replacement: &[u8] = if bytes[i] == b'\t' { b"    " } else { b" " };
                    let rep_len = replacement.len();
                    if !truncated && body_byte_len + rep_len <= MAX_COMMENT_BODY {
                        // SAFETY: replacement is pure ASCII, always valid UTF-8.
                        out.push_str(std::str::from_utf8(replacement).unwrap());
                        body_byte_len += rep_len;
                    } else {
                        truncated = true;
                    }
                    i += 1;
                    continue;
                }

                if bytes[i] == b'\n' {
                    newline_count += 1;
                    if newline_count > MAX_COMMENT_LINES {
                        return Err(format!(
                            "CM_ comment opening at byte offset {} spans more than {} lines \
                             without a closing quote — the DBC file appears malformed.",
                            open_quote_pos, MAX_COMMENT_LINES
                        ));
                    }
                    // Keep the newline — dbc-rs handles it natively.
                    if !truncated && body_byte_len + 1 <= MAX_COMMENT_BODY {
                        out.push('\n');
                        body_byte_len += 1;
                    } else {
                        truncated = true;
                    }
                    i += 1;
                    continue;
                }

                // Regular UTF-8 character: determine its byte width.
                let ch_len   = utf8_char_len(bytes[i]);
                let ch_bytes = &bytes[i..i + ch_len.min(len - i)];
                if !truncated && body_byte_len + ch_len <= MAX_COMMENT_BODY {
                    if let Ok(s) = std::str::from_utf8(ch_bytes) {
                        out.push_str(s);
                    }
                    body_byte_len += ch_len;
                } else {
                    truncated = true;
                }
                i += ch_len;
            }

            if !found_closing {
                return Err(format!(
                    "CM_ comment opening at byte offset {} reached end of file without a \
                     closing quote — the DBC file appears malformed.",
                    open_quote_pos
                ));
            }

            if had_tab_or_cr {
                warnings.push(format!(
                    "CM_ comment contained tab or carriage-return characters that are not \
                     valid in DBC quoted strings; they were replaced with spaces."
                ));
            }

            if truncated {
                let body_preview: String = out[body_start..].chars().take(40).collect();
                warnings.push(format!(
                    "CM_ comment truncated to {} bytes (was longer): \"{}…\"",
                    MAX_COMMENT_BODY, body_preview
                ));
            }

            out.push('"'); // closing quote
            // Continue — the rest of the CM_ line (semicolon, newline) will be
            // picked up by the main loop on the next iteration.
        } else {
            // Not inside a CM_ directive — copy the character verbatim,
            // correctly handling multi-byte UTF-8 sequences.
            let ch_len   = utf8_char_len(bytes[i]);
            let ch_bytes = &bytes[i..i + ch_len.min(len - i)];
            if let Ok(s) = std::str::from_utf8(ch_bytes) {
                out.push_str(s);
            } else {
                // Shouldn't happen after decode_tolerant(), but fall back safely.
                out.push(bytes[i] as char);
            }
            i += ch_len;
        }
    }

    Ok(SanitizeResult { content: out, warnings })
}

/// Return the byte width of the UTF-8 character starting at `b`.
#[inline]
fn utf8_char_len(b: u8) -> usize {
    if b < 0x80        { 1 }
    else if b < 0xE0   { 2 }
    else if b < 0xF0   { 3 }
    else               { 4 }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_comment_unchanged() {
        let dbc = r#"CM_ BO_ 256 "Short comment";"#;
        let r   = sanitize(dbc).expect("should not fail");
        assert!(r.warnings.is_empty());
        assert!(r.content.contains("Short comment"));
    }

    #[test]
    fn long_comment_truncated() {
        let long: String = "A".repeat(500);
        let dbc = format!(r#"CM_ BO_ 256 "{long}";"#);
        let r   = sanitize(&dbc).expect("should not fail");
        assert_eq!(r.warnings.len(), 1, "expected one truncation warning");
        let body_start = r.content.find('"').unwrap() + 1;
        let body_end   = r.content.rfind('"').unwrap();
        let body = &r.content[body_start..body_end];
        assert!(body.len() <= MAX_COMMENT_BODY, "body len={}", body.len());
    }

    #[test]
    fn non_cm_lines_untouched() {
        let dbc = "VERSION \"1.0\"\nBO_ 256 Msg : 8 ECM\n";
        let r   = sanitize(dbc).expect("should not fail");
        assert!(r.warnings.is_empty());
        assert_eq!(r.content, dbc);
    }

    // ── Multi-line comment tests ─────────────────────────────────────────────

    #[test]
    fn multiline_comment_newlines_preserved() {
        // dbc-rs natively allows \n in non-c_identifier quoted strings.
        // The sanitizer must pass newlines through unchanged.
        let dbc = "CM_ SG_ 1 2 \"line one\nline two\";\n";
        let r   = sanitize(dbc).expect("multi-line comment should parse");
        assert!(r.warnings.is_empty(), "no warnings expected for plain newlines");
        assert!(r.content.contains("line one\nline two"),
                "newlines must be preserved verbatim");
    }

    #[test]
    fn tab_in_comment_replaced_with_space() {
        // dbc-rs rejects \t in quoted strings — sanitizer must replace it.
        let dbc = "CM_ BU_ Node \"col1\tcol2\tcol3\";\n";
        let r   = sanitize(dbc).expect("tab in comment should be sanitized");
        assert_eq!(r.warnings.len(), 1, "expected one tab-replacement warning");
        assert!(r.warnings[0].contains("tab"),
                "warning should mention tab characters");
        assert!(r.content.contains("col1    col2    col3"),
                "tabs should become four spaces");
        assert!(!r.content.contains('\t'), "no stray tab in output");
    }

    #[test]
    fn multiline_with_tabs_real_world_snippet() {
        // Representative of the failing file: multi-line comment containing tabs.
        let dbc = "CM_ BU_ FZD \"Integrationsmodul\nRLS\t\t\tPeter Schiele\nMDS\t\t\tRobert Siwy\";\n";
        let r   = sanitize(dbc).expect("real-world snippet should parse");
        // Newlines survive; tabs become spaces; one tab warning.
        assert!(r.content.contains("Integrationsmodul\nRLS"), "newline preserved");
        assert!(!r.content.contains('\t'), "no tabs in output");
        // Each tab becomes four spaces.
        assert!(r.content.contains("RLS    "), "tab expanded to four spaces");
        assert_eq!(r.warnings.len(), 1, "one warning for tab replacement");
    }

    #[test]
    fn crlf_in_comment_cr_dropped() {
        // Windows \r\n: \r is dropped, \n is kept.
        let dbc = "CM_ SG_ 1 2 \"part A\r\npart B\";\n";
        let r   = sanitize(dbc).expect("CRLF multi-line should parse");
        assert!(r.content.contains("part A\npart B"),
                "\\r dropped, \\n kept");
        assert!(!r.content.contains('\r'), "no stray CR in output");
    }

    #[test]
    fn multiline_comment_many_lines_ok() {
        // Exactly MAX_COMMENT_LINES newlines — should still succeed.
        let body: String = (0..MAX_COMMENT_LINES).map(|n| format!("L{n}")).collect::<Vec<_>>().join("\n");
        let dbc = format!("CM_ SG_ 1 2 \"{body}\";\n");
        let r   = sanitize(&dbc).expect("should be within line limit");
        assert!(r.content.contains("L0"));
    }

    #[test]
    fn multiline_comment_too_many_lines_is_error() {
        // MAX_COMMENT_LINES + 1 newlines → hard error.
        let body: String = "\n".repeat(MAX_COMMENT_LINES + 1);
        let dbc = format!("CM_ SG_ 1 2 \"{body}\";\n");
        let result = sanitize(&dbc);
        assert!(result.is_err(), "should fail: too many lines");
        let msg = result.unwrap_err();
        assert!(msg.contains("more than"), "error should mention line limit: {msg}");
    }

    #[test]
    fn comment_eof_without_closing_quote_is_error() {
        let dbc = "CM_ SG_ 1 2 \"no closing quote";
        let result = sanitize(dbc);
        assert!(result.is_err(), "should fail: no closing quote");
        let msg = result.unwrap_err();
        assert!(msg.contains("end of file"), "error should mention EOF: {msg}");
    }

    #[test]
    fn non_ascii_outside_cm_preserved() {
        // Degree symbol (UTF-8 0xC2 0xB0) outside a CM_ block must not be corrupted.
        let dbc = "VERSION \"test\"\nBO_ 1 Temp°C : 8 ECM\n";
        let r   = sanitize(dbc).expect("should not fail");
        assert!(r.content.contains("Temp°C"), "UTF-8 multi-byte char should be preserved");
    }

    // ── Encoding tests ───────────────────────────────────────────────────────

    #[test]
    fn pure_utf8_unchanged() {
        let bytes = "VERSION \"1.0\"\n".as_bytes();
        let (s, flag) = decode_tolerant(bytes);
        assert!(!flag);
        assert_eq!(s, "VERSION \"1.0\"\n");
    }

    #[test]
    fn latin1_degree_symbol() {
        // 0xB0 is '°' in Latin-1 / ISO 8859-1
        let bytes: &[u8] = b"CM_ SG_ 1 Temp \"\xb0C\";";
        let (s, flag) = decode_tolerant(bytes);
        assert!(flag, "should flag non-UTF-8");
        assert!(s.contains('°'), "degree symbol should be preserved");
    }

    #[test]
    fn mixed_utf8_and_latin1() {
        // Valid UTF-8 prefix, then a Latin-1 byte, then more ASCII
        let mut bytes = b"hello ".to_vec();
        bytes.push(0xE9); // 'é' in Latin-1
        bytes.extend_from_slice(b" world");
        let (s, flag) = decode_tolerant(&bytes);
        assert!(flag);
        assert!(s.starts_with("hello "));
        assert!(s.ends_with(" world"));
    }
}
