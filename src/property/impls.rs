use bevy::{
    ecs::query::{Fetch, WorldQueryGats},
    prelude::*,
};

use crate::parser::EcssError;

use super::{Property, PropertyToken, PropertyValues};

pub(crate) use style::*;

mod style {
    use super::*;

    macro_rules! impl_style_enum {
        ($cache:ty, $name:expr, $struct:ident, $style_prop:ident, $($prop:expr => $variant:expr),+$(,)?) => {
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
                    if let Some(token) = values.identifiers().next() {
                        use $cache::*;
                        // Chain if-let when `cargofmt` supports it
                        // https://github.com/rust-lang/rustfmt/pull/5203
                        if let PropertyToken::Identifier(ident) = token {
                            match ident.as_str() {
                                $($prop => return Ok($variant)),+,
                                _ => (),
                            }
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
