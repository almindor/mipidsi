# mipidsi

This crate provides a generic display driver to connect to TFT displays
that implement the [MIPI DCS](https://www.mipi.org/specifications/display-command-set).

Uses [display_interface](https://crates.io/crates/display-interface) to talk to the hardware via transports (currently SPI, I2C and Parallel GPIO).

An optional batching of draws is supported via the `batch` feature (default on)

*NOTES*: 

* The name of this crate is a bit unfortunate as this driver works with displays that use the MIPI DCS via any transport supported by [display_interface](https://crates.io/crates/display-interface) but MIPI DSI is NOT supported at this time.

* This driver does **NOT** handle the backlight pin to keep the code simpler. Users must control the backlight manually.

## Architecture

The `Display` driver itself contains most of the functionality. Each specific display model implements the `Model` trait for every color format it supports. Each model can also have different variants which are handled via the `Builder` struct.

[embedded-graphics-core](https://crates.io/crates/embedded-graphics-core) is used to provide the drawing API.

## Models

Each supported display model can be used either through the `Builder::with_model` call or through a shortcut function such as `Builder::st7789` if provided. External crates can be used to provide additional models, and can even expand the display constructor pool via trait extension.

Variants that require different screen sizes and window addressing offsets are now supported via the `Builder` logic as well (see docs).

### List of supported models

* ST7789
* ST7735
* ILI9486
* ILI9341
* ILI9342C

## Migration

See [MIGRATION.md](MIGRATION.md) document.

### Example
```rust
// create a DisplayInterface from SPI and DC pin, with no manual CS control
let di = SPIInterfaceNoCS::new(spi, dc);
// create the ILI9486 display driver in rgb666 color mode from the display interface and use a HW reset pin during init
let mut display = Builder::ili9486_rgb666(di)
    .init(&mut delay, Some(rst))?; // delay provider from your MCU
// clear the display to black
display.clear(Rgb666::BLACK)?;
```

License: MIT

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.61.0 and up. It *might*
compile with older versions but that may change in any new patch release.
