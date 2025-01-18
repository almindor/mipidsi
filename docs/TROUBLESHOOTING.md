# Troubleshooting guide

This guide lists common issues that can cause a blank or corrupted display.

## Display stays black/blank

### Reset pin

The reset pin on all supported display controllers is active low, requiring it to be driven **high** in order for the display to operate. It is recommended to connect the reset pin to a GPIO pin and let this crate control the pin by passing it to the builder via the `reset_pin` method. If this isn't possible in the target application the user must make sure that the reset pin on the display controller is kept in the high state before `init` is called.

### Backlight pin

This driver does **NOT** handle the backlight pin to keep the code simpler. Users must control the backlight manually. First thing to try is to see if setting the backlight pin to high fixes the issue.

### Transport misconfiguration (e.g. SPI)

Make sure that the transport layer is configured correctly. Typical mistakes are the use of wrong SPI MODE or too fast transfer speeds that are not supported by the display

## Incorrect colors

The way colors are displayed depend on the subpixel layout and technology (like TN or IPS) used by the LCD panel. These physical parameters aren't known by the display controller and must be manually set by the user as `Builder` settings when the display is initialized.

To make it easier to identify the correct settings the `mipidsi` crate provides a `TestImage`, which can be used to verify the color settings and adjust them in case they are incorrect.

```rust
let mut display = Builder::ili9486_rgb666(di)
    .init(&mut delay, Some(rst))?;

TestImage::new().draw(&mut display)?;
```

The expected output from drawing the test image is:

![Correct colors](colors_correct.svg)

If the test image isn't displayed as expected use one of the reference image below the determine which settings need to be added to the `Builder`.

### Wrong subpixel order

![Wrong subpixel order](colors_wrong_subpixel_order.svg)

```rust
.with_color_order(mipidsi::options::ColorOrder::Bgr)
```

### Wrong color inversion

![Wrong color inversion](colors_wrong_color_inversion.svg)

```rust
.with_invert_colors(mipidsi::options::ColorInversion::Inverted)
```

### Wrong subpixel order and color inversion

![Wrong subpixel order and color inversion](colors_both_wrong.svg)

```rust
.with_color_order(mipidsi::options::ColorOrder::Bgr)
.with_invert_colors(mipidsi::options::ColorInversion::Inverted)
```