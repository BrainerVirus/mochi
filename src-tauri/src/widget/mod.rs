mod commands;

pub use commands::{hide_widget, set_widget_height, setup_widget, show_widget, toggle_widget};

use serde::{Deserialize, Serialize};

pub const WIDGET_LABEL: &str = "widget";
pub const WIDGET_MIN_WIDTH: f64 = 280.0;
pub const WIDGET_MAX_WIDTH: f64 = 480.0;
pub const WIDGET_MIN_HEIGHT: f64 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WidgetDensity {
    Compact,
    Normal,
    Expanded,
}

pub fn clamp_widget_width(width: f64) -> f64 {
    width.clamp(WIDGET_MIN_WIDTH, WIDGET_MAX_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_widget_width_enforces_design_bounds() {
        assert_eq!(clamp_widget_width(100.0), WIDGET_MIN_WIDTH);
        assert_eq!(clamp_widget_width(320.0), 320.0);
        assert_eq!(clamp_widget_width(900.0), WIDGET_MAX_WIDTH);
    }

    #[test]
    fn widget_density_serializes_as_kebab_case() {
        let json = serde_json::to_string(&WidgetDensity::Expanded).expect("serialize");
        assert_eq!(json, "\"expanded\"");
    }
}
