use bevy::{ecs::query::QueryItem, prelude::*};

use crate::EcssError;

use super::{Property, PropertyValues};

pub(crate) use style::*;
pub(crate) use text::*;

/// Impls for `bevy_ui` [`Style`] component
mod style {
    use super::*;
    /// Implements a new property for [`Style`] component which expects a rect value.
    macro_rules! impl_style_rect {
        ($name:expr, $struct:ident, $style_prop:ident$(.$style_field:ident)*) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            $(#[doc = concat!("::",stringify!($style_field))])*
            #[doc = "](`Style`) field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Cache = UiRect;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> &'static str {
                    $name
                }

                fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
                    if let Some(val) = values.rect() {
                        Ok(val)
                    } else {
                        Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
                    }
                }

                fn apply<'w>(
                    cache: &Self::Cache,
                    mut components: QueryItem<Self::Components>,
                    _asset_server: &AssetServer,
                    _commands: &mut Commands,
                ) {
                    components.$style_prop$(.$style_field)? = *cache;
                }
            }
        };
    }

    impl_style_rect!("margin", MarginProperty, margin);
    impl_style_rect!("padding", PaddingProperty, padding);
    impl_style_rect!("border", BorderProperty, border);

    /// Implements a new property for [`Style`] component which expects a single value.
    macro_rules! impl_style_single_value {
        ($name:expr, $struct:ident, $cache:ty, $parse_func:ident, $style_prop:ident$(.$style_field:ident)*) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            $(#[doc = concat!("::",stringify!($style_field))])*
            #[doc = "](`Style`) field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Cache = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> &'static str {
                    $name
                }

                fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
                    if let Some(val) = values.$parse_func() {
                        Ok(val)
                    } else {
                        Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
                    }
                }

                fn apply<'w>(
                    cache: &Self::Cache,
                    mut components: QueryItem<Self::Components>,
                    _asset_server: &AssetServer,
                    _commands: &mut Commands,
                ) {
                    components.$style_prop$(.$style_field)? = *cache;
                }
            }
        };
    }

    // Val properties
    impl_style_single_value!("left", LeftProperty, Val, val, left);
    impl_style_single_value!("right", RightProperty, Val, val, right);
    impl_style_single_value!("top", TopProperty, Val, val, top);
    impl_style_single_value!("bottom", BottomProperty, Val, val, bottom);

    impl_style_single_value!("width", WidthProperty, Val, val, width);
    impl_style_single_value!("height", HeightProperty, Val, val, height);

    impl_style_single_value!("min-width", MinWidthProperty, Val, val, width);
    impl_style_single_value!("min-height", MinHeightProperty, Val, val, height);

    impl_style_single_value!("max-width", MaxWidthProperty, Val, val, width);
    impl_style_single_value!("max-height", MaxHeightProperty, Val, val, height);

    impl_style_single_value!("flex-basis", FlexBasisProperty, Val, val, height);

    impl_style_single_value!("flex-grow", FlexGrowProperty, f32, f32, flex_grow);
    impl_style_single_value!("flex-shrink", FlexShrinkProperty, f32, f32, flex_shrink);

    impl_style_single_value!(
        "aspect-ratio",
        AspectRatioProperty,
        Option<f32>,
        option_f32,
        aspect_ratio
    );

    /// Implements a new property for [`Style`] component which expects an enum.
    macro_rules! impl_style_enum {
        ($cache:ty, $name:expr, $struct:ident, $style_prop:ident$(.$style_field:ident)*, $($prop:expr => $variant:expr),+$(,)?) => {
            #[doc = "Applies the `"]
            #[doc = $name]
            #[doc = "` property on [Style::"]
            #[doc = stringify!($style_prop)]
            #[doc = "]("]
            #[doc = concat!("`", stringify!($cache), "`")]
            #[doc = ") field of all sections on matched [`Style`] components."]
            #[derive(Default)]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Cache = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> &'static str {
                    $name
                }

                fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
                    if let Some(identifier) = values.identifier() {
                        use $cache::*;
                        // Chain if-let when `cargofmt` supports it
                        // https://github.com/rust-lang/rustfmt/pull/5203
                        match identifier {
                            $($prop => return Ok($variant)),+,
                            _ => (),
                        }
                    }

                    Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
                }

                fn apply<'w>(
                    cache: &Self::Cache,
                    mut components: QueryItem<Self::Components>,
                    _asset_server: &AssetServer,
                    _commands: &mut Commands,
                ) {
                    components.$style_prop$(.$style_field)? = *cache;
                }
            }
        };
    }

    impl_style_enum!(Display, "display", DisplayProperty, display,
        "flex" => Flex,
        "none" => None
    );

    impl_style_enum!(PositionType, "position-type", PositionTypeProperty, position_type,
        "absolute" => Absolute,
        "relative" => Relative,
    );

    impl_style_enum!(Direction, "direction", DirectionProperty, direction,
        "inherit" => Inherit,
        "left-to-right" => LeftToRight,
        "right-to-left" => RightToLeft,
    );

    impl_style_enum!(FlexDirection, "flex-direction", FlexDirectionProperty, flex_direction,
        "row" => Row,
        "column" => Column,
        "row-reverse" => RowReverse,
        "column-reverse" => ColumnReverse,
    );

    impl_style_enum!(FlexWrap, "flex-wrap", FlexWrapProperty, flex_wrap,
        "no-wrap" => NoWrap,
        "wrap" => Wrap,
        "wrap-reverse" => WrapReverse,
    );

    impl_style_enum!(AlignItems, "align-items", AlignItemsProperty, align_items,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "baseline" => Baseline,
        "stretch" => Stretch,
    );

    impl_style_enum!(AlignSelf, "align-self", AlignSelfProperty, align_self,
        "auto" => Auto,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "baseline" => Baseline,
        "stretch" => Stretch,
    );

    impl_style_enum!(AlignContent, "align-content", AlignContentProperty, align_content,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "stretch" => Stretch,
        "space-between" => SpaceBetween,
        "space-around" => SpaceAround,
    );

    impl_style_enum!(JustifyContent, "justify-content", JustifyContentProperty, justify_content,
        "flex-start" => FlexStart,
        "flex-end" => FlexEnd,
        "center" => Center,
        "space-between" => SpaceBetween,
        "space-around" => SpaceAround,
        "space-evenly" => SpaceEvenly,
    );

    impl_style_enum!(OverflowAxis, "overflow-x", OverflowAxisXProperty, overflow.x,
        "visible" => Visible,
        "hidden" => Clip,
    );

    impl_style_enum!(OverflowAxis, "overflow-y", OverflowAxisYProperty, overflow.y,
        "visible" => Visible,
        "hidden" => Clip,
    );
}

/// Impls for `bevy_text` [`Text`] component
mod text {
    use super::*;

    /// Applies the `color` property on [`TextStyle::color`](`TextStyle`) field of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub(crate) struct FontColorProperty;

    impl Property for FontColorProperty {
        type Cache = Color;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "color"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(color) = values.color() {
                Ok(color)
            } else {
                Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
            }
        }

        fn apply<'w>(
            cache: &Self::Cache,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            components
                .sections
                .iter_mut()
                .for_each(|section| section.style.color = *cache);
        }
    }

    /// Applies the `font` property on [`TextStyle::font`](`TextStyle`) property of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub(crate) struct FontProperty;

    impl Property for FontProperty {
        type Cache = String;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "font"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(path) = values.string() {
                Ok(path)
            } else {
                Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
            }
        }

        fn apply<'w>(
            cache: &Self::Cache,
            mut components: QueryItem<Self::Components>,
            asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            components
                .sections
                .iter_mut()
                .for_each(|section| section.style.font = asset_server.load(cache));
        }
    }

    /// Applies the `font-size` property on [`TextStyle::font_size`](`TextStyle`) property of all sections on matched [`Text`] components.
    #[derive(Default)]
    pub(crate) struct FontSizeProperty;

    impl Property for FontSizeProperty {
        type Cache = f32;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "font-size"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(size) = values.f32() {
                Ok(size)
            } else {
                Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
            }
        }

        fn apply<'w>(
            cache: &Self::Cache,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            components
                .sections
                .iter_mut()
                .for_each(|section| section.style.font_size = *cache);
        }
    }

    /// Applies the `text-align` property on [`Text::horizontal`](`TextAlignment`) components.
    #[derive(Default)]
    pub(crate) struct TextAlignProperty;

    impl Property for TextAlignProperty {
        // Using Option since Cache must impl Default, which  doesn't
        type Cache = Option<TextAlignment>;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "text-align"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(ident) = values.identifier() {
                match ident {
                    "left" => return Ok(Some(TextAlignment::Left)),
                    "center" => return Ok(Some(TextAlignment::Center)),
                    "right" => return Ok(Some(TextAlignment::Right)),
                    _ => (),
                }
            }
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }

        fn apply<'w>(
            cache: &Self::Cache,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            components.alignment = cache.expect("Should always have a inner value");
        }
    }

    /// Apply a custom `text-content` which updates [`TextSection::value`](`TextSection`) of all sections on matched [`Text`] components
    #[derive(Default)]
    pub(crate) struct TextContentProperty;

    impl Property for TextContentProperty {
        type Cache = String;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "text-content"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(content) = values.string() {
                Ok(content)
            } else {
                Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
            }
        }

        fn apply<'w>(
            cache: &Self::Cache,
            mut components: QueryItem<Self::Components>,
            _asset_server: &AssetServer,
            _commands: &mut Commands,
        ) {
            components
                .sections
                .iter_mut()
                // TODO: Maybe change this so each line break is a new section
                .for_each(|section| section.value = cache.clone());
        }
    }
}

/// Applies the `background-color` property on [`BackgroundColor`] component of matched entities.
#[derive(Default)]
pub(crate) struct BackgroundColorProperty;

impl Property for BackgroundColorProperty {
    type Cache = Color;
    type Components = Entity;
    type Filters = With<BackgroundColor>;

    fn name() -> &'static str {
        "background-color"
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        if let Some(color) = values.color() {
            Ok(color)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: &Self::Cache,
        components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        commands: &mut Commands,
    ) {
        commands.entity(components).insert(BackgroundColor(*cache));
    }
}

/// Applies the `border-color` property on [`BorderColor`] component of matched entities.
#[derive(Default)]
pub struct BorderColorProperty;

impl Property for BorderColorProperty {
    type Cache = Color;
    type Components = Entity;
    type Filters = With<BorderColor>;

    fn name() -> &'static str {
        "border-color"
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        if let Some(color) = values.color() {
            Ok(color)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: &Self::Cache,
        components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        commands: &mut Commands,
    ) {
        commands.entity(components).insert(BorderColor(*cache));
    }
}

/// Applies the `image-path` property on [`bevy::ui::UiImage`] texture property of all sections on matched [`bevy::ui::UiImage`] components.
#[derive(Default)]
pub struct ImageProperty;

impl Property for ImageProperty {
    type Cache = String;
    type Components = &'static mut UiImage;
    type Filters = With<Node>;

    fn name() -> &'static str {
        "image-path"
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        if let Some(path) = values.string() {
            Ok(path)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    fn apply<'w>(
        cache: &Self::Cache,
        mut components: QueryItem<Self::Components>,
        asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        components.texture = asset_server.load(cache);
    }
}
