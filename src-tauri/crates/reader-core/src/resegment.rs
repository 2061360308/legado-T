use fancy_regex::Regex;

const SENTENCE_END: &str = "？。！?!~";
const SENTENCE_END_WITH_DOT: &str = ".？。！?!~";
const SAY: &str = "问说喊唱叫骂道着答";

fn replace_all(pattern: &str, text: &str, replacement: &str) -> String {
    Regex::new(pattern)
        .map(|regex| regex.replace_all(text, replacement).into_owned())
        .unwrap_or_else(|_| text.to_string())
}

fn normalize_quotes(text: &str) -> String {
    let mut text = text.replace("&quot;", "“");
    text = replace_all(r#"[“”"]+\s*[“”"]+"#, &text, "”\n“");
    text = replace_all(r#"[:：]['"‘“”]+"#, &text, "：“");
    text
}

fn clean_line(line: &str) -> String {
    line.chars()
        .filter(|ch| *ch != '\u{3000}' && !ch.is_whitespace())
        .collect()
}

fn merge_broken_lines(text: &str, chapter_title: &str) -> String {
    let mut out = String::new();
    for (index, raw) in text.split('\n').enumerate() {
        let line = clean_line(raw);
        if line.is_empty() {
            continue;
        }
        if index == 0 && line == clean_line(chapter_title) {
            continue;
        }
        if !out.is_empty() {
            let chars: Vec<char> = out.chars().collect();
            let last = chars.last().copied().unwrap_or_default();
            let previous = chars
                .get(chars.len().saturating_sub(2))
                .copied()
                .unwrap_or_default();
            let ends_sentence = SENTENCE_END.contains(last);
            let ends_quote = "”\"".contains(last) && SENTENCE_END.contains(previous);
            if ends_sentence || ends_quote {
                out.push('\n');
            }
        }
        out.push_str(&line);
    }
    out
}

fn split_line(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut in_quote = false;
    let mut quoted_sentences = 0;
    let mut length = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        out.push(ch);
        length += 1;
        if "\"“”".contains(ch) {
            in_quote = !in_quote;
            if !in_quote {
                quoted_sentences = 0;
            }
            i += 1;
            continue;
        }
        if SENTENCE_END_WITH_DOT.contains(ch) {
            if in_quote {
                quoted_sentences += 1;
            }
            let next = chars.get(i + 1).copied();
            let at_quote_end = matches!(next, Some('”') | Some('"'));
            let quote_after = next == Some('”') || next == Some('"');
            if !in_quote && !at_quote_end {
                if let Some(next) = next {
                    if !SENTENCE_END_WITH_DOT.contains(next) {
                        out.push('\n');
                        length = 0;
                    }
                }
            } else if at_quote_end {
                out.push('”');
                i += 1;
                if let Some(next) = chars.get(i + 1).copied() {
                    if !matches!(next, '“' | '"') {
                        out.push('\n');
                        length = 0;
                    }
                }
            } else if in_quote && quoted_sentences >= 5 && quote_after {
                quoted_sentences = 0;
            }
        }
        if length >= 120 {
            out.push('\n');
            length = 0;
        }
        i += 1;
    }
    out
}

fn normalize_line_breaks(text: &str) -> String {
    let mut text = text.to_string();
    text = replace_all(r#"\s*[“”"]+\s*[“”"][\s“”"]*"#, &text, "”\n“");
    text = replace_all(r#"[:：][“”"\s]+"#, &text, "：“");
    text = replace_all(
        r#"\n[“"”]([^\n“"”]+)([,:，：][“"”])([^\n“"”]+)"#,
        &text,
        "\n$1：“$3",
    );
    text = replace_all(r#"\n\s*"#, &text, "\n");
    text.trim_start_matches(['\r', '\n', ' ']).to_string()
}

pub fn re_segment(text: &str, chapter_title: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }
    let merged = merge_broken_lines(text, chapter_title);
    let mut normalized = normalize_quotes(&merged);
    normalized = replace_all(r#"[“”"]+(？。！?!~)[“”"]+"#, &normalized, "”$1\n“");
    normalized = replace_all(r#"[“”"]+(？。！?!~)([^“”"])"#, &normalized, "”$1\n$2");
    normalized = replace_all(&format!(r#"([{SAY}])\."#), &normalized, "$1。\n");
    let rebuilt = normalized
        .split('\n')
        .map(split_line)
        .collect::<Vec<_>>()
        .join("\n");
    normalize_line_breaks(&rebuilt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_dialogue_together() {
        let text = "他说：“今天很好。”然后离开了。";
        let result = re_segment(text, "第一章");
        assert!(result.contains("他说：“今天很好。”"));
        assert!(result.contains('\n'));
    }

    #[test]
    fn strips_embedded_spaces_and_title() {
        let result = re_segment("第一章\n这 是 正 文", "第一章");
        assert_eq!(result, "这是正文");
    }
}
