
# Custom Component Selector

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
