use tauri::image::Image;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

use crate::core::models::ProviderId;

use super::presentation::{TrayIconPresentation, TraySelection};

/// Logical menu-bar glyph size in points (matches CodexBar ProviderBrandIcon / tray slot).
const GLYPH_PT: u32 = 18;
const ICON_PT: u32 = 22;
const H_PADDING_PT: u32 = 2;
/// Render at 2× then output retina bitmap (CodexBar IconRenderer uses outputScale = 2).
const OUTPUT_SCALE: u32 = 2;

const GLYPH_PX: u32 = GLYPH_PT * OUTPUT_SCALE;
const ICON_PX: u32 = ICON_PT * OUTPUT_SCALE;
const H_PADDING_PX: u32 = H_PADDING_PT * OUTPUT_SCALE;

const TEMPLATE_RGB: u8 = 0xF5;

/// Template-friendly menu bar glyph only — percent text is shown via `TrayIcon::set_title`.
pub fn tray_icon_for_presentation(presentation: &TrayIconPresentation) -> Image<'static> {
    let mut rgba = vec![0_u8; (ICON_PX * ICON_PX * 4) as usize];
    match presentation.selection {
        TraySelection::Overview => blit_overview_glyph(&mut rgba),
        TraySelection::Provider(provider) => blit_provider_glyph(provider, &mut rgba),
    }
    Image::new_owned(rgba, ICON_PX, ICON_PX)
}

pub fn tray_icon_fallback() -> Image<'static> {
    let mut rgba = vec![0_u8; (ICON_PX * ICON_PX * 4) as usize];
    blit_overview_glyph(&mut rgba);
    Image::new_owned(rgba, ICON_PX, ICON_PX)
}

fn blit_overview_glyph(rgba: &mut [u8]) {
    let Some(glyph) = render_overview_glyph() else {
        return;
    };
    blit_template_glyph(&glyph, rgba, H_PADDING_PX, H_PADDING_PX);
}

fn blit_provider_glyph(provider: ProviderId, rgba: &mut [u8]) {
    let Some(glyph) = rasterize_provider_svg(provider) else {
        return;
    };
    blit_template_glyph(&glyph, rgba, H_PADDING_PX, H_PADDING_PX);
}

fn blit_template_glyph(glyph: &Pixmap, rgba: &mut [u8], x: u32, y: u32) {
    let width = ICON_PX;
    let data = glyph.data();
    let glyph_width = glyph.width();
    let glyph_height = glyph.height();

    for row in 0..glyph_height {
        for col in 0..glyph_width {
            let src_index = ((row * glyph_width + col) * 4) as usize;
            let alpha = data[src_index + 3];
            if alpha == 0 {
                continue;
            }

            let dst_x = x + col;
            let dst_y = y + row;
            if dst_x >= width || dst_y >= width {
                continue;
            }

            let dst_index = ((dst_y * width + dst_x) * 4) as usize;
            rgba[dst_index] = TEMPLATE_RGB;
            rgba[dst_index + 1] = TEMPLATE_RGB;
            rgba[dst_index + 2] = TEMPLATE_RGB;
            rgba[dst_index + 3] = rgba[dst_index + 3].max(alpha);
        }
    }
}

/// 2×2 grid matching the Overview tab LayoutGrid icon (CodexBar square.grid.2x2).
fn render_overview_glyph() -> Option<Pixmap> {
    let mut pixmap = Pixmap::new(GLYPH_PX, GLYPH_PX)?;
    let scale = OUTPUT_SCALE as f32;
    let mut paint = Paint::default();
    paint.set_color_rgba8(TEMPLATE_RGB, TEMPLATE_RGB, TEMPLATE_RGB, 255);

    let cells = [
        (3.0, 3.0, 8.0, 8.0),
        (10.0, 3.0, 15.0, 8.0),
        (3.0, 10.0, 8.0, 15.0),
        (10.0, 10.0, 15.0, 15.0),
    ];

    for (left, top, right, bottom) in cells {
        let mut path = PathBuilder::new();
        path.push_rect(tiny_skia::Rect::from_ltrb(
            left * scale,
            top * scale,
            (right + 1.0) * scale,
            (bottom + 1.0) * scale,
        )?);
        pixmap.fill_path(
            &path.finish()?,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }

    Some(pixmap)
}

fn rasterize_provider_svg(provider: ProviderId) -> Option<Pixmap> {
    let svg = provider_svg(provider);
    let options = usvg::Options {
        font_family: "sans-serif".to_string(),
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(svg, &options).ok()?;
    let svg_size = tree.size();
    if svg_size.width() <= 0.0 || svg_size.height() <= 0.0 {
        return None;
    }

    let scale = GLYPH_PX as f32 / svg_size.width().max(svg_size.height());
    let scaled_width = svg_size.width() * scale;
    let scaled_height = svg_size.height() * scale;
    let tx = (GLYPH_PX as f32 - scaled_width) / 2.0;
    let ty = (GLYPH_PX as f32 - scaled_height) / 2.0;
    let transform = Transform::from_translate(tx, ty).post_scale(scale, scale);

    let mut pixmap = Pixmap::new(GLYPH_PX, GLYPH_PX)?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Some(pixmap)
}

fn provider_svg(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::Codex => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/codex.svg"
            ))
        }
        ProviderId::Claude => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/claude.svg"
            ))
        }
        ProviderId::Cursor => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/cursor.svg"
            ))
        }
        ProviderId::Gemini => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/gemini.svg"
            ))
        }
        ProviderId::Copilot => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/copilot.svg"
            ))
        }
        ProviderId::Antigravity => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../src/assets/providers/antigravity.svg"
        )),
        ProviderId::Factory => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/factory.svg"
            ))
        }
        ProviderId::Zai => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/zai.svg"
            ))
        }
        ProviderId::Kiro => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/kiro.svg"
            ))
        }
        ProviderId::Augment => {
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../src/assets/providers/augment.svg"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ProviderId, UsageSnapshot, UsageWindow};
    use crate::tray::presentation::{resolve_tray_presentation, TraySelection};

    fn snapshot(provider: ProviderId, used_percent: f32) -> UsageSnapshot {
        UsageSnapshot::new(
            provider,
            UsageWindow::new("Session", used_percent, None),
            None,
            "1970-01-01T00:00:00Z",
            "test",
        )
    }

    fn opaque_pixel_count(icon: &Image<'_>) -> usize {
        icon.rgba().chunks(4).filter(|px| px[3] > 0).count()
    }

    fn semi_transparent_pixel_count(icon: &Image<'_>) -> usize {
        icon.rgba()
            .chunks(4)
            .filter(|px| px[3] > 0 && px[3] < 255)
            .count()
    }

    #[test]
    fn tray_icon_for_overview_renders_grid_glyph() {
        let presentation = resolve_tray_presentation(
            &[snapshot(ProviderId::Codex, 10.0)],
            TraySelection::Overview,
        );
        let icon = tray_icon_for_presentation(&presentation);
        assert_eq!(icon.width(), ICON_PX);
        assert_eq!(icon.height(), ICON_PX);
        assert!(opaque_pixel_count(&icon) > 0);
    }

    #[test]
    fn tray_icon_for_provider_renders_brand_glyph() {
        let presentation = resolve_tray_presentation(
            &[snapshot(ProviderId::Codex, 10.0)],
            TraySelection::Provider(ProviderId::Codex),
        );
        let icon = tray_icon_for_presentation(&presentation);
        assert_eq!(icon.width(), ICON_PX);
        assert_eq!(icon.height(), ICON_PX);
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
    fn codex_provider_glyph_uses_anti_aliased_edges() {
        let icon = tray_icon_for_presentation(&resolve_tray_presentation(
            &[snapshot(ProviderId::Codex, 10.0)],
            TraySelection::Provider(ProviderId::Codex),
        ));
        assert!(
            semi_transparent_pixel_count(&icon) > 0,
            "expected SVG rasterization to produce soft alpha edges"
        );
    }

    #[test]
    fn tray_icon_fallback_renders_overview_grid() {
        let icon = tray_icon_fallback();
        assert_eq!(icon.width(), ICON_PX);
        assert_eq!(icon.height(), ICON_PX);
        assert!(opaque_pixel_count(&icon) > 0);
    }
}
