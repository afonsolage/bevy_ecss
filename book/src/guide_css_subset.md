# CSS Subset

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