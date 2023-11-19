# Custom property

It is possible to implement your own properties, be it part of `CSS` standard or not.
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