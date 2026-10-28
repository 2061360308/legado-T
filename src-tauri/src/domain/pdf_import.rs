//! PDF import (pdfium): page screenshots plus the embedded text layer.
//! OCR is triggered on demand by the reader; import never blocks on OCR.

use std::fs;
use std::path::{Path, PathBuf};

use pdfium_render::prelude::*;

use crate::domain::model::ContentBlock;
use crate::{Error, Result};

pub struct ParsedPdfPage {
    pub page_number: u32,
    pub text: String,
    pub image: PathBuf,
    pub width: u32,
    pub height: u32,
}

pub struct ParsedPdf {
    pub pages: Vec<ParsedPdfPage>,
}

fn pdfium_candidates() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(dir) = std::env::var("PDFIUM_LIB_PATH") {
        dirs.push(PathBuf::from(dir));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.to_path_buf());
            dirs.push(dir.join("deps"));
            dirs.push(dir.join("pdfium"));
            dirs.push(dir.join("resources").join("pdfium"));
        }
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        dirs.push(
            PathBuf::from(local)
                .join("pdfium-rs")
                .join("chromium_8058")
                .join("pdfium-win-x64")
                .join("bin"),
        );
    }
    dirs
}

fn bind_pdfium() -> Result<Pdfium> {
    for dir in pdfium_candidates() {
        let path = Pdfium::pdfium_platform_library_name_at_path(&dir);
        if let Ok(bindings) = Pdfium::bind_to_library(path) {
            return Ok(Pdfium::new(bindings));
        }
    }
    Pdfium::bind_to_system_library()
        .map(Pdfium::new)
        .map_err(|e| Error::Invalid(format!("PDFium unavailable: {e}")))
}

pub fn parse(path: &Path, image_dir: &Path) -> Result<ParsedPdf> {
    fs::create_dir_all(image_dir)?;
    let pdfium = bind_pdfium()?;
    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| Error::Invalid(format!("PDF open failed: {e}")))?;
    let render_config = PdfRenderConfig::new().set_target_width(1240);

    let mut pages = Vec::with_capacity(document.pages().len() as usize);
    for (index, page) in document.pages().iter().enumerate() {
        let page_number = index as u32 + 1;
        let text = page
            .text()
            .map(|text| text.all())
            .unwrap_or_default();
        let bitmap = page
            .render_with_config(&render_config)
            .map_err(|e| Error::Invalid(format!("PDF page render failed: {e}")))?;
        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        let image = image_dir.join(format!("page_{page_number}.png"));
        bitmap
            .as_image()
            .map_err(|e| Error::Invalid(format!("PDF page image failed: {e}")))?
            .save(&image)
            .map_err(|e| Error::Internal(format!("PDF page image save failed: {e}")))?;
        pages.push(ParsedPdfPage {
            page_number,
            text,
            image,
            width,
            height,
        });
    }
    if pages.is_empty() {
        return Err(Error::Invalid("PDF has no readable pages".into()));
    }
    Ok(ParsedPdf { pages })
}

pub fn page_blocks(page: &ParsedPdfPage) -> Vec<ContentBlock> {
    let text_len = page.text.chars().filter(|ch| !ch.is_whitespace()).count();
    if text_len >= 20 {
        let mut blocks = Vec::new();
        for line in page.text.lines() {
            let line = line.trim();
            if !line.is_empty() {
                blocks.push(ContentBlock::paragraph(line));
            }
        }
        return blocks;
    }
    vec![ContentBlock::ImagePage {
        src: page.image.to_string_lossy().to_string(),
        page: page.page_number,
        width: page.width,
        height: page.height,
    }]
}
