# [![Bevy](assets/branding/bevy_ecss.png)](https://bevyengine.org)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/afonsolage/bevy_ecss#license)
[![Realease Doc](https://docs.rs/bevy_ecss/badge.svg)](https://docs.rs/bevy_ecss)
[![Rust](https://github.com/afonsolage/bevy_ecss/workflows/CI/badge.svg)](https://github.com/afonsolage/bevy_ecss/actions)
[![Crate](https://img.shields.io/crates/v/bevy_ecss.svg)](https://crates.io/crates/bevy_ecss)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

# Bevy ECSS

## What is Bevy ECSS?

Bevy ECSS is a crate which allows the usage of a subset of [`CSS`](https://developer.mozilla.org/en-US/docs/Web/CSS) to interact with [`bevy_ecs`](https://crates.io/crates/bevy_ecs). It's mainly aimed to apply styling on [`bevy_ui`](https://crates.io/crates/bevy) but it can be used by any component by implementing custom properties.

### Why the name?

Just because Bevy ECS + CSS is a perfect fit!

## Usage

To use Bevy ECSS just add a `StyleSheet` with a loaded `css` file to any entity and all style sheet rules will be applied to the entity and _all_ its [`descendants`](https://stackoverflow.com/questions/1182189/css-child-vs-descendant-selectors) (children of children of children and so on).

```rust
use bevy::prelude::*;
use bevy_ecss::prelude::*;

fn setup_awesome_ui(root: Entity, mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .entity(root)
        .insert(StyleSheet::new(asset_server.load("sheets/awesome.css")));
}
```

That's it, now your UI will indeed look _awesome_!


## CSS Subset

Bevy ECSS only supports a subset of CSS at moment, since many properties and features requires more advanced selectors, components and properties which currently isn't implemented.

Here you can find a list of all currently supported selectors and properties:

### <ins>Selectors</ins>

|    Type     | Details                                                                                                       | Example              |
| :---------: | :------------------------------------------------------------------------------------------------------------ | :------------------- |
|   _Name_    | Selects by using `bevy` built-int [`Name`](https://docs.rs/bevy/latest/bevy/core/struct.Name.html) component. | `#inventory { ... }` |
|   _Class_   | Selects by using `Class` component, which is provided by Bevy ECSS.                                           | `.enabled { ... }`   |
| _Component_ | Selects by using any component, but it has to be registered before usage. You can find more details bellow.   | `button { ... }`     |

You may combine any of the above selector types to create a complex selector, if you like so. For instance, `window.enabled.pop-up` select all `window`s, which are `enabled` and are of `pop-up` type. The same rules of [`CSS Class selectors`](https://developer.mozilla.org/en-US/docs/Web/CSS/Class_selectors) applies here. 

_This assumes that `window` is a `bevy_ecs` component and was registered before usage. Also assumes the entities has the `Class` component with at least `enabled pop-up` class name._

Aditionally, Bevy ECSS also supports [`descendant combinator`](https://developer.mozilla.org/en-US/docs/Web/CSS/Descendant_combinator) which selects _all_ entities that are descendant the given selector tree.

```css
#quest-window text {
    color: red;
}
```

The above rule will match _all_ entities which has a [`Text`](https://docs.rs/bevy/latest/bevy/text/struct.Text.html) component and is descendant of any entity which as a [`Name`](https://docs.rs/bevy/latest/bevy/core/struct.Name.html) component which the value of `quest-window`.

So it's possible to combine complex composed selectors with descendant combinator.

```css
#main-menu button.enabled .border {
    background-color: #ff03ab;
}
```

This rule will match all components which has a `Class` with the value of `border` and are descendant of any entity which has a `button` component _and_ a `Class` component with the value of `enabled` and also are descendant of any entity which has a `Name` component with value `main-menu`.

### <ins>Properties</ins>

Here is a list of all currently supported properties. Note that these are properties which are provived by Bevy ECSS but you can also add your own properties at anytime.



_Before reading properties description, we'll use this notation to describe accepted values:_

|        Notation        | Description                                                                                                                                                                                                                            |
| :--------------------: | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|        `00.00%`        | Any percent value, like `93%` or `4.45%`                                                                                                                                                                                               |
|       `00.00px`        | Any dimensional value, like `11px` or `0.99px`                                                                                                                                                                                         |
|        `00.00`         | Any number value, like `0` or `14.2`                                                                                                                                                                                                   |
| `<ident>` \| `<ident>` | Only one of the identifiers are allowed, without quotes, like `none` or `hidden`                                                                                                                                                       |
|  <`area-short-hand`>   | Allows the [`short hand area constructor`](https://developer.mozilla.org/en-US/docs/Web/CSS/margin#syntax) by using either dimensions or percentage, like `10px` or `5% 10px 3% auto`. No global values are supported yet |


### <center>[`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) properties</center>

|       Property        |                                                                            Values                                                                             | Description                                                                                                                                                                                                                                                               |
|:---------------------:|:-------------------------------------------------------------------------------------------------------------------------------------------------------------:| :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
|       `display`       |                                                                       `flex` \| `none`                                                                        | Applies the  `display`         property on [`display`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.display) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.                 |
|    `position-type`    |                                                                   `absolute` \| `relative`                                                                    | Applies the  `position-type`   property on [`position_type`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.position_type) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.     |
|      `direction`      |                                                        `inherit` \| `left-to-right` \| `right-to-left`                                                        | Applies the  `direction`       property on [`direction`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.direction) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.             |
|   `flex-direction`    |                                                    `row` \| `column` \| `row-reverse` \| `column-reverse`                                                     | Applies the  `flex-direction`  property on [`flex_direction`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.flex_direction) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.   |
|      `flex-wrap`      |                                                             `no-wrap` \| `wrap` \| `wrap-reverse`                                                             | Applies the  `flex-wrap`       property on [`flex_wrap`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.flex_wrap) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.             |
|     `align-items`     |                                               `flex-start` \| `flex-end` \| `center` \| `baseline` \| `stretch`                                               | Applies the  `align-items`     property on [`align_items`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.align_items) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.         |
|     `align-self`      |                                          `auto` \| `flex-start` \| `flex-end` \| `center` \| `baseline` \| `stretch`                                          | Applies the  `align-self`      property on [`align_self`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.align_self) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.           |
|    `align-content`    |                                   `flex-start` \| `flex-end` \| `center` \| `stretch` \| `space-between` \| `space-around`                                    | Applies the  `align-content`   property on [`align_content`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.align_content) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.     |
|   `justify-content`   |                                 `flex-start` \| `flex-end` \| `center` \| `space-between` \| `space-around` \| `space-evenly`                                 | Applies the  `justify-content` property on [`justify_content`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.justify_content) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components. |
|     `overflow-x`      |                                                                     `visible` \| `hidden`                                                                     | Applies the  `overflow-x`      property on [`overflow.x`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.overflow) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.                |
|     `overflow-y`      |                                                                     `visible` \| `hidden`                                                                     | Applies the  `overflow-y`      property on [`overflow.y`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.overflow) field of all sections on matched [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html) components.                |
|        `left`         |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`left`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.left) field of all matched components.                                                                                                   |
|        `right`        |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`right`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.right) field of all matched components.                                                                                                  |
|         `top`         |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`top`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.top) field of all matched components.                                                                                                    |
|       `bottom`        |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`bottom`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.bottom) field of all matched components.                                                                                                 |
|        `width`        |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`width`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.width) field of all matched components.                                                                                                          |
|       `height`        |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`height`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.height) field of all matched components.                                                                                                         |
|      `min-width`      |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`min_width`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.min_width) field of all matched components.                                                                                                  |
|     `min-height`      |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`min_height`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.min_height) field of all matched components.                                                                                                 |
|      `max-width`      |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`max_width`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.max_width) field of all matched components.                                                                                                  |
|     `max-height`      |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`max_height`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.max_height) field of all matched components.                                                                                                 |
|     `flex-basis`      |                                                                     `00.00%` \| `00.00px`                                                                     | Applies the             property on [`flex_basis`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.flex_basis) field of all matched components.                                                                                                 |
|      `flex-grow`      |                                                                       `0` \| `1` \| `2`                                                                       | Applies the             property on [`flex_grow`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.flex_grow)   field of all matched components.                                                                                                    |
|     `flex-shrink`     |                                                                       `0` \| `1` \| `2`                                                                       | Applies the             property on [`flex_shrink`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.flex_shrink) field of all matched components.                                                                                                  |
|    `aspect-ratio`     |                                                                       `00.00` \| `none`                                                                       | Applies the             property on [`aspect_ratio`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.aspect_ratio) field of all matched components.                                                                                                |
|       `margin`        |                                                                      <`area-short-hand`>                                                                      | Applies the             property on [`margin`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.margin) field of all matched components.                                                                                                            |
|       `padding`       |                                                                      <`area-short-hand`>                                                                      | Applies the             property on [`padding`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.padding) field of all matched components.                                                                                                          |
|       `border`        |                                                                      <`area-short-hand`>                                                                      | Applies the             property on [`border`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html#structfield.border) field of all matched components.                                                                                                            |

### <center>[`Text`](https://docs.rs/bevy/latest/bevy/prelude/struct.Text.html) properties</center>

|     Property     |                                                                            Values                                                                            | Description                                                                                                                                                                                                                             |
| :--------------: | :----------------------------------------------------------------------------------------------------------------------------------------------------------: | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
|     `color`      | [`named-colors`](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) \| [`hex_colors`](https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color) | Applies the property on [`style.color`](https://docs.rs/bevy/latest/bevy/text/struct.TextSection.html#structfield.style) for all [`sections`](https://docs.rs/bevy/latest/bevy/text/struct.TextSection.html) of matched components.     |
|                  |
|      `font`      |                                                                     `"path/to/font.ttf"`                                                                     | Applies the property on [`style.font`](https://docs.rs/bevy/latest/bevy/text/struct.TextSection.html#structfield.style) for all [`sections`](https://docs.rs/bevy/latest/bevy/text/struct.TextSection.html) of matched components.      |
|                  |
|   `font-size`    |                                                                           `00.00`                                                                            | Applies the property on [`style.font_size`](https://docs.rs/bevy/latest/bevy/text/struct.TextSection.html#structfield.style) for all [`sections`](https://docs.rs/bevy/latest/bevy/text/struct.TextSection.html) of matched components. |
|                  |
|  `text-content`  |                                                                     `"Some text value"`                                                                      | Applies the property on [`value`](https://docs.rs/bevy/latest/bevy/text/struct.TextSection.html#structfield.value) for all [`sections`](https://docs.rs/bevy/latest/bevy/text/struct.TextSection.html) of matched components.           |
|                  |
|   `text-align`   |                                                                `left` \| `center` \| `right`                                                                 | Applies the property on [`alignment`](https://docs.rs/bevy/latest/bevy/text/struct.Text.html#structfield.alignment) of all matched components.                                                                                          |
|                  |

### <center>Components properties</center>

|       Property       |                                                                            Values                                                                            | Description                                                                                                                                  |
|:--------------------:| :----------------------------------------------------------------------------------------------------------------------------------------------------------: |:---------------------------------------------------------------------------------------------------------------------------------------------|
|  `background-color`  | [`named-colors`](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) \| [`hex_colors`](https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color) | Applies the property on [`BackgroundColor`](https://docs.rs/bevy/latest/bevy/prelude/struct.BackgroundColor.html) of all matched components. |
|    `border-color`    | [`named-colors`](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) \| [`hex_colors`](https://developer.mozilla.org/en-US/docs/Web/CSS/hex-color)  | Applies the property on [`BorderColor`](https://docs.rs/bevy/latest/bevy/prelude/struct.BorderColor.html) of all matched components.         |                                                                                                         |

### <center>Image properties</center>

|   Property   |       Values       | Description                                                                                                                                                                                                                          |
|:------------:|:------------------:|:-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `image-path` | "path/to/image.png" | Applies the property on [`image.texture`](https://docs.rs/bevy/latest/bevy/prelude/struct.UiImage.html#structfield.texture) for all [`images`](https://docs.rs/bevy/latest/bevy/ui/struct.UiImage.html) of matched components. |


## Component Selector Builtin

Bevy ECSS provites the following components selector:

|      Selector      |                                         Component                                         |
| :----------------: | :---------------------------------------------------------------------------------------: |
| `background-color` | [`BackgroundColor`](https://docs.rs/bevy/latest/bevy/prelude/struct.BackgroundColor.html) |
|       `text`       |             [`Text`](https://docs.rs/bevy/latest/bevy/text/struct.Text.html)              |
|      `button`      |          [`Button`](https://docs.rs/bevy/latest/bevy/prelude/struct.Button.html)          |
|       `node`       |            [`Node`](https://docs.rs/bevy/latest/bevy/prelude/struct.Node.html)            |
|      `style`       |           [`Style`](https://docs.rs/bevy/latest/bevy/prelude/struct.Style.html)           |
|     `ui-image`     |         [`UiImage`](https://docs.rs/bevy/latest/bevy/prelude/struct.UiImage.html)         |
|   `interaction`    |      [`Interaction`](https://docs.rs/bevy/latest/bevy/prelude/enum.Interaction.html)      |

This list will be expanded to match `bevy_ui` and other `bevy` core components.

## Custom Component Selector

You may also register your own components or alias/overwrite builtin components selector.
```rust
use bevy::prelude::*;
use bevy_ecss::prelude::*;

#[derive(Component)]
struct MyFancyComponentSelector;

#[derive(Component)]
struct FancyColor;

fn some_main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins).add_plugins(EcssPlugin::default());
    // You may use it as selector now, like
    // fancy-pants {
    //      background-color: pink;
    // }
    app.register_component_selector::<MyFancyComponentSelector>("fancy-pants");
    // Or you can overwrite a component selector.
    app.register_component_selector::<FancyColor>("background-color");
}
```

## Custom Property

It's also possible to implement your own properties, be it part of `CSS` standard or not.
Let's implement a custom `alpha` property with will set the alpha channel of any [`BackgroundColor`](https://docs.rs/bevy/latest/bevy/prelude/struct.BackgroundColor.html).
```rust
# use bevy::{ecs::query::QueryItem, prelude::*};
# use bevy_ecss::{prelude::*, EcssError, Property, PropertyValues};

#[derive(Default)]
pub(crate) struct AlphaProperty;

impl Property for AlphaProperty {
    // This is the cached value to be used when applying the property value.
    // It is evaluated only on the first time and futures runs are cached for performance reasons.
    type Cache = f32;
    // Which components the property needs when applying the cached value.
    // It is the same as using bevy_ecs Query<C, F>.
    type Components = &'static mut BackgroundColor;
    // If this property can be set only when there is another property, it's possible to filter here.
    // It's not recommended to use only With<> and Without<>.
    type Filters = ();

    fn name() -> &'static str {
        // The name of property. prefer kebab-case for consistency.
        "alpha"
    }

    fn parse<'a>(values: &PropertyValues) -> Result<Self::Cache, EcssError> {
        // PropertyValues::f32 tries to parse property value into a numeric value
        if let Some(value) = values.f32() {
            Ok(value)
        } else {
            Err(EcssError::InvalidPropertyValue(Self::name().to_string()))
        }
    }

    // This function will be called for every entity matched on every rule selector.
    fn apply<'w>(
        cache: &Self::Cache,
        mut components: QueryItem<Self::Components>,
        _asset_server: &AssetServer,
        _commands: &mut Commands,
    ) {
        components.0.set_a(*cache);
    }
}
```

Now just register the property on `App`:
```rust ignore
app.register_property::<AlphaProperty>();
```

Done! Whenever an `alpha` property is found on any `css` file, the `AlphaProperty` will be applied. You can find this full example [`here`](https://github.com/afonsolage/bevy_ecss/blob/main/examples/alpha.rs).


## Bevy support table
| bevy  | bevy_ecss |
| :---: | :-------: |
|  0.8  |    0.1    |
|  0.9  |    0.2    |
|  0.10 |    0.3    |
|  0.11 |    0.4    |
|  0.12 |    0.5    |


## Contributing

Got some idea, feedback, question or found any bug? Feel free to open an issue at any time!

## License

Bevy ECSS is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.
