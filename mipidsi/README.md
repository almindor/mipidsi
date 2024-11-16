# mipidsi

<p align="center">
    <a href="https://crates.io/crates/mipidsi"><img src="https://img.shields.io/crates/v/mipidsi.svg" alt="Crates.io"></a>
    <a href="https://docs.rs/mipidsi"><img src="https://docs.rs/mipidsi/badge.svg" alt="Docs.rs"></a>
    <a href="https://matrix.to/#/#rust-embedded-graphics:matrix.org"><img src="https://img.shields.io/matrix/rust-embedded-graphics:matrix.org" alt="Discuss driver on embedded-graphics on Matrix"></a>
</p>

This crate provides a generic display driver to connect to TFT displays
that implement the [MIPI Display Command Set](https://www.mipi.org/specifications/display-command-set).

Uses [display_interface](https://crates.io/crates/display-interface) to talk to the hardware via transports (currently SPI, I2C and Parallel GPIO).

An optional batching of draws is supported via the `batch` feature (default on).

An optional `buffered` feature enabled buffering to improve performance for operations such as `fill_solid`. It allocates stack buffers of size `512` bytes during some draw operations. This is default turned on.

*NOTES*:

* The name of this crate is a bit unfortunate as this driver works with displays that use the MIPI Display Command Set via any transport supported by [display_interface](https://crates.io/crates/display-interface) but MIPI Display Serial Interface is NOT supported at this time.

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).

## Architecture

The `Display` driver itself contains most of the functionality. Each specific display model implements the `Model` trait for every color format it supports. Each model can also have different variants which are handled via the `Builder` struct.

[embedded-graphics-core](https://crates.io/crates/embedded-graphics-core) is used to provide the drawing API.

## Models

Each supported display model can be used either through the `Builder::with_model` call or through a shortcut function such as `Builder::st7789` if provided. External crates can be used to provide additional models, and can even expand the display constructor pool via trait extension.

Variants that require different screen sizes and window addressing offsets are now supported via the `Builder` logic as well (see docs).

### List of supported models

* GC9A01
* ILI9341
* ILI9342C
* ILI9486
* ST7735
* ST7789
* ST7796

## Migration

See [MIGRATION.md](../docs/MIGRATION.md) document.

## Troubleshooting

See [TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md) document.

### Example
```rust
// create a DisplayInterface from SPI and DC pin, with no manual CS control
let di = SPIInterfaceNoCS::new(spi, dc);
// create the ILI9486 display driver in rgb666 color mode from the display interface and use a HW reset pin during init
let mut display = Builder::new(ILI9486Rgb666, di)
    .reset_pin(rst)
    .init(&mut delay)?; // delay provider from your MCU
// clear the display to black
display.clear(Rgb666::BLACK)?;
```

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.75.0 and up. It *might*
compile with older versions but that may change in any new patch release.
