use tauri::image::Image;

use super::usage::TrayUsageTone;

const ICON_SIZE: u32 = 32;

pub fn tray_icon_for_tone(tone: TrayUsageTone) -> Image<'static> {
    let (red, green, blue) = match tone {
        TrayUsageTone::Normal => (0xA3, 0xD9, 0xA5),
        TrayUsageTone::Warning => (0xFF, 0xE4, 0xA1),
        TrayUsageTone::Critical => (0xFF, 0x8A, 0x8A),
    };

    let rgba = circle_icon_rgba(red, green, blue);
    Image::new_owned(rgba, ICON_SIZE, ICON_SIZE)
}

fn circle_icon_rgba(red: u8, green: u8, blue: u8) -> Vec<u8> {
    let radius = (ICON_SIZE / 2) as i32 - 2;
    let center = (ICON_SIZE / 2) as i32;
    let radius_sq = radius * radius;
    let inner_border_sq = (radius - 2) * (radius - 2);
    let mut rgba = vec![0_u8; (ICON_SIZE * ICON_SIZE * 4) as usize];

    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let dx = x as i32 - center;
            let dy = y as i32 - center;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > radius_sq {
                continue;
            }

            let index = ((y * ICON_SIZE + x) * 4) as usize;
            let is_border = dist_sq >= inner_border_sq;
            if is_border {
                // Dark ring improves contrast on macOS light menu bars.
                rgba[index] = 0x2A;
                rgba[index + 1] = 0x2A;
                rgba[index + 2] = 0x2A;
            } else {
                rgba[index] = red;
                rgba[index + 1] = green;
                rgba[index + 2] = blue;
            }
            rgba[index + 3] = 255;
        }
    }

    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_icon_for_each_tone_has_expected_dimensions() {
        for tone in [
            TrayUsageTone::Normal,
            TrayUsageTone::Warning,
            TrayUsageTone::Critical,
        ] {
            let icon = tray_icon_for_tone(tone);
            assert_eq!(icon.width(), ICON_SIZE);
            assert_eq!(icon.height(), ICON_SIZE);
            assert_eq!(icon.rgba().len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
        }
    }
}
