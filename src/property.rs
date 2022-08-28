use bevy::{
    text::{HorizontalAlign, VerticalAlign},
    ui::{
        AlignContent, AlignItems, AlignSelf, Direction, Display, FlexDirection, FlexWrap,
        JustifyContent, Overflow, PositionType, UiRect, Val,
    },
};

use cssparser::{ToCss, Token};
use heck::ToKebabCase;
use smallvec::{SmallVec, smallvec};

use crate::{colors, parser::EcssError};

#[derive(Debug, Clone)]
pub enum Property {
    Display(Display),
    PositionType(PositionType),
    Direction(Direction),
    FlexDirection(FlexDirection),
    FlexWrap(FlexWrap),
    AlignItems(AlignItems),
    AlignSelf(AlignSelf),
    AlignContent(AlignContent),
    JustifyContent(JustifyContent),
    PositionLeft(Val),
    PositionRight(Val),
    PositionTop(Val),
    PositionBottom(Val),
    Margin(UiRect<Val>),
    Padding(UiRect<Val>),
    Border(UiRect<Val>),
    FlexGrow(f32),
    FlexShrink(f32),
    FlexBasis(Val),
    SizeWidth(Val),
    SizeHeight(Val),
    SizeMinWidth(Val),
    SizeMinHeight(Val),
    SizeMaxWidth(Val),
    SizeMaxHeight(Val),

    AspectRatio(Option<f32>),
    Overflow(Overflow),

    TextVerticalAlign(VerticalAlign),
    TextHorizontalAlign(HorizontalAlign),
    Font(String),
    FontSize(f32),
    FontColor([f32; 4]),
    Color([f32; 4]),
}

impl Property {
    pub fn new(name: &str, values: PropertyValue) -> Result<Property, EcssError> {
        let property = match name {
            "display" => Property::Display(values.try_into()?),
            // Using CSS Property name "position" instead of "position-type"
            "position" => Property::PositionType(values.try_into()?),
            "direction" => Property::Direction(values.try_into()?),
            "flex-direction" => Property::FlexDirection(values.try_into()?),
            "flex-wrap" => Property::FlexWrap(values.try_into()?),
            "align-items" => Property::AlignItems(values.try_into()?),
            "align-self" => Property::AlignSelf(values.try_into()?),
            "align-content" => Property::AlignContent(values.try_into()?),
            "justify-content" => Property::JustifyContent(values.try_into()?),

            "left" => Property::PositionLeft(values.try_into()?),
            "right" => Property::PositionRight(values.try_into()?),
            "top" => Property::PositionTop(values.try_into()?),
            "bottom" => Property::PositionBottom(values.try_into()?),

            "width" => Property::SizeWidth(values.try_into()?),
            "height" => Property::SizeHeight(values.try_into()?),
            "min-width" => Property::SizeMinWidth(values.try_into()?),
            "min-height" => Property::SizeMinHeight(values.try_into()?),
            "max-width" => Property::SizeMaxWidth(values.try_into()?),
            "max-height" => Property::SizeMaxHeight(values.try_into()?),
            "margin" => Property::Margin(values.try_into()?),
            "padding" => Property::Padding(values.try_into()?),
            "border" => Property::Border(values.try_into()?),
            "flex-grow" => Property::FlexGrow(values.try_into()?),
            "flex-shrink" => Property::FlexShrink(values.try_into()?),
            "flex-basis" => Property::FlexBasis(values.try_into()?),
            "aspect-ratio" => Property::AspectRatio(values.try_into()?),
            "overflow" => Property::Overflow(values.try_into()?),
            "font" => Property::Font(values.try_into()?),
            "font-size" => Property::FontSize(values.try_into()?),

            // Using CSS Property name "color" instead of "font-color"
            "color" => Property::FontColor(values.try_into()?),
            // Using CSS Property name "background_color" instead of "color"
            "background-color" => Property::Color(values.try_into()?),

            // Using CSS Property name "vertical-align" instead of "text-vertical-align"
            "vertical-align" => Property::TextVerticalAlign(values.try_into()?),
            // Using CSS Property name "text-align" instead of "text-horizontal-align"
            "text-align" => Property::TextHorizontalAlign(values.try_into()?),
            _ => return Err(EcssError::UnsupportedProperty(name.to_string())),
        };

        Ok(property)
    }
}

#[derive(Debug, Clone)]
pub struct PropertyValue<'i>(SmallVec<[Token<'i>; 8]>);

impl<'i> PropertyValue<'i> {
    fn only_ident(self) -> Self {
        Self(
            self.0
                .into_iter()
                .filter(|t| matches!(t, Token::Ident(_)))
                .collect(),
        )
    }

    fn only_dim_or_perc(self) -> Self {
        Self(
            self.0
                .into_iter()
                .filter(|t| matches!(t, Token::Dimension { .. } | Token::Percentage { .. }))
                .collect(),
        )
    }
}

impl<'i> std::ops::Deref for PropertyValue<'i> {
    type Target = SmallVec<[Token<'i>; 8]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'i> From<SmallVec<[Token<'i>; 8]>> for PropertyValue<'i> {
    fn from(v: SmallVec<[Token<'i>; 8]>) -> Self {
        Self(v)
    }
}

fn token_to_val<'i>(token: Token<'i>) -> Val {
    match token {
        Token::Percentage { unit_value, .. } => Val::Percent(unit_value * 100.0),
        Token::Dimension { value, .. } => Val::Px(value),
        _ => Val::Undefined,
    }
}

fn token_to_f32<'i>(token: Token<'i>) -> f32 {
    match token {
        Token::Percentage { unit_value, .. } => unit_value * 100.0,
        Token::Dimension { value, .. } => value,
        Token::Number { value, .. } => value,
        _ => 0.0,
    }
}

fn token_to_option<'i, T: TryFrom<PropertyValue<'i>>>(token: Token<'i>) -> Option<T> {
    match token {
        Token::Ident(_) => None,
        _ => T::try_from(PropertyValue(smallvec![token])).ok(),
    }
}

fn token_to_color<'i>(token: Token<'i>) -> [f32; 4] {
    match token {
        Token::IDHash(ref hash) | Token::Hash(ref hash) => {
            if let Ok(color) = cssparser::Color::parse_hash(hash.as_bytes()) && let cssparser::Color::RGBA(rgba) = color {
                [rgba.red_f32(), rgba.green_f32(), rgba.blue_f32(), rgba.alpha_f32()]
            } else {
                [1.0; 4]
            }
        }
        Token::Ident(name) => colors::parse_named_color(&name),
        _ => [1.0; 4],
    }
}

impl<'i> TryFrom<PropertyValue<'i>> for UiRect<Val> {
    type Error = EcssError;

    fn try_from(value: PropertyValue<'i>) -> Result<Self, Self::Error> {
        let mut value = value.only_dim_or_perc();
        if value.is_empty() {
            return Err(EcssError::InvalidPropertyValue(format!("{:?}", value)));
        }

        let mut result = UiRect::all(token_to_val(value.0.remove(0)));

        if value.is_empty() == false {
            result.right = token_to_val(value.0.remove(0));
        }

        if value.is_empty() == false {
            result.bottom = token_to_val(value.0.remove(0));
        }

        if value.is_empty() == false {
            result.left = token_to_val(value.0.remove(0));
        }

        Ok(result)
    }
}

impl<'i> TryFrom<PropertyValue<'i>> for Val {
    type Error = EcssError;

    fn try_from(value: PropertyValue<'i>) -> Result<Self, Self::Error> {
        let mut value = value.only_dim_or_perc();
        if value.is_empty() {
            return Err(EcssError::InvalidPropertyValue(format!("{:?}", value)));
        }

        Ok(token_to_val(value.0.remove(0)))
    }
}

impl<'i> TryFrom<PropertyValue<'i>> for f32 {
    type Error = EcssError;

    fn try_from(value: PropertyValue<'i>) -> Result<Self, Self::Error> {
        match value
            .0
            .into_iter()
            .filter(|t| matches!(t, Token::Number { .. }))
            .next()
        {
            Some(t) => Ok(token_to_f32(t)),
            None => Err(EcssError::InvalidPropertyValue("number".to_string())),
        }
    }
}

impl<'i> TryFrom<PropertyValue<'i>> for String {
    type Error = EcssError;

    fn try_from(value: PropertyValue<'i>) -> Result<Self, Self::Error> {
        match value
            .0
            .into_iter()
            .filter(|t| matches!(t, Token::QuotedString(_)))
            .next()
        {
            Some(t) => Ok(t.to_css_string()),
            None => Err(EcssError::InvalidPropertyValue("string".to_string())),
        }
    }
}

impl<'i> TryFrom<PropertyValue<'i>> for Option<f32> {
    type Error = EcssError;

    fn try_from(value: PropertyValue<'i>) -> Result<Self, Self::Error> {
        match value
            .0
            .into_iter()
            .filter(|t| matches!(t, Token::Number { .. } | Token::Ident(_)))
            .next()
        {
            Some(t) => Ok(token_to_option(t)),
            None => Err(EcssError::InvalidPropertyValue("string".to_string())),
        }
    }
}

impl<'i> TryFrom<PropertyValue<'i>> for [f32; 4] {
    type Error = EcssError;

    fn try_from(value: PropertyValue<'i>) -> Result<Self, Self::Error> {
        match value
            .0
            .into_iter()
            .filter(|t| matches!(t, Token::Ident(_) | Token::Hash(_) | Token::IDHash(_)))
            .next()
        {
            Some(token) => Ok(token_to_color(token)),
            None => Err(EcssError::InvalidPropertyValue("color".to_string())),
        }
    }
}

macro_rules! try_from_enum {
    ($t:ty, $($name:expr => $variant:expr),+$(,)?) => {
        impl<'i> TryFrom<PropertyValue<'i>> for $t {
            type Error = EcssError;

            fn try_from(value: PropertyValue<'i>) -> Result<Self, EcssError> {
                use $t::*;

                let value = value.only_ident();
                if value.is_empty() == false && let Token::Ident(ref v) = value[0] {
                    match v.as_ref() {
                        $($name => return Ok($variant)),+,
                        _ => (),
                    }
                }

                Err(EcssError::InvalidPropertyValue(format!(
                    "{}",
                    stringify!($t).to_kebab_case()
                )))
            }
        }
    };
}

try_from_enum!(Display,
    "flex" => Flex,
    "none" => None,
);

try_from_enum!(PositionType,
    "absolute" => Absolute,
    "relative" => Relative,
);

try_from_enum!(Direction,
    "inherit" => Inherit,
    "left-to-right" => LeftToRight,
    "right-to-left" => RightToLeft,
);

try_from_enum!(FlexDirection,
    "row" => Row,
    "column" => Column,
    "row-reverse" => RowReverse,
    "column-reverse" => ColumnReverse,
);

try_from_enum!(FlexWrap,
    "no-wrap" => NoWrap,
    "wrap" => Wrap,
    "wrap-reverse" => WrapReverse,
);

try_from_enum!(AlignItems,
    "flex-start" => FlexStart,
    "flex-end" => FlexEnd,
    "center" => Center,
    "baseline" => Baseline,
    "stretch" => Stretch,
);

try_from_enum!(AlignSelf,
    "auto" => Auto,
    "flex-start" => FlexStart,
    "flex-end" => FlexEnd,
    "center" => Center,
    "baseline" => Baseline,
    "stretch" => Stretch,
);

try_from_enum!(AlignContent,
    "flex-start" => FlexStart,
    "flex-end" => FlexEnd,
    "center" => Center,
    "stretch" => Stretch,
    "space-between" => SpaceBetween,
    "space-around" => SpaceAround,
);

try_from_enum!(JustifyContent,
    "flex-start" => FlexStart,
    "flex-end" => FlexEnd,
    "center" => Center,
    "space-between" => SpaceBetween,
    "space-around" => SpaceAround,
    "space-evenly" => SpaceEvenly,
);

try_from_enum!(Overflow,
    "visible" => Visible,
    "hidden" => Hidden,
);

try_from_enum!(VerticalAlign,
    "top" => Top,
    "center" => Center,
    "bottom" => Bottom,
);

try_from_enum!(HorizontalAlign,
    "left" => Left,
    "center" => Center,
    "right" => Right,
);
