# mipidsi

This repository provides generic display drivers to connect to TFT displays
that implement the [MIPI Display Command Set](https://www.mipi.org/specifications/display-command-set).

It consists of

* [mipidsi](mipidsi) - sync version of the driver
* [mipidsi-async](mipidsi-async) - async version of the driver

*NOTES*:

* The name of these crates is a bit unfortunate as the drivers work with displays that use the MIPI Display Command Set via any transport supported by [display_interface](https://crates.io/crates/display-interface) but MIPI Display Serial Interface is NOT supported at this time.

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).