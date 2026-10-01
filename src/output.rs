use crate::lint::Finding;

/// Escapes a string for use inside a JSON string literal. Hand-rolled because
/// the project takes no dependencies; messages can contain card ids from the
/// input file, so quotes, backslashes and control characters all have to be
/// handled.
fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Renders findings as a compiler-style line per finding.
pub fn format_text(path: &str, findings: &[Finding]) -> String {
    if findings.is_empty() {
        return format!("{path}: no issues found\n");
    }
    let mut out = String::new();
    for f in findings {
        out.push_str(&format!("{path}:{}: {}: {}\n", f.line, f.severity, f.message));
    }
    out
}

/// Renders findings as a JSON array, one object per finding. An empty deck
/// result is `[]` so consumers never have to special-case a clean file.
pub fn format_json(path: &str, findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "[]\n".to_string();
    }
    let items: Vec<String> = findings
        .iter()
        .map(|f| {
            format!(
                "  {{\"file\": \"{}\", \"line\": {}, \"severity\": \"{}\", \"message\": \"{}\"}}",
                escape_json(path),
                f.line,
                f.severity,
                escape_json(&f.message)
            )
        })
        .collect();
    format!("[\n{}\n]\n", items.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::Severity;

    fn finding(line: usize, severity: Severity, message: &str) -> Finding {
        Finding {
            line,
            severity,
            message: message.to_string(),
        }
    }

    #[test]
    fn escapes_quotes_backslashes_and_control_characters() {
        assert_eq!(escape_json("a\"b\\c\nd\u{1}"), "a\\\"b\\\\c\\nd\\u0001");
    }

    #[test]
    fn text_output_matches_the_compiler_style() {
        let out = format_text("deck.txt", &[finding(2, Severity::Error, "bad")]);
        assert_eq!(out, "deck.txt:2: error: bad\n");
    }

    #[test]
    fn text_output_reports_a_clean_file() {
        assert_eq!(format_text("deck.txt", &[]), "deck.txt: no issues found\n");
    }

    #[test]
    fn json_output_for_no_findings_is_an_empty_array() {
        assert_eq!(format_json("deck.txt", &[]), "[]\n");
    }

    #[test]
    fn json_output_lists_each_finding() {
        let out = format_json(
            "deck.txt",
            &[
                finding(2, Severity::Error, "ease \"low\""),
                finding(5, Severity::Warning, "long"),
            ],
        );
        let expected = "[\n  {\"file\": \"deck.txt\", \"line\": 2, \"severity\": \"error\", \"message\": \"ease \\\"low\\\"\"},\n  {\"file\": \"deck.txt\", \"line\": 5, \"severity\": \"warning\", \"message\": \"long\"}\n]\n";
        assert_eq!(out, expected);
    }
}
