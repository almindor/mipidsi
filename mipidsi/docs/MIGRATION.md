# Migration guide for MIPIDSI driver

## v0.6 -> 0.7

No breaking changes.

## v0.5 -> 0.6

### Users

* no change, addition of `Builder::with_invert_colors(bool)` allows more combinations of display variants.

### Model writers and specific variants

`Model::init` now expects the `options: &ModelOptions` instead of just `madcl: u8` argument. This allows the use of the `invert_colors` field during init.

## v0.4 -> 0.5

### Users

* use `Builder` to construct the `Display` and set any options directly on construction

#### v0.4

```rust
let display = Display::st7789(di, rst, DisplayOptions::default());
display.init(&mut delay);
```

#### v0.5

```rust
let display = Builder::st7789(di) // known model or with_model(model)
    .with_display_size(240, 240) // set any options on the builder before init
    .init(&mut delay, Some(rst)); // optional reset pin
```

### Model writers and specific variants

`Model::new` was reverted and is no longer necessary. Models now don't own the `ModelOptions` which has been moved off to the `Display` directly. `Model::init` has changed to include `madctl` parameter which is now provided by the `Display` and should be used as-is unless overrides are required.
`Model::default_options` was added to facilitate "generic variant" construction.

Helper constructors have been moved from `Display` to `Builder` with similar implementations as before.
`DisplayOptions` and `ModelOptions` values are now a function of the `Builder` and do not necessitate a constructor helper anymore. e.g. `Display::st7789_240x240(...)` becomes `Builder::st7789(...).with_display_size(240, 240)` controlled by the user.
Any variants can still set all of the options for a variant via a builder shortcut, such as `Builder::st7789_pico1`.

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