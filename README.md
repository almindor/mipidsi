# mipidsi

This crate provides a generic display driver to connect to TFT displays
that implement the [MIPI DSI](https://www.mipi.org/specifications/dsi).

Uses [display_interface](https://crates.io/crates/display-interface) to talk to the hardware.

An optional batching of draws is supported via the `batch` feature (default on)

## Architecture

The `Display` driver itself contains most of the functionality. Each specific display model implements the `Model` trait for every color format it supports. Each model can also have different variants which are handled via the `Builder` struct.

[embedded-graphics-core](https://crates.io/crates/embedded-graphics-core) is used to provide the drawing API.

## Models

Each supported display model can be used either through the `Builder::with_model` call or through a shortcut function such as `Builder::st7789` if provided. External crates can be used to provide additional models, and can even expand the display constructor pool via trait extension.

Variants that require different screen sizes and window addressing offsets are now supported via the `Builder` logic as well (see docs).

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

This crate is guaranteed to compile on stable Rust 1.59.0 and up. It *might*
compile with older versions but that may change in any new patch release.
