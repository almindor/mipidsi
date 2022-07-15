# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Changed
- bumped MSRV to 1.59

### Added
- added `Model::address_window_offset()` to `Model` trait
- added `ST7789VW` waveshare model

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