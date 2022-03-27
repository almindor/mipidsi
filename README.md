# mipidsi

This crate provides a generic display driver to connect to TFT displays
that implement the [MIPI DSI](https://www.mipi.org/specifications/dsi).

Uses [display_interface](https://crates.io/crates/display-interface) to talk to the hardware.

An optional batching of draws is supported via the `batch` feature (default on)

## Architecture

The `Display` driver itself contains most of the functionality. Each specific display model implements the `Model` trait for every color format it supports.

[embedded-graphics-core](https://crates.io/crates/embedded-graphics-core) is used to provide the drawing API.

### Example
```rust
// create a DisplayInterface from SPI and DC pin, with no manual CS control
let di = SPIInterfaceNoCS::new(spi, dc);
// create the ILI9486 display driver in rgb666 color mode from the display interface and RST pin
let mut display = Display::ili9486_rgb666(di, rst);
display.init(&mut delay::Ets)?;
// clear the display to black
display.clear(Rgb666::BLACK)?;
```

License: MIT
