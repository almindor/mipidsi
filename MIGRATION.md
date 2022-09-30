# Migration guide for MIPIDSI driver

## v0.3 -> v0.4

### Users

* `Display` helper constructors now take `DisplayOptions` argument instead of `init`

#### v0.3

```rust
let display = Display::st7789(di, rst);
display.init(&mut delay, DisplayOptions::default());
```

#### v0.4 

```rust
let display = Display::st7789(di, rst, DisplayOptions::default());
display.init(&mut delay);
```

### Model writers and specific variants

`Model` now requires that the `Model::new` constructor takes `ModelOptions` which is an artefact of `DisplayOptions` in combination with display sizing and windowing settings. This is used by the "helper constructors" to provide pre-made `Model` variants to users.

Most display `Model`s require just one constructor but some like the `ST7789` have a lot of variants that have different display, frameber sizes or even windowing clipping offsets. These can now be provided easily by creating a helper constructor that provides the dimensions information to create `ModelOptions`.

For users that need to use a variant of a `Model` that does not yet have a constructor helper, this can be done manually provided you know what your display dimensions and offsets are.
