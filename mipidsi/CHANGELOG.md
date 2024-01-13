# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added

- added `GC9A01` model support
- added `Display::wake` method
- added `Display::sleep` method
- added `Display::is_sleeping` method
- added `Display::dcs` method to allow sending custom DCS commands to the device

## [v0.7.1] - 2023-05-24

### Changed

 - fixed MSRV in `Cargo.toml` to match the rest at `v1.61`

## [v0.7.0] - 2023-05-24

### Changed

- switched `embedded-graphics-core v0.4.0`
- updated initialization delays `ILI934x` model

## [v0.6.0] - 2023-01-12

### Added

- added `Builder::with_window_offset_handler` method
- added `ModelOptions::invert_colors` flag
- added `Builder::with_invert_colors(bool)` method
- added `ILI9341` model support

### Changed

- `Model::init` changed to expect `options: &ModelOptions`
- reworked how `DCS` instructions are handled using the new `dcs` module and `DcsCommand` trait and implementations
- reworked model init functions to use new `dcs` module

### Removed

- removed duplicated `INVON` call in `ST7735s` model init

## [v0.5.0] - 2022-10-19

### Added

- added the `Builder` as construction method for displays to simplify configuration
and protect against use-before-init bugs
- added `Model::default_options()` so that each model can provide a sane default regardless of helper constructors

### Changed

- `Model` no longer has to own `ModelOptions`
- `Model::new` was removed
- the optional `RST` reset hw pin is now only used during the `Builder::init` call

### Removed

- removed direct `Display` constructors. Use `Builder` instead (see migration guide)
- removed `DisplayOptions` in favour of `Builder` settings

## [v0.4.0] - 2022-09-30

### Added

- support for model variants via `DisplayOptions`
- support for `raspberry pico1` variant of the `ST7789` display
- support for the `waveshare` variants of the `ST7789` display

### Changed

- split [DisplayOptions] into [DisplayOptions] and [ModelOptions] with sizing initialization safety constructors
- refactored `Display::init` and constructors to match new variant code
- fixed off by one error in fill operations

### Removed

- removed "no reset pin" constructor helpers (uses `Option` now)

## [v0.3.0] - 2022-08-30

### Added

- added `ILI9342C` model support thanks to [Jesse Braham's](https://github.com/jessebraham) [PR](https://github.com/almindor/mipidsi/pull/25)

## [v0.2.2] - 2022-08-26

### Changed

- fix `Display::clear` out of bounds pixels
- remove `ST7789` model `Bgr` bit override

## [v0.2.1] - 2022-08-03

### Added

- clarified display model constructor usage in `README`

### Changed

- fix `i32` -> `u16` conversion overflow bug in `batch` module in case of negative coordinates

## [v0.2.0] - 2021-04-12

### Changed
- fix RGB/BGR color issue on some models
- expand `Orientation` to use mirror image settings properly
- change `Display::init` to include `DisplayOptions` and allow setting all `MADCTL` values on init, including `Orientation`
- fix issues [#6](https://github.com/almindor/mipidsi/issues/6), [#8](https://github.com/almindor/mipidsi/issues/8) and [#10](https://github.com/almindor/mipidsi/issues/10)
    - big thanks to [@brianmay](https://github.com/brianmay) and [@KerryRJ](https://github.com/KerryRJ)

## [v0.1.0] - 2021-09-09

### Added
- Initial release
