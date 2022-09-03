# Bevy ECSS

[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/afonsolage/bevy_ecss#license)
[![Rust](https://github.com/afonsolage/bevy_ecss/workflows/CI/badge.svg)](https://github.com/afonsolage/bevy_ecss/actions)

## What is Bevy ECSS?

Bevy ECSS is a crate which allows the usage of a subset of [`CSS`](https://developer.mozilla.org/en-US/docs/Web/CSS) to interact with [`bevy_ecs`](https://crates.io/crates/bevy_ecs). It's mainly aimed to apply styling on [`bevy_ui`](https://crates.io/crates/bevy) but it can be used by any component by implementing custom properties.

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

Type | Details | Example
:---: | :--- | :---
_Name_ | Selects by using `bevy` built-int [`Name`](https://docs.rs/bevy/latest/bevy/core/struct.Name.html) component. | `#inventory { ... }`
_Class_ | Selects by using `Class` component, which is provided by Bevy ECSS. | `.enabled { ... }`
_Component_ | Selects by using any component name, but it has to be registered before usage. You can find more details bellow | `button { ... }`

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

 Property | Values | Description
 :---: | :--- | :---
`display` | `flex`\|`none` | Applies the `display` property on `display` field of all sections on matched `Style` components.

## License

Bevy ECSS is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.