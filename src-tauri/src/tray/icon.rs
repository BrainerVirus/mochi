use tauri::image::Image;

use crate::core::models::ProviderId;

use super::presentation::{TrayIconPresentation, TraySelection};

const GLYPH_SIZE: u32 = 18;
const ICON_SIZE: u32 = 22;
const H_PADDING: u32 = 2;

/// Template-friendly menu bar glyph only — percent text is shown via `TrayIcon::set_title`.
pub fn tray_icon_for_presentation(presentation: &TrayIconPresentation) -> Image<'static> {
    let mut rgba = vec![0_u8; (ICON_SIZE * ICON_SIZE * 4) as usize];
    match presentation.selection {
        TraySelection::Overview => blit_overview_glyph(&mut rgba, ICON_SIZE, H_PADDING, H_PADDING),
        TraySelection::Provider(provider) => {
            blit_glyph(provider, &mut rgba, ICON_SIZE, H_PADDING, H_PADDING)
        }
    }
    Image::new_owned(rgba, ICON_SIZE, ICON_SIZE)
}

pub fn tray_icon_fallback() -> Image<'static> {
    let mut rgba = vec![0_u8; (ICON_SIZE * ICON_SIZE * 4) as usize];
    blit_overview_glyph(&mut rgba, ICON_SIZE, H_PADDING, H_PADDING);
    Image::new_owned(rgba, ICON_SIZE, ICON_SIZE)
}

fn blit_overview_glyph(rgba: &mut [u8], canvas_width: u32, x: u32, y: u32) {
    let glyph = overview_glyph_mask();
    for row in 0..GLYPH_SIZE {
        for col in 0..GLYPH_SIZE {
            if !glyph_pixel(&glyph, col, row) {
                continue;
            }
            set_pixel(rgba, canvas_width, x + col, y + row, 0xF5, 0xF5, 0xF5, 255);
        }
    }
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

/// 2×2 grid matching the Overview tab LayoutGrid icon (CodexBar square.grid.2x2).
fn overview_glyph_mask() -> Vec<bool> {
    let mut mask = vec![false; (GLYPH_SIZE * GLYPH_SIZE) as usize];
    let cells = [
        (3_u32, 3_u32, 8_u32, 8_u32),
        (10_u32, 3_u32, 15_u32, 8_u32),
        (3_u32, 10_u32, 8_u32, 15_u32),
        (10_u32, 10_u32, 15_u32, 15_u32),
    ];

    for (left, top, right, bottom) in cells {
        for y in top..=bottom {
            for x in left..=right {
                if x < GLYPH_SIZE && y < GLYPH_SIZE {
                    mask[(y * GLYPH_SIZE + x) as usize] = true;
                }
            }
        }
    }

    mask
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

/// Codex mark: rounded square frame with chevron (matches tray panel SVG).
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
    const TEXT_HEIGHT: u32 = 7;
    let mut mask = vec![false; (GLYPH_SIZE * GLYPH_SIZE) as usize];
    let pattern = letter_pattern(ch);
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
    if y >= ICON_SIZE || x >= width {
        return;
    }
    let index = ((y * width + x) * 4) as usize;
    rgba[index] = r;
    rgba[index + 1] = g;
    rgba[index + 2] = b;
    rgba[index + 3] = a;
}

fn letter_pattern(ch: char) -> [u16; 7] {
    match ch {
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
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
    use crate::tray::presentation::{resolve_tray_presentation, TraySelection};

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
    fn tray_icon_for_overview_renders_grid_glyph() {
        let presentation = resolve_tray_presentation(&[snapshot(ProviderId::Codex, 10.0)], TraySelection::Overview);
        let icon = tray_icon_for_presentation(&presentation);
        assert_eq!(icon.width(), ICON_SIZE);
        assert_eq!(icon.height(), ICON_SIZE);
        assert!(icon.rgba().iter().any(|value| *value > 0));
    }

    #[test]
    fn tray_icon_for_provider_renders_brand_glyph() {
        let presentation =
            resolve_tray_presentation(&[snapshot(ProviderId::Codex, 10.0)], TraySelection::Provider(ProviderId::Codex));
        let icon = tray_icon_for_presentation(&presentation);
        assert_eq!(icon.width(), ICON_SIZE);
        assert_eq!(icon.height(), ICON_SIZE);
        assert!(icon.rgba().iter().any(|value| *value > 0));
    }

    #[test]
    fn tray_icon_fallback_renders_overview_grid() {
        let icon = tray_icon_fallback();
        assert_eq!(icon.width(), ICON_SIZE);
        assert_eq!(icon.height(), ICON_SIZE);
        assert!(icon.rgba().iter().any(|value| *value > 0));
    }
}
