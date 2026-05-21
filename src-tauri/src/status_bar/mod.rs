use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WaybarOutput {
    pub text: String,
    pub tooltip: String,
    pub class: String,
    pub percentage: u8,
}

pub fn waybar_output(label: &str, percentage: u8) -> WaybarOutput {
    let class = match percentage {
        0..=69 => "ok",
        70..=89 => "warning",
        _ => "critical",
    };

    WaybarOutput {
        text: format!("Mochi {percentage}%"),
        tooltip: label.to_string(),
        class: class.to_string(),
        percentage,
    }
}

pub fn format_output(format: &str, percentage: u8, label: &str) -> String {
    match format {
        "waybar" => serde_json::to_string(&waybar_output(label, percentage)).unwrap_or_default(),
        "json" => serde_json::to_string(&waybar_output(label, percentage)).unwrap_or_default(),
        "text" => format!("Mochi {percentage}% ({label})"),
        _ => format!("unsupported format: {format}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waybar_marks_critical_usage() {
        let output = waybar_output("Claude", 95);
        assert_eq!(output.class, "critical");
    }
}
