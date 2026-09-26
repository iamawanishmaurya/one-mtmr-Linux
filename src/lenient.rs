/// Lenient JSON preprocessing for real-world MTMR presets.
///
/// Community presets routinely contain: UTF-8 BOM, text preamble before the
/// array, // and /* */ comments, trailing commas, and raw control characters
/// inside strings (unescaped \r bytes in AppleScript). MTMR tolerates all of
/// that; serde_json does not. sanitize() rewrites the text into strict JSON.

pub fn sanitize(raw: &str) -> String {
    let raw = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    // Skip any preamble before the top-level array/object.
    let start = raw.find(|c| c == '[' || c == '{').unwrap_or(0);
    let bytes: Vec<char> = raw[start..].chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0usize;
    let mut in_str = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_str {
            match c {
                '\\' => {
                    // keep escape pairs; drop raw control chars after backslash? real \r bytes appear alone
                    if i + 1 < bytes.len() {
                        out.push('\\');
                        out.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                '"' => {
                    in_str = false;
                    out.push('"');
                    i += 1;
                }
                c if (c as u32) < 0x20 => {
                    // escape raw control chars inside strings
                    out.push_str(&format!("\\u{:04x}", c as u32));
                    i += 1;
                }
                c => {
                    out.push(c);
                    i += 1;
                }
            }
            continue;
        }
        match c {
            '"' => {
                in_str = true;
                out.push('"');
                i += 1;
            }
            '/' if i + 1 < bytes.len() && bytes[i + 1] == '/' => {
                while i < bytes.len() && bytes[i] != '\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < bytes.len() && bytes[i + 1] == '*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == '*' && bytes[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            ',' => {
                // drop trailing comma: peek past whitespace/comments for } or ]
                let mut j = i + 1;
                loop {
                    while j < bytes.len() && bytes[j].is_whitespace() {
                        j += 1;
                    }
                    if j + 1 < bytes.len() && bytes[j] == '/' && bytes[j + 1] == '/' {
                        while j < bytes.len() && bytes[j] != '\n' {
                            j += 1;
                        }
                        continue;
                    }
                    if j + 1 < bytes.len() && bytes[j] == '/' && bytes[j + 1] == '*' {
                        j += 2;
                        while j + 1 < bytes.len() && !(bytes[j] == '*' && bytes[j + 1] == '/') {
                            j += 1;
                        }
                        j += 2;
                        continue;
                    }
                    break;
                }
                if j < bytes.len() && (bytes[j] == '}' || bytes[j] == ']') {
                    // skip the comma
                } else {
                    out.push(',');
                }
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> serde_json::Value {
        serde_json::from_str(&sanitize(s)).unwrap()
    }

    #[test]
    fn keeps_valid_json_untouched() {
        let src = r#"[
            {"type": "escape", "title": "esc \"quoted\""},
            {"type": "staticButton", "note": "a/b not a comment", "n": 3}
        ]"#;
        let v = parse(src);
        assert_eq!(v.as_array().unwrap().len(), 2);
    }

    #[test]
    fn strips_line_and_block_comments() {
        let v = parse(concat!(
            "// header comment\n",
            "[\n",
            "  // inline\n",
            "  {\"type\": \"escape\"},\n",
            "  /* blocked\n",
            "     {\"type\": \"nightShift\"},\n",
            "  */\n",
            "  {\"type\": \"play\"}\n",
            "]"
        ));
        assert_eq!(v.as_array().unwrap().len(), 2);
    }

    #[test]
    fn strips_trailing_commas() {
        let v = parse(r#"[{"a": 1,}, {"b": [1, 2,],}]"#);
        assert_eq!(v.as_array().unwrap().len(), 2);
        assert_eq!(v[1]["b"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn escapes_control_chars_in_strings() {
        // raw \r bytes inside AppleScript strings, as community presets ship them
        let src = "[{\"type\": \"appleScriptTitledButton\", \"source\": {\"inline\": \"tell app \\\"X\\\"\rdo it\rend tell\"}}]";
        let v = parse(src);
        let s = v[0]["source"]["inline"].as_str().unwrap();
        assert!(s.contains("do it"));
    }

    #[test]
    fn strips_bom_and_preamble() {
        let src = "\u{feff}readme text here\n\n[{\"type\": \"escape\"}]";
        let v = parse(src);
        assert_eq!(v[0]["type"], "escape");
    }

    #[test]
    fn comment_between_comma_and_brace() {
        let v = parse("[{\"a\": 1, /* c */ }]");
        assert_eq!(v[0]["a"], 1);
    }
}
