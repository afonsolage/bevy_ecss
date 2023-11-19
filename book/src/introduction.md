# Introduction

<img class="right" src="https://github.com/afonsolage/bevy_ecss/raw/main/assets/branding/bevy_ecss.png" alt="The Bevy ECSS logo">

Bevy ECSS is a crate which allows the usage of a subset of [`CSS`](https://developer.mozilla.org/en-US/docs/Web/CSS) to interact with [`bevy_ecs`](https://crates.io/crates/bevy_ecs). It's mainly aimed to apply styling on [`bevy_ui`](https://crates.io/crates/bevy) but it can be used by any component by implementing custom properties.

## Why the name?

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
