use bevy::{
    ecs::query::{Fetch, WorldQueryGats},
    prelude::*,
};

use crate::parser::EcssError;

use super::{Property, PropertyValues};

pub(crate) use style::*;

mod style {
    use super::*;

    macro_rules! impl_style_single_val {
        ($name:expr, $struct:ident, $style_prop:ident$(.$style_field:ident)*) => {
            #[derive(Default)]
            /// [`Property`](crate::Property) implementation for [`Style`]
            pub(crate) struct $struct;

            impl Property for $struct {
                type Cache = Val;
                type Components = &'static mut Style;
                type Filters = With<Node>;

                fn name() -> &'static str {
                    $name
                }

                fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
                    if let Some(val) = values.single_val() {
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

    impl_style_single_val!("left", LeftProperty, position.left);
    impl_style_single_val!("right", RightProperty, position.right);
    impl_style_single_val!("top", TopProperty, position.top);
    impl_style_single_val!("bottom", BottomProperty, position.bottom);

    impl_style_single_val!("width", WidthProperty, size.width);
    impl_style_single_val!("height", HeightProperty, size.height);

    impl_style_single_val!("min-width", MinWidthProperty, min_size.width);
    impl_style_single_val!("min-height", MinHeightProperty, min_size.height);

    impl_style_single_val!("max-width", MaxWidthProperty, max_size.width);
    impl_style_single_val!("max-height", MaxHeightProperty, max_size.height);


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
