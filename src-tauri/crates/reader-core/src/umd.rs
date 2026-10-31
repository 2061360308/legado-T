use std::io::Read;

use flate2::read::ZlibDecoder;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UmdCover {
    pub bytes: Vec<u8>,
    pub ext: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UmdChapter {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UmdBook {
    pub title: String,
    pub author: String,
    pub kind: String,
    pub cover: Option<UmdCover>,
    pub chapters: Vec<UmdChapter>,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        let value = *self
            .bytes
            .get(self.pos)
            .ok_or_else(|| format!("UMD truncated at {}", self.pos))?;
        self.pos += 1;
        Ok(value)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| "UMD length overflow".to_string())?;
        if end > self.bytes.len() {
            return Err(format!(
                "UMD truncated at {} need {} (file {})",
                self.pos,
                len,
                self.bytes.len()
            ));
        }
        let value = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(value)
    }

    fn read_u16_le(&mut self) -> Result<u16, String> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32_le(&mut self) -> Result<u32, String> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn skip(&mut self, len: usize) -> Result<(), String> {
        self.read_bytes(len).map(|_| ())
    }
}

fn utf16_le(bytes: &[u8]) -> String {
    let mut values = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        values.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&values)
}

fn cover_extension(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg".into()
    } else if bytes.starts_with(b"\x89PNG") {
        "png".into()
    } else if bytes.starts_with(b"GIF8") {
        "gif".into()
    } else if bytes.starts_with(b"RIFF") {
        "webp".into()
    } else {
        "jpg".into()
    }
}

fn decompress(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|error| format!("UMD decompress failed: {error}"))?;
    Ok(out)
}

fn scan_content_chunks(bytes: &[u8], start: usize) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos = start;
    while pos + 9 <= bytes.len() {
        if bytes[pos] != b'$' {
            pos += 1;
            continue;
        }
        let length = u32::from_le_bytes([
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
            bytes[pos + 8],
        ]) as usize;
        if length >= 9 && pos + length <= bytes.len() {
            if let Ok(data) = decompress(&bytes[pos + 9..pos + length]) {
                out.extend_from_slice(&data);
                pos += length;
                if bytes[pos..].starts_with(&[b'#', 0xF1, 0, 0, 0x15]) {
                    pos += 21;
                }
                continue;
            }
        }
        pos += 1;
    }
    out
}

pub fn parse(bytes: &[u8]) -> Result<UmdBook, String> {
    let mut cursor = Cursor::new(bytes);
    if cursor.read_u32_le()? != 0xDE9A9B89 {
        return Err("not a valid UMD file".to_string());
    }

    let mut title = String::new();
    let mut author = String::new();
    let mut kind = String::new();
    let mut cover: Option<UmdCover> = None;
    let mut content = Vec::new();
    let mut content_offsets: Vec<u32> = Vec::new();
    let mut titles: Vec<String> = Vec::new();
    let mut total_content_len = 0usize;
    let mut check_number = 0u32;
    let mut chunk_start: Option<usize> = None;

    let mut marker = cursor.read_u8()?;
    while marker == b'#' {
        let seg_type = cursor.read_u16_le()?;
        let _seg_flag = cursor.read_u8()?;
        let length = cursor.read_u8()? as usize;
        if length < 5 {
            return Err("UMD segment length invalid".to_string());
        }
        let length = length - 5;
        match seg_type {
            1 => {
                let _umd_type = cursor.read_u8()?;
                cursor.skip(2)?;
            }
            2 => title = utf16_le(cursor.read_bytes(length)?),
            3 => author = utf16_le(cursor.read_bytes(length)?),
            7 => kind = utf16_le(cursor.read_bytes(length)?),
            11 => total_content_len = cursor.read_u32_le()? as usize,
            12 => {
                let _file_len = cursor.read_u32_le()?;
            }
            129 | 131 | 132 => check_number = cursor.read_u32_le()?,
            130 => {
                let _ = cursor.read_u8()?;
                check_number = cursor.read_u32_le()?;
            }
            135 => cursor.skip(6)?,
            241 => cursor.skip(16.min(length))?,
            _ => cursor.skip(length)?,
        }

        let mut next = cursor.read_u8()?;
        while next == b'$' {
            let additional_check = cursor.read_u32_le()?;
            let additional_len = cursor.read_u32_le()? as usize;
            if additional_len < 9 {
                return Err("UMD additional segment invalid".to_string());
            }
            let additional_len = additional_len - 9;
            match seg_type {
                130 => {
                    let data = cursor.read_bytes(additional_len)?;
                    if !data.is_empty() {
                        cover = Some(UmdCover {
                            ext: cover_extension(data),
                            bytes: data.to_vec(),
                        });
                    }
                }
                131 => {
                    for _ in 0..additional_len / 4 {
                        content_offsets.push(cursor.read_u32_le()?);
                    }
                    if additional_len % 4 != 0 {
                        cursor.skip(additional_len % 4)?;
                    }
                }
                132 => {
                    if check_number != additional_check {
                        let data = cursor.read_bytes(additional_len)?;
                        content.extend_from_slice(&decompress(data)?);
                    } else {
                        let end = cursor.pos + additional_len;
                        while cursor.pos < end {
                            let len = cursor.read_u8()? as usize;
                            titles.push(utf16_le(cursor.read_bytes(len)?));
                        }
                        chunk_start = Some(cursor.pos);
                    }
                }
                _ => cursor.skip(additional_len)?,
            }
            next = cursor.read_u8()?;
        }
        marker = next;
    }

    if let Some(start) = chunk_start {
        let scanned = scan_content_chunks(bytes, start);
        if !scanned.is_empty() {
            content = scanned;
        }
    }
    if content.is_empty() && total_content_len > 0 {
        return Err("UMD content is empty".to_string());
    }
    if content_offsets.is_empty() {
        content_offsets.push(0);
    }
    if titles.is_empty() {
        titles.push(if title.is_empty() {
            "正文".to_string()
        } else {
            title.clone()
        });
    }

    let mut chapters = Vec::new();
    for index in 0..content_offsets.len() {
        let start = content_offsets[index] as usize;
        let end = content_offsets
            .get(index + 1)
            .map(|value| *value as usize)
            .unwrap_or(content.len());
        if start > content.len() || end > content.len() || start > end {
            continue;
        }
        let text = utf16_le(&content[start..end])
            .replace('\u{2029}', "\n")
            .replace('\r', "\n");
        let lines = text
            .split('\n')
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        if lines.is_empty() {
            continue;
        }
        let chapter_title = titles
            .get(index)
            .cloned()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("第{}章", index + 1));
        chapters.push(UmdChapter {
            title: chapter_title,
            text: lines.join("\n"),
        });
    }
    if chapters.is_empty() {
        chapters.push(UmdChapter {
            title: if title.is_empty() {
                "正文".to_string()
            } else {
                title.clone()
            },
            text: utf16_le(&content)
                .replace('\u{2029}', "\n")
                .replace('\r', "\n"),
        });
    }
    Ok(UmdBook {
        title,
        author,
        kind,
        cover,
        chapters,
    })
}
