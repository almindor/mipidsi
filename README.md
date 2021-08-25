# mipidsi

This crate provides a genericddisplay driver to connect to TFT displays
that implement the [MIPI DSI](https://www.mipi.org/specifications/dsi).
Currently only supports SPI with DC pin setups via the [display_interface]

An optional batching of draws is supported via the `batch` feature (default on)

### Example
```rust
// create a DisplayInterface from SPI and DC pin, with no manual CS control
let di = SPIInterfaceNoCS::new(spi, dc);
// create the ILI9486 display driver from the display interface and RST pin
let mut display = Display::ili9486(di, rst);
// clear the display to black
display.clear(Rgb666::BLACK)?;

License: MIT
