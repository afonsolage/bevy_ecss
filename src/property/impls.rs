use bevy::{ecs::query::QueryItem, prelude::*};

use crate::parser::EcssError;

use super::{Property, PropertyValues};

pub(crate) use style::*;
pub(crate) use text::*;

mod style {
    use super::*;

    macro_rules! impl_style_rect {
        ($name:expr, $struct:ident, $style_prop:ident$(.$style_field:ident)*) => {
            #[derive(Default)]
            /// [`Property`](crate::Property) implementation for [`Style`]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Cache = UiRect<Val>;
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

    macro_rules! impl_style_single_value {
        ($name:expr, $struct:ident, $cache:ty, $parse_func:ident, $style_prop:ident$(.$style_field:ident)*) => {
            #[derive(Default)]
            /// [`Property`](crate::Property) implementation for [`Style`]
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
    impl_style_single_value!("left", LeftProperty, Val, single_val, position.left);
    impl_style_single_value!("right", RightProperty, Val, single_val, position.right);
    impl_style_single_value!("top", TopProperty, Val, single_val, position.top);
    impl_style_single_value!("bottom", BottomProperty, Val, single_val, position.bottom);

    impl_style_single_value!("width", WidthProperty, Val, single_val, size.width);
    impl_style_single_value!("height", HeightProperty, Val, single_val, size.height);

    impl_style_single_value!(
        "min-width",
        MinWidthProperty,
        Val,
        single_val,
        min_size.width
    );
    impl_style_single_value!(
        "min-height",
        MinHeightProperty,
        Val,
        single_val,
        min_size.height
    );

    impl_style_single_value!(
        "max-width",
        MaxWidthProperty,
        Val,
        single_val,
        max_size.width
    );
    impl_style_single_value!(
        "max-height",
        MaxHeightProperty,
        Val,
        single_val,
        max_size.height
    );

    impl_style_single_value!(
        "flex-basis",
        FlexBasisProperty,
        Val,
        single_val,
        max_size.height
    );

    impl_style_single_value!("flex-grow", FlexGrowProperty, f32, single_f32, flex_grow);
    impl_style_single_value!(
        "flex-shrink",
        FlexShrinkProperty,
        f32,
        single_f32,
        flex_shrink
    );

    impl_style_single_value!(
        "aspect-ratio",
        AspectRatioProperty,
        Option<f32>,
        option_f32,
        aspect_ratio
    );

    macro_rules! impl_style_enum {
        ($cache:ty, $name:expr, $struct:ident, $style_prop:ident, $($prop:expr => $variant:expr),+$(,)?) => {
            #[derive(Default)]
            /// [`Property`](crate::Property) implementation for [`Style`]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Cache = $cache;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> &'static str {
                    $name
                }

                fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
                    if let Some(identifier) = values.single_identifier() {
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
                    components.$style_prop = *cache;
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

    impl_style_enum!(Overflow, "direction", OverflowProperty, overflow,
        "visible" => Visible,
        "hidden" => Hidden,
    );
}

mod text {
    use super::*;

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

    #[derive(Default)]
    pub(crate) struct FontSizeProperty;

    impl Property for FontSizeProperty {
        type Cache = f32;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "font"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(size) = values.single_f32() {
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

    #[derive(Default)]
    pub(crate) struct VerticalAlignProperty;

    impl Property for VerticalAlignProperty {
        type Cache = Option<VerticalAlign>;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "vertical-align"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(ident) = values.single_identifier() {
                match ident {
                    "top" => return Ok(Some(VerticalAlign::Top)),
                    "center" => return Ok(Some(VerticalAlign::Center)),
                    "bottom" => return Ok(Some(VerticalAlign::Bottom)),
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
            components.alignment.vertical = cache.unwrap();
        }
    }

    #[derive(Default)]
    pub(crate) struct HorizontalAlignProperty;

    impl Property for HorizontalAlignProperty {
        type Cache = Option<HorizontalAlign>;
        type Components = &'static mut Text;
        type Filters = With<Node>;

        fn name() -> &'static str {
            "text-align"
        }

        fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
            if let Some(ident) = values.single_identifier() {
                match ident {
                    "left" => return Ok(Some(HorizontalAlign::Left)),
                    "center" => return Ok(Some(HorizontalAlign::Center)),
                    "right" => return Ok(Some(HorizontalAlign::Right)),
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
            components.alignment.horizontal = cache.unwrap();
        }
    }
}

#[derive(Default)]
pub(crate) struct UiColorProperty;

impl Property for UiColorProperty {
    type Cache = Color;
    type Components = Entity;
    type Filters = With<UiColor>;

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
        commands.entity(components).insert(UiColor(*cache));
    }
}
