#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitResult {
    pub parts: Vec<String>,
    pub delimiter: Option<String>,
}

/// Split a Legado rule on top-level delimiters.
///
/// Delimiters inside `[]`, `()`, `{}`, quotes and `<js>...</js>` are ignored.
pub fn split_top_level(rule: &str, delimiters: &[&str]) -> SplitResult {
    let Some((_, delimiter)) = find_next_delimiter(rule, delimiters, 0) else {
        return SplitResult {
            parts: vec![rule.trim().to_string()],
            delimiter: None,
        };
    };

    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut search_from = 0usize;
    while let Some((index, _)) = find_next_delimiter(rule, &[delimiter], search_from) {
        parts.push(rule[start..index].trim().to_string());
        start = index + delimiter.len();
        search_from = start;
    }
    parts.push(rule[start..].trim().to_string());

    SplitResult {
        parts,
        delimiter: Some(delimiter.to_string()),
    }
}

pub fn split_combinator(rule: &str) -> SplitResult {
    split_top_level(rule, &["&&", "||", "%%"])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceParts {
    pub body: String,
    pub pattern: Option<String>,
    pub replacement: Option<String>,
    pub first: bool,
}

/// Split `body##pattern##replacement###`.
///
/// `###` means "replace only the first match".
pub fn split_replace(rule: &str) -> ReplaceParts {
    let parts: Vec<&str> = rule.split("##").collect();
    if parts.len() < 2 {
        return ReplaceParts {
            body: rule.trim().to_string(),
            pattern: None,
            replacement: None,
            first: false,
        };
    }
    ReplaceParts {
        body: parts[0].trim().to_string(),
        pattern: Some(parts[1].to_string()),
        replacement: parts.get(2).map(|value| (*value).to_string()),
        first: parts.len() > 3,
    }
}

fn find_next_delimiter<'a>(
    rule: &'a str,
    delimiters: &[&'a str],
    from: usize,
) -> Option<(usize, &'a str)> {
    let mut square_depth = 0i32;
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut in_js = false;

    for (index, ch) in rule.char_indices().filter(|(index, _)| *index >= from) {
        if in_js {
            if rule[index..].starts_with("</js>") {
                in_js = false;
            }
            continue;
        }
        if rule[index..].starts_with("<js>") {
            in_js = true;
            quote = None;
            escaped = false;
            continue;
        }

        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                continue;
            }
            '[' => square_depth += 1,
            ']' => square_depth -= 1,
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            '\\' => {
                escaped = true;
                continue;
            }
            _ => {}
        }

        if square_depth == 0 && paren_depth == 0 && brace_depth == 0 {
            if let Some(delimiter) = delimiters.iter().find(|delimiter| {
                rule[index..].starts_with(**delimiter)
                    && !(**delimiter == "@" && rule[index..].starts_with("@@"))
            }) {
                return Some((index, *delimiter));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ignores_delimiters_inside_attribute_selector() {
        let result = split_top_level(r#"div[a="x&&y"]&&span"#, &["&&", "||", "%%"]);
        assert_eq!(result.delimiter.as_deref(), Some("&&"));
        assert_eq!(result.parts, vec![r#"div[a="x&&y"]"#, "span"]);
    }

    #[test]
    fn split_ignores_delimiters_inside_js() {
        let result = split_top_level(
            r#"<js>if (a || b) { return "x&&y" }</js>||.fallback"#,
            &["&&", "||", "%%"],
        );
        assert_eq!(result.delimiter.as_deref(), Some("||"));
        assert_eq!(
            result.parts,
            vec![r#"<js>if (a || b) { return "x&&y" }</js>"#, ".fallback"]
        );
    }

    #[test]
    fn split_replace_with_first_match() {
        let result = split_replace(r#"class.title##\s+##-###"#);
        assert_eq!(result.body, "class.title");
        assert_eq!(result.pattern.as_deref(), Some(r"\s+"));
        assert_eq!(result.replacement.as_deref(), Some("-"));
        assert!(result.first);
    }
}
