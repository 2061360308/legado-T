use std::cell::RefCell;

use ttf_parser::Face;

static FONT_BYTES: &[u8] = include_bytes!("../assets/NotoSansSC-Regular.otf");

thread_local! {
    static FACE: RefCell<Option<Face<'static>>> =
        RefCell::new(Face::parse(FONT_BYTES, 0).ok());
}

fn with_face<R>(f: impl FnOnce(&Face<'static>) -> R) -> Option<R> {
    FACE.with(|cell| cell.borrow().as_ref().map(f))
}

fn fallback_advance(ch: char, font_size: f32, letter_spacing: f32) -> f32 {
    if ch == '\n' || ch == '\r' {
        return 0.0;
    }
    crate::domain::text_measure::char_width(ch) as f32 / 1000.0 * font_size
        + letter_spacing * font_size
}

pub fn advance_px(ch: char, font_size: f32, letter_spacing: f32) -> f32 {
    with_face(|face| {
        let units = face.units_per_em() as f32;
        let glyph = face
            .glyph_index(ch)
            .unwrap_or_else(|| ttf_parser::GlyphId(0));
        let advance = face
            .glyph_hor_advance(glyph)
            .map(|value| value as f32)
            .unwrap_or_else(|| if ch.is_ascii() { units / 2.0 } else { units });
        advance / units * font_size + letter_spacing * font_size
    })
    .unwrap_or_else(|| fallback_advance(ch, font_size, letter_spacing))
}

pub fn measure_px(text: &str, font_size: f32, letter_spacing: f32) -> f32 {
    text.chars()
        .map(|ch| advance_px(ch, font_size, letter_spacing))
        .sum()
}

pub fn cn_char_width(font_size: f32, letter_spacing: f32) -> f32 {
    advance_px('我', font_size, letter_spacing)
}

pub fn line_metrics(font_size: f32) -> (f32, f32, f32) {
    with_face(|face| {
        let units = face.units_per_em() as f32;
        let scale = font_size / units;
        (
            face.ascender() as f32 * scale,
            (-face.descender()) as f32 * scale,
            face.line_gap() as f32 * scale,
        )
    })
    .unwrap_or((font_size * 0.88, font_size * 0.24, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_font_has_cjk_advance() {
        let width = advance_px('我', 20.0, 0.0);
        assert!((18.0..=22.0).contains(&width), "{width}");
    }

    #[test]
    fn missing_glyph_does_not_panic() {
        assert!(advance_px('\u{10ffff}', 20.0, 0.0) >= 0.0);
    }
}
