# SPI ST7789 on a Raspberry Pi Zero W Example

Tested with [Display HAT Mini by Pomoroni](https://shop.pimoroni.com/products/display-hat-mini?variant=39496084717651).

## Build, strip, copy, and run

### Mac OS

Prerequisite: musl-cross (Homebrew users: `brew install filosottile/musl-cross/musl-cross`)

```bash
# build for rpi zero 2 w
cargo build --release --target=arm-unknown-linux-musleabihf -p spi-st7789-rpi-zero-w
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
