use bevy::prelude::Color;

fn to_bevy_color(css_color: Option<cssparser::Color>) -> Option<Color> {
    // TODO: Implement other colors type
    if let Some(cssparser::Color::Rgba(cssparser::RgbaLegacy {
        red,
        green,
        blue,
        alpha,
    })) = css_color
    {
        Some(Color::rgba_u8(red, green, blue, (alpha * 255.0) as u8))
    } else {
        None
    }
}

pub(super) fn parse_hex_color(hex: &str) -> Option<Color> {
    to_bevy_color(cssparser::parse_hash_color(hex.as_bytes()).ok())
}

// Source: https://developer.mozilla.org/en-US/docs/Web/CSS/named-color

/// Parses a named color, like "silver" or "azure" into a [`Color`]
///
/// Accepts any [valid CSS named-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color).
pub(super) fn parse_named_color(name: &str) -> Option<Color> {
    to_bevy_color(cssparser::parse_color_keyword(name).ok())
}
