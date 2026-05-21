use tauri::image::Image;

use crate::core::models::ProviderId;

use super::presentation::TrayIconPresentation;

const GLYPH_SIZE: u32 = 18;
const ICON_HEIGHT: u32 = 22;
const TEXT_HEIGHT: u32 = 7;
const TEXT_SCALE: u32 = 2;
const CHAR_WIDTH: u32 = 5 * TEXT_SCALE + 1;
const H_PADDING: u32 = 2;

pub fn tray_icon_for_presentation(presentation: &TrayIconPresentation) -> Image<'static> {
    let provider = presentation.provider.unwrap_or(ProviderId::Codex);
    let label = presentation
        .title
        .clone()
        .unwrap_or_else(|| format!("{}%", presentation.remaining_percent));

    let text_width = text_pixel_width(&label);
    let width = H_PADDING + GLYPH_SIZE + 2 + text_width + H_PADDING;
    let mut rgba = vec![0_u8; (width * ICON_HEIGHT * 4) as usize];

    blit_glyph(provider, &mut rgba, width, 0, 2);
    draw_text(&label, &mut rgba, width, H_PADDING + GLYPH_SIZE + 2, 7);

    Image::new_owned(rgba, width, ICON_HEIGHT)
}

pub fn tray_icon_fallback() -> Image<'static> {
    let width = H_PADDING + GLYPH_SIZE + H_PADDING;
    let mut rgba = vec![0_u8; (width * ICON_HEIGHT * 4) as usize];
    blit_glyph(ProviderId::Codex, &mut rgba, width, H_PADDING, 2);
    Image::new_owned(rgba, width, ICON_HEIGHT)
}

fn blit_glyph(provider: ProviderId, rgba: &mut [u8], canvas_width: u32, x: u32, y: u32) {
    let glyph = provider_glyph_mask(provider);
    for row in 0..GLYPH_SIZE {
        for col in 0..GLYPH_SIZE {
            if !glyph_pixel(&glyph, col, row) {
                continue;
            }
            set_pixel(rgba, canvas_width, x + col, y + row, 0xF5, 0xF5, 0xF5, 255);
        }
    }
}

fn provider_glyph_mask(provider: ProviderId) -> Vec<bool> {
    match provider {
        ProviderId::Codex => codex_glyph_mask(),
        ProviderId::Claude => letter_glyph_mask('C'),
        ProviderId::Cursor => letter_glyph_mask('U'),
        ProviderId::Gemini => letter_glyph_mask('G'),
        ProviderId::Copilot => letter_glyph_mask('P'),
        ProviderId::Antigravity => letter_glyph_mask('A'),
        ProviderId::Factory => letter_glyph_mask('F'),
        ProviderId::Zai => letter_glyph_mask('Z'),
        ProviderId::Kiro => letter_glyph_mask('K'),
        ProviderId::Augment => letter_glyph_mask('+'),
    }
}

/// Simplified Codex mark: rounded square frame with a chevron cut-in (template-friendly).
fn codex_glyph_mask() -> Vec<bool> {
    let mut mask = vec![false; (GLYPH_SIZE * GLYPH_SIZE) as usize];
    let inset = 2_u32;

    for y in 0..GLYPH_SIZE {
        for x in 0..GLYPH_SIZE {
            let on_border = x <= inset
                || y <= inset
                || x >= GLYPH_SIZE - 1 - inset
                || y >= GLYPH_SIZE - 1 - inset;
            let in_chevron = x >= 7 && x <= 11 && y >= 6 && y <= 12 && x + 1 >= y && x <= y + 2;
            if on_border && !in_chevron {
                mask[(y * GLYPH_SIZE + x) as usize] = true;
            }
        }
    }

    mask
}

fn letter_glyph_mask(ch: char) -> Vec<bool> {
    let mut mask = vec![false; (GLYPH_SIZE * GLYPH_SIZE) as usize];
    let pattern = glyph_pattern(ch);
    let offset_x = 5_u32;
    let offset_y = 4_u32;

    for row in 0..TEXT_HEIGHT {
        for col in 0..5 {
            if pattern[row as usize] & (1 << (4 - col)) != 0 {
                let x = offset_x + col;
                let y = offset_y + row;
                if x < GLYPH_SIZE && y < GLYPH_SIZE {
                    mask[(y * GLYPH_SIZE + x) as usize] = true;
                }
            }
        }
    }

    mask
}

fn glyph_pixel(mask: &[bool], x: u32, y: u32) -> bool {
    mask[(y * GLYPH_SIZE + x) as usize]
}

fn set_pixel(rgba: &mut [u8], width: u32, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
    if y >= ICON_HEIGHT || x >= width {
        return;
    }
    let index = ((y * width + x) * 4) as usize;
    rgba[index] = r;
    rgba[index + 1] = g;
    rgba[index + 2] = b;
    rgba[index + 3] = a;
}

fn text_pixel_width(text: &str) -> u32 {
    text.chars()
        .map(|ch| if ch == '%' { CHAR_WIDTH + 1 } else { CHAR_WIDTH })
        .sum()
}

fn draw_text(text: &str, rgba: &mut [u8], canvas_width: u32, mut x: u32, y: u32) {
    for ch in text.chars() {
        let pattern = glyph_pattern(ch);
        for row in 0..TEXT_HEIGHT {
            for col in 0..5 {
                if pattern[row as usize] & (1 << (4 - col)) == 0 {
                    continue;
                }
                for sy in 0..TEXT_SCALE {
                    for sx in 0..TEXT_SCALE {
                        set_pixel(
                            rgba,
                            canvas_width,
                            x + col * TEXT_SCALE + sx,
                            y + row * TEXT_SCALE + sy,
                            0xF5,
                            0xF5,
                            0xF5,
                            255,
                        );
                    }
                }
            }
        }
        x += if ch == '%' { CHAR_WIDTH + 1 } else { CHAR_WIDTH };
    }
}

fn glyph_pattern(ch: char) -> [u16; 7] {
    match ch {
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        '%' => [0b10001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10001, 0b00000],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        '+' => [0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000, 0b00000],
        _ => [0b00000; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tray::presentation::resolve_tray_presentation;
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};

    fn snapshot(provider: ProviderId, used_percent: f32) -> UsageSnapshot {
        UsageSnapshot {
            provider,
            primary: UsageWindow::new("Session", used_percent, None),
            secondary: None,
            updated_at: "1970-01-01T00:00:00Z".to_string(),
            source: "test".to_string(),
        }
    }

    #[test]
    fn tray_icon_for_presentation_has_nonzero_pixels() {
        let presentation = resolve_tray_presentation(&[snapshot(ProviderId::Codex, 10.0)]);
        let icon = tray_icon_for_presentation(&presentation);
        assert!(icon.width() > GLYPH_SIZE);
        assert_eq!(icon.height(), ICON_HEIGHT);
        assert!(icon.rgba().iter().any(|value| *value > 0));
    }

    #[test]
    fn tray_icon_fallback_renders_without_title() {
        let icon = tray_icon_fallback();
        assert_eq!(icon.height(), ICON_HEIGHT);
        assert!(icon.rgba().iter().any(|value| *value > 0));
    }
}
