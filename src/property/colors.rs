use bevy::prelude::Color;

pub(super) fn parse_hex_color(hex: &str) -> Option<Color> {
    if let Ok(cssparser::Color::RGBA(cssparser::RGBA {
        red,
        green,
        blue,
        alpha,
    })) = cssparser::Color::parse_hash(hex.as_bytes())
    {
        Some(Color::rgba_u8(red, green, blue, alpha))
    } else {
        None
    }
}

// Source: https://developer.mozilla.org/en-US/docs/Web/CSS/named-color

/// Parses a named color, like "silver" or "azure" into a [`Color`]
///
/// Accepts any [valid CSS named-colors](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color).
pub(super) fn parse_named_color(name: &str) -> Option<Color> {
    if let Ok(cssparser::Color::RGBA(cssparser::RGBA {
        red,
        green,
        blue,
        alpha,
    })) = cssparser::parse_color_keyword(name)
    {
        Some(Color::rgba_u8(red, green, blue, alpha))
    } else {
        None
    }
}
