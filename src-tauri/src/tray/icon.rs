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
    blit_mask(&glyph, rgba, canvas_width, x, y);
}

fn blit_glyph(provider: ProviderId, rgba: &mut [u8], canvas_width: u32, x: u32, y: u32) {
    let glyph = provider_glyph_mask(provider);
    blit_mask(&glyph, rgba, canvas_width, x, y);
}

fn blit_mask(mask: &[bool], rgba: &mut [u8], canvas_width: u32, x: u32, y: u32) {
    for row in 0..GLYPH_SIZE {
        for col in 0..GLYPH_SIZE {
            if !glyph_pixel(mask, col, row) {
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
    mask_from_rows(match provider {
        ProviderId::Codex => CODEX_GLYPH,
        ProviderId::Claude => CLAUDE_GLYPH,
        ProviderId::Cursor => CURSOR_GLYPH,
        ProviderId::Gemini => GEMINI_GLYPH,
        ProviderId::Copilot => COPILOT_GLYPH,
        ProviderId::Antigravity => ANTIGRAVITY_GLYPH,
        ProviderId::Factory => FACTORY_GLYPH,
        ProviderId::Zai => ZAI_GLYPH,
        ProviderId::Kiro => KIRO_GLYPH,
        ProviderId::Augment => AUGMENT_GLYPH,
    })
}

fn mask_from_rows(rows: [&str; 18]) -> Vec<bool> {
    let mut mask = vec![false; (GLYPH_SIZE * GLYPH_SIZE) as usize];
    for (row, pattern) in rows.iter().enumerate() {
        for (col, ch) in pattern.chars().enumerate() {
            if ch == '1' {
                mask[row * GLYPH_SIZE as usize + col] = true;
            }
        }
    }
    mask
}

// Rasterized from `src/assets/providers/*.svg` at 18×18 (resvg, template-friendly).
const CODEX_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000001000000000",
    "000000111110000000",
    "000001100111110000",
    "000111011100011000",
    "001111111011001000",
    "001111111111111100",
    "011011111110011100",
    "001011110011101100",
    "001101110011110100",
    "001110111111110100",
    "001111111111111100",
    "001100110110111100",
    "000110011110110000",
    "000011111001100000",
    "000000011111000000",
    "000000000100000000",
    "000000000000000000",
];

const CLAUDE_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000000000000000",
    "000001100110000000",
    "000001110110010000",
    "000100110110111000",
    "000111111111110000",
    "000011111111100000",
    "000001111111011100",
    "001111111111111100",
    "001111111111111000",
    "000000111111111100",
    "000011111111100000",
    "000111111111110000",
    "000001101101111000",
    "000001101101100000",
    "000000001100000000",
    "000000000000000000",
    "000000000000000000",
];

const CURSOR_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000000000000000",
    "000000011110000000",
    "000001111111100000",
    "000011111111110000",
    "001111111111111100",
    "001110000000001100",
    "001111100000001100",
    "001111111000011100",
    "001111111000111100",
    "001111111000111100",
    "001111111001111100",
    "001111111001111100",
    "000011111011110000",
    "000001111111100000",
    "000000011110000000",
    "000000000000000000",
    "000000000000000000",
];

const GEMINI_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000000000000000",
    "000000001100000000",
    "000000001100000000",
    "000000011110000000",
    "000000111111000000",
    "000001111111100000",
    "000011111111110000",
    "001111111111111100",
    "001111111111111100",
    "000011111111110000",
    "000000111111000000",
    "000000011110000000",
    "000000011110000000",
    "000000001100000000",
    "000000001100000000",
    "000000000000000000",
    "000000000000000000",
];

const COPILOT_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000001100000000",
    "000001111111100000",
    "000011111111110000",
    "000100011110001000",
    "001100001100001100",
    "001100011110001100",
    "001100111111001100",
    "011111110011111110",
    "111100000000001111",
    "111100110011001111",
    "111100110011001111",
    "111100110011001111",
    "011100000000001110",
    "000111110011111000",
    "000001111111100000",
    "000000000000000000",
    "000000000000000000",
];

const ANTIGRAVITY_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000000000000000",
    "000000011110000000",
    "000000111111000000",
    "000000111111000000",
    "000001111111100000",
    "000001111111100000",
    "000001111111100000",
    "000011111111110000",
    "000011111111110000",
    "000011100001110000",
    "000111100000111000",
    "000111000000111000",
    "001110000000011100",
    "001110000000011100",
    "011100000000001110",
    "000000000000000000",
    "000000000000000000",
];

const FACTORY_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000000000000000",
    "000001100111100000",
    "000001111111100000",
    "000011010011110000",
    "001010011011011100",
    "011111011010001110",
    "001011111110111100",
    "001100011111101000",
    "000101111110001100",
    "001111011111110100",
    "011100010110111110",
    "001110110110011100",
    "000011110010110000",
    "000001111111100000",
    "000001111011100000",
    "000000000000000000",
    "000000000000000000",
];

const ZAI_GLYPH: [&str; 18] = [
    "000000000000000000",
    "011111111001111110",
    "011111111011111110",
    "011111111111111110",
    "000000001111111100",
    "000000001111111000",
    "000000011111111000",
    "000000111111110000",
    "000000111111100000",
    "000001111111000000",
    "000011111111000000",
    "000111111110000000",
    "000111111100000000",
    "001111111100000000",
    "011111111111111110",
    "011111110111111110",
    "011111100111111110",
    "000000000000000000",
];

const KIRO_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000000000000000",
    "000000000000000000",
    "000000011111000000",
    "000000111111100000",
    "000001111111110000",
    "000001111111110000",
    "000001111110110000",
    "000001111111110000",
    "000011111111110000",
    "000011111111110000",
    "000011111111110000",
    "000011111111100000",
    "000001111111100000",
    "000000111111000000",
    "000000000000000000",
    "000000000000000000",
    "000000000000000000",
];

const AUGMENT_GLYPH: [&str; 18] = [
    "000000000000000000",
    "000000010100000000",
    "000000010101110000",
    "011111100111111000",
    "011000000000001100",
    "011000000000001100",
    "011000000000001100",
    "011000000000001100",
    "110001000001001110",
    "011011100011101100",
    "011011000001101100",
    "011000000000001100",
    "011000000011111100",
    "001111100011111000",
    "000111000000000000",
    "000000000000000000",
    "000000000000000000",
    "000000000000000000",
];

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

    fn opaque_pixel_count(icon: &Image<'_>) -> usize {
        icon.rgba().chunks(4).filter(|px| px[3] > 0).count()
    }

    #[test]
    fn tray_icon_for_overview_renders_grid_glyph() {
        let presentation =
            resolve_tray_presentation(&[snapshot(ProviderId::Codex, 10.0)], TraySelection::Overview);
        let icon = tray_icon_for_presentation(&presentation);
        assert_eq!(icon.width(), ICON_SIZE);
        assert_eq!(icon.height(), ICON_SIZE);
        assert!(opaque_pixel_count(&icon) > 0);
    }

    #[test]
    fn tray_icon_for_provider_renders_brand_glyph() {
        let presentation = resolve_tray_presentation(
            &[snapshot(ProviderId::Codex, 10.0)],
            TraySelection::Provider(ProviderId::Codex),
        );
        let icon = tray_icon_for_presentation(&presentation);
        assert_eq!(icon.width(), ICON_SIZE);
        assert_eq!(icon.height(), ICON_SIZE);
        assert!(opaque_pixel_count(&icon) > 0);
    }

    #[test]
    fn codex_provider_glyph_differs_from_overview_grid() {
        let overview = tray_icon_for_presentation(&resolve_tray_presentation(
            &[snapshot(ProviderId::Codex, 10.0)],
            TraySelection::Overview,
        ));
        let codex = tray_icon_for_presentation(&resolve_tray_presentation(
            &[snapshot(ProviderId::Codex, 10.0)],
            TraySelection::Provider(ProviderId::Codex),
        ));
        assert_ne!(overview.rgba(), codex.rgba());
    }

    #[test]
    fn tray_icon_fallback_renders_overview_grid() {
        let icon = tray_icon_fallback();
        assert_eq!(icon.width(), ICON_SIZE);
        assert_eq!(icon.height(), ICON_SIZE);
        assert!(opaque_pixel_count(&icon) > 0);
    }
}
