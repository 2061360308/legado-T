//! XPath selector backed by `sxd-xpath`.
//!
//! HTML is first parsed by `scraper`/html5ever (the same tree used by CSS),
//! then serialised into well-formed XML for `sxd_document`. This mirrors
//! Jsoup + JsoupXpath on Android and avoids rejecting real-world HTML that is
//! not well-formed XML.

use std::collections::HashSet;

use scraper::{ElementRef, Html};
use sxd_xpath::{Context, Factory, Value as XPathValue};

use crate::content::{Content, Value};
use crate::selector::css::RuleOutput;

const LEGADO_XPATH_EXTRACTORS: &[&str] = &[
    "text",
    "ownText",
    "html",
    "outerHtml",
    "allText",
    "textNodes",
    "node",
];

const XPATH_AXIS_NAMES: &[&str] = &[
    "ancestor",
    "ancestor-or-self",
    "attribute",
    "child",
    "descendant",
    "descendant-or-self",
    "following",
    "following-sibling",
    "namespace",
    "parent",
    "preceding",
    "preceding-sibling",
    "self",
];

pub(crate) fn silence_errors() {}

pub fn is_valid(xpath: &str) -> bool {
    let (path, _) = split_xpath_axis(xpath);
    let path = normalize_xpath_quotes(&path);
    Factory::new()
        .build(&path)
        .map(|expr| expr.is_some())
        .unwrap_or(false)
}

pub fn eval(content: &Content, xpath: &str, output: RuleOutput) -> Result<Value, String> {
    let (path, extractor) = split_xpath_axis(xpath);
    let path = normalize_xpath_quotes(&path);
    let xml = content.with_html(html_to_xml);
    let package =
        sxd_document::parser::parse(&xml).map_err(|e| format!("HTML 转 XML 失败：{e}"))?;
    let document = package.as_document();
    let context = Context::new();
    let expression = Factory::new()
        .build(&path)
        .map_err(|e| format!("XPath 编译失败：{e}"))?
        .ok_or_else(|| format!("XPath 为空：{path}"))?;
    let value = expression
        .evaluate(&context, document.root())
        .map_err(|e| format!("XPath 求值失败：{e}"))?;

    let items: Vec<String> = match value {
        XPathValue::Nodeset(nodes) => {
            let nodes = nodes.document_order();
            match extractor {
                Some(extractor) => nodes
                    .iter()
                    .filter_map(|node| node_extract(node, extractor))
                    .collect(),
                None if output == RuleOutput::Elements => nodes
                    .iter()
                    .map(|node| match node {
                        sxd_xpath::nodeset::Node::Element(_) => node_to_html(node),
                        _ => node.string_value(),
                    })
                    .collect(),
                None => nodes.iter().map(|node| node.string_value()).collect(),
            }
        }
        XPathValue::String(text) => vec![text],
        XPathValue::Number(number) => vec![number.to_string()],
        XPathValue::Boolean(boolean) => vec![boolean.to_string()],
    };

    if items.is_empty() {
        if let Some(alt) = rewrite_absolute_index(&path) {
            let alt_rule = match extractor {
                Some(extractor) => format!("{alt}@{extractor}"),
                None => alt,
            };
            if let Ok(value) = eval(content, &alt_rule, output)
                && !value.is_empty()
            {
                return Ok(value);
            }
        }
        if let Some(at) = path.rfind("/@")
            && let Ok(Some(parents)) = Factory::new().build(&path[..at])
            && let Ok(XPathValue::Nodeset(nodes)) = parents.evaluate(&context, document.root())
        {
            let count = nodes.size();
            if count > 0 {
                let empty = vec![String::new(); count];
                return Ok(match output {
                    RuleOutput::Elements => Value::Elements(empty),
                    RuleOutput::Text => match empty.len() {
                        1 => Value::Text(String::new()),
                        _ => Value::List(empty),
                    },
                });
            }
        }
        return Ok(match output {
            RuleOutput::Text => Value::Text(String::new()),
            RuleOutput::Elements => Value::None,
        });
    }

    Ok(match output {
        RuleOutput::Elements => Value::Elements(items),
        RuleOutput::Text => match items.len() {
            1 => Value::Text(items.into_iter().next().unwrap_or_default()),
            _ => Value::List(items),
        },
    })
}

fn html_to_xml(document: &Html) -> String {
    const VOID: &[&str] = &[
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param",
        "source", "track", "wbr",
    ];

    fn escape_text(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }

    fn escape_attr(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    fn valid_name(name: &str) -> bool {
        let mut chars = name.chars();
        match chars.next() {
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {}
            _ => return false,
        }
        chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':'))
    }

    fn write_element(element: ElementRef<'_>, out: &mut String) {
        let name = element.value().name();
        out.push('<');
        out.push_str(name);
        let mut seen = HashSet::new();
        for (key, value) in element.value().attrs() {
            if !valid_name(key) || !seen.insert(key) {
                continue;
            }
            out.push(' ');
            out.push_str(key);
            out.push_str("=\"");
            out.push_str(&escape_attr(value));
            out.push('"');
        }
        if VOID.contains(&name) {
            out.push_str(" />");
            return;
        }
        out.push('>');
        for child in element.children() {
            match ElementRef::wrap(child) {
                Some(child) => write_element(child, out),
                None => {
                    if let Some(text) = child.value().as_text() {
                        out.push_str(&escape_text(text));
                    }
                }
            }
        }
        out.push_str("</");
        out.push_str(name);
        out.push('>');
    }

    let mut out = String::with_capacity(document.html().len() + 256);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    write_element(document.root_element(), &mut out);
    out
}

fn normalize_xpath_quotes(xpath: &str) -> String {
    let mut out = String::with_capacity(xpath.len());
    let mut chars = xpath.chars();
    while let Some(ch) = chars.next() {
        if ch != '"' {
            out.push(ch);
            continue;
        }
        let mut literal = String::new();
        let mut closed = false;
        for next in chars.by_ref() {
            if next == '"' {
                closed = true;
                break;
            }
            literal.push(next);
        }
        if closed && !literal.contains('\'') {
            out.push('\'');
            out.push_str(&literal);
            out.push('\'');
        } else {
            out.push('"');
            out.push_str(&literal);
            if closed {
                out.push('"');
            }
        }
    }
    out
}

fn split_xpath_axis(xpath: &str) -> (String, Option<&str>) {
    let trimmed = xpath.trim();
    let len = trimmed.len();
    let bytes = trimmed.as_bytes();
    let mut start = len;
    for &byte in bytes.iter().rev() {
        if byte.is_ascii_alphabetic() {
            start -= 1;
        } else {
            break;
        }
    }
    if start < 2 {
        return (trimmed.to_string(), None);
    }
    let left = &trimmed[..start];
    if !left.ends_with("::") {
        return (trimmed.to_string(), None);
    }
    let extractor = &trimmed[start..];
    if !LEGADO_XPATH_EXTRACTORS.contains(&extractor) {
        return (trimmed.to_string(), None);
    }
    let before = left[..left.len() - 2].trim_end();
    let last_segment = before.rsplit('/').next().unwrap_or(before);
    if XPATH_AXIS_NAMES.contains(&last_segment) {
        return (trimmed.to_string(), None);
    }
    (before.trim().to_string(), Some(extractor))
}

fn node_to_html_inner(node: sxd_xpath::nodeset::Node<'_>, text: &mut String) {
    use sxd_xpath::nodeset::Node;
    match node {
        Node::Element(element) => {
            let name = element.name().local_part();
            text.push('<');
            text.push_str(name);
            for attr in element.attributes() {
                let attr_name = attr.name();
                text.push(' ');
                text.push_str(attr_name.local_part());
                text.push_str("=\"");
                text.push_str(&attr.value().replace('"', "&quot;"));
                text.push('"');
            }
            text.push('>');
            for child in element.children() {
                use sxd_document::dom::ChildOfElement;
                match child {
                    ChildOfElement::Element(child) => {
                        node_to_html_inner(Node::Element(child), text);
                    }
                    ChildOfElement::Text(child) => {
                        text.push_str(
                            &child
                                .text()
                                .replace('&', "&amp;")
                                .replace('<', "&lt;")
                                .replace('>', "&gt;"),
                        );
                    }
                    ChildOfElement::Comment(child) => {
                        text.push_str("<!--");
                        text.push_str(child.text());
                        text.push_str("-->");
                    }
                    ChildOfElement::ProcessingInstruction(child) => {
                        if let Some(value) = child.value() {
                            text.push_str(&format!("<?{} {}?>", child.target(), value));
                        }
                    }
                }
            }
            text.push_str("</");
            text.push_str(name);
            text.push('>');
        }
        Node::Text(child) => {
            text.push_str(
                &child
                    .text()
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;"),
            );
        }
        Node::Comment(child) => {
            text.push_str("<!--");
            text.push_str(child.text());
            text.push_str("-->");
        }
        Node::ProcessingInstruction(child) => {
            if let Some(value) = child.value() {
                text.push_str(&format!("<?{} {}?>", child.target(), value));
            }
        }
        _ => {}
    }
}

fn node_to_html(node: &sxd_xpath::nodeset::Node<'_>) -> String {
    let mut out = String::new();
    node_to_html_inner(*node, &mut out);
    out
}

fn node_inner_html(node: &sxd_xpath::nodeset::Node<'_>) -> String {
    let mut out = String::new();
    for child in node.children() {
        node_to_html_inner(child, &mut out);
    }
    out
}

fn node_extract(node: &sxd_xpath::nodeset::Node<'_>, extractor: &str) -> Option<String> {
    use sxd_xpath::nodeset::Node;
    match extractor {
        "text" | "textNodes" | "allText" | "node" => {
            let value = node.string_value();
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        }
        "ownText" => {
            let mut own = String::new();
            for child in node.children() {
                if let Node::Text(text) = child {
                    own.push_str(text.text().trim());
                    own.push(' ');
                }
            }
            let value = own.trim().to_string();
            if value.is_empty() { None } else { Some(value) }
        }
        "html" => {
            let value = node_inner_html(node);
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        }
        "outerHtml" => {
            let value = node_to_html(node);
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        }
        _ => None,
    }
}
fn rewrite_absolute_index(path: &str) -> Option<String> {
    let rest = path.strip_prefix("//")?;
    let segment_end = rest.find('/').unwrap_or(rest.len());
    let head = &rest[..segment_end];
    let open = head.find('[')?;
    let close = head.find(']')?;
    if open == 0 || close <= open {
        return None;
    }
    let tag = &head[..open];
    if !tag
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return None;
    }
    Some(format!(
        "(//{tag}){}{}",
        &head[open..],
        &rest[segment_end..]
    ))
}
