# SPI ST7789 on a Raspberry Pi Zero W Example

This example demonstrates how to use the [Display HAT Mini by Pomoroni](https://shop.pimoroni.com/products/display-hat-mini?variant=39496084717651) on a Raspberry Pi Zero W.

The example shows a scrolling text and a pulsing RGB LED.

Buttons:

- A: change LED color
- B: exit
- X: move text up
- Y: move text down

## Pre-requisite

**Enable SPI** by any of this options:

- `sudo raspi-config`
- `sudo raspi-config nonint do_spi 0`
- manually adding `dtparam=spi=on` to /boot/config.txt
- the graphical Raspberry Pi Configuration UI

[More info about SPI](https://docs.golemparts.com/rppal/0.14.1/rppal/spi/index.html#spi0)

## Build, strip, copy, and run

### Mac OS

Pre-requisite: musl-cross (Homebrew users: `brew install FiloSottile/musl-cross/musl-cross --without-x86_64 --with-arm-hf`)

```bash
# build for rpi zero w
cargo build --release --target=arm-unknown-linux-musleabihf
# look at the size of the bin file
ls -lh target/arm-unknown-linux-musleabihf/release/spi-st7789-rpi-zero-w
# strip it
arm-linux-musleabihf-strip target/arm-unknown-linux-musleabihf/release/spi-st7789-rpi-zero-w
# look at it now ;)
ls -lh target/arm-unknown-linux-musleabihf/release/spi-st7789-rpi-zero-w
# copy over ssh
scp target/arm-unknown-linux-musleabihf/release/spi-st7789-rpi-zero-w pi@raspberrypi.local:~/
# ssh into the rpi to run it
ssh pi@raspberrypi.local
# run it
./spi-st7789-rpi-zero-w
```

### Linux

Not tested. Follow this article [Raspberry Pi Zero Raspbian/Rust Primer](https://dev.to/jeikabu/raspberry-pi-zero-raspbian-rust-primer-3aj6).
