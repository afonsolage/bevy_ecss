use bevy::prelude::Color;

pub(super) fn parse_hex_color(hex: &str) -> Option<Color> {
    if let Ok((r, g, b, a)) = cssparser::color::parse_hash_color(hex.as_bytes()) {
        Some(Color::rgba_u8(r, g, b, (a * 255.0) as u8))
    } else {
        None
    }
}

// Source: https://developer.mozilla.org/en-US/docs/Web/CSS/named-color

/// Parses a named color, like "silver" or "azure" into a [`Color`]
///
/// Accepts any [valid CSS named-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color).
pub(super) fn parse_named_color(name: &str) -> Option<Color> {
    if let Ok(cssparser_color::Color::Rgba(cssparser_color::RgbaLegacy {
        red,
        green,
        blue,
        alpha,
    })) = cssparser_color::parse_color_keyword(name)
    {
        Some(Color::rgba_u8(red, green, blue, (alpha * 255.0) as u8))
    } else {
        None
    }
}
