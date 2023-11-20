# Theme example

<canvas id="bevy"></canvas>
<script type="module">
    // Import and run your bevy wasm code
    import init from './theme.js'
    init();
</script>

## CSS

### Dark theme

```css
{{#include ../../assets/sheets/dark_theme.css}}
```

### Light theme

```css
{{#include ../../assets/sheets/light_theme.css}}
```

## Code

```rust
{{#include ../../examples/theme.rs}}
```