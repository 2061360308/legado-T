use std::io::{Cursor, Read, Write};

use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use base64::Engine;
use md5::{Digest, Md5};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Clone)]
pub struct BackupFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub encrypted: bool,
    pub decrypted: bool,
}

fn derive_key(password: &str) -> [u8; 16] {
    let digest = Md5::digest(password.as_bytes());
    let mut key = [0u8; 16];
    key.copy_from_slice(&digest[..16]);
    key
}

fn derive_key_kotlin(password: &str) -> [u8; 16] {
    let digest = Md5::digest(password.as_bytes());
    let hex = format!("{digest:x}");
    let mut key = [0u8; 16];
    key.copy_from_slice(&hex.as_bytes()[..16]);
    key
}

fn crypt(data: &[u8], key: [u8; 16], encrypt: bool) -> Option<Vec<u8>> {
    let cipher = Aes128::new_from_slice(&key).ok()?;
    let mut buf = if encrypt {
        let pad = 16 - (data.len() % 16);
        let mut buffer = data.to_vec();
        buffer.extend(std::iter::repeat(pad as u8).take(pad));
        buffer
    } else {
        if data.is_empty() || data.len() % 16 != 0 {
            return None;
        }
        data.to_vec()
    };
    for chunk in buf.chunks_mut(16) {
        if encrypt {
            cipher.encrypt_block(chunk.into());
        } else {
            cipher.decrypt_block(chunk.into());
        }
    }
    if !encrypt {
        let pad = *buf.last()? as usize;
        if pad == 0 || pad > 16 || pad > buf.len() {
            return None;
        }
        buf.truncate(buf.len() - pad);
    }
    Some(buf)
}

fn encrypt_kotlin(data: &[u8], password: &str) -> Option<String> {
    crypt(data, derive_key_kotlin(password), true)
        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes))
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len() / 2);
    let mut index = 0;
    while index < bytes.len() {
        let high = (bytes[index] as char).to_digit(16)?;
        let low = (bytes[index + 1] as char).to_digit(16)?;
        out.push(((high << 4) | low) as u8);
        index += 2;
    }
    Some(out)
}

fn decrypt_any(text: &str, password: &str) -> Option<Vec<u8>> {
    if let Some(bytes) = from_hex(text) {
        if let Some(plain) = crypt(&bytes, derive_key(password), false) {
            return Some(plain);
        }
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(text)
        .ok()?;
    crypt(&bytes, derive_key_kotlin(password), false)
}

pub fn encrypt_text(text: &str, password: &str) -> Option<String> {
    encrypt_kotlin(text.as_bytes(), password)
}

pub fn decrypt_text(text: &str, password: &str) -> Option<String> {
    decrypt_any(text, password).and_then(|bytes| String::from_utf8(bytes).ok())
}

pub fn pack(files: &[BackupFile]) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for file in files {
        if file.bytes.is_empty() {
            continue;
        }
        zip.start_file(&file.name, options)
            .map_err(|error| error.to_string())?;
        zip.write_all(&file.bytes)
            .map_err(|error| error.to_string())?;
    }
    let cursor = zip.finish().map_err(|error| error.to_string())?;
    Ok(cursor.into_inner())
}

pub fn unpack(bytes: &[u8]) -> Result<Vec<BackupFile>, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("invalid backup zip: {error}"))?;
    let mut files = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        if entry.is_dir() {
            continue;
        }
        let mut data = Vec::new();
        entry
            .read_to_end(&mut data)
            .map_err(|error| error.to_string())?;
        files.push(BackupFile {
            name: entry.name().map_err(|error| error.to_string())?.to_string(),
            bytes: data,
        });
    }
    Ok(files)
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn xml_unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn is_sensitive(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower.contains("password") || lower.contains("secret") || key.contains("密码")
}

pub fn write_config_xml(entries: &[(String, String)], password: Option<&str>) -> String {
    let mut xml =
        String::from("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\" ?>\n<map>\n");
    for (key, value) in entries {
        if is_sensitive(key) {
            if let Some(pass) = password {
                if let Some(encrypted) = encrypt_kotlin(value.as_bytes(), pass) {
                    xml.push_str(&format!(
                        "  <string name=\"{}\" encrypted=\"true\">{}</string>\n",
                        xml_escape(key),
                        xml_escape(&encrypted)
                    ));
                    continue;
                }
            }
        }
        xml.push_str(&format!(
            "  <string name=\"{}\">{}</string>\n",
            xml_escape(key),
            xml_escape(value)
        ));
    }
    xml.push_str("</map>\n");
    xml
}

fn extract_attr(tag: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=\"");
    tag.split(&marker)
        .nth(1)
        .and_then(|value| value.split('"').next())
        .map(str::to_string)
}

pub fn parse_config_xml(xml: &str, password: Option<&str>) -> Vec<ConfigEntry> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(position) = rest.find('<') {
        rest = &rest[position + 1..];
        let Some(tag_end) = rest.find('>') else {
            break;
        };
        let tag = &rest[..tag_end];
        if tag.starts_with('/') || tag.starts_with('?') || tag.starts_with('!') {
            rest = &rest[tag_end + 1..];
            continue;
        }
        let self_closing = tag.ends_with('/');
        let kind = tag.split_whitespace().next().unwrap_or("");
        let supported = matches!(kind, "string" | "int" | "long" | "float" | "boolean");
        let Some(key) = extract_attr(tag, "name").map(|value| xml_unescape(&value)) else {
            rest = &rest[tag_end + 1..];
            continue;
        };
        if !supported || key.is_empty() {
            rest = &rest[tag_end + 1..];
            continue;
        }
        let encrypted = tag.contains("encrypted=\"true\"");
        let value = if self_closing {
            extract_attr(tag, "value").unwrap_or_default()
        } else if kind == "string" || kind == "boolean" {
            let after = &rest[tag_end + 1..];
            let closing = format!("</{kind}>");
            let Some(end) = after.find(&closing) else {
                break;
            };
            let body = &after[..end];
            rest = &after[end + closing.len()..];
            xml_unescape(body)
        } else {
            extract_attr(tag, "value").unwrap_or_default()
        };
        let (value, decrypted) = if encrypted {
            match password.and_then(|pass| decrypt_any(&value, pass)) {
                Some(bytes) => (String::from_utf8_lossy(&bytes).to_string(), true),
                None => (value, false),
            }
        } else {
            (value, true)
        };
        out.push(ConfigEntry {
            key,
            value,
            encrypted,
            decrypted,
        });
        if self_closing {
            rest = &rest[tag_end + 1..];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_round_trip() {
        let files = vec![BackupFile {
            name: "bookshelf.json".into(),
            bytes: b"[]".to_vec(),
        }];
        let packed = pack(&files).unwrap();
        let restored = unpack(&packed).unwrap();
        assert_eq!(restored[0].name, "bookshelf.json");
        assert_eq!(restored[0].bytes, b"[]");
    }

    #[test]
    fn kotlin_config_round_trip() {
        let xml = write_config_xml(&[("password".into(), "secret".into())], Some("123456"));
        let entries = parse_config_xml(&xml, Some("123456"));
        assert_eq!(entries[0].value, "secret");
        assert!(entries[0].decrypted);
    }
}
