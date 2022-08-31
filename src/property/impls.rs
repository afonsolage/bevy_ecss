use bevy::{
    ecs::query::{Fetch, WorldQueryGats},
    prelude::*,
};

use crate::parser::EcssError;

use super::{Property, PropertyValues};

pub(crate) use style::*;

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
                        Err(EcssError::InvalidPropertyValue($name.to_string()))
                    }
                }

                fn apply<'w>(
                    cache: &Self::Cache,
                    mut components: <<Self::Components as WorldQueryGats<'w>>::Fetch as Fetch<
                        'w,
                    >>::Item,
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
                        Err(EcssError::InvalidPropertyValue($name.to_string()))
                    }
                }

                fn apply<'w>(
                    cache: &Self::Cache,
                    mut components: <<Self::Components as WorldQueryGats<'w>>::Fetch as Fetch<
                        'w,
                    >>::Item,
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

                    Err(EcssError::InvalidPropertyValue($name.to_string()))
                }

                fn apply<'w>(
                    cache: &Self::Cache,
                    mut components: <<Self::Components as WorldQueryGats<'w>>::Fetch as Fetch<
                        'w,
                    >>::Item,
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
