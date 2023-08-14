use display_interface_spi::SPIInterface;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use mipidsi::Builder;
use rppal::gpio::{Gpio, OutputPin};
use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

// Pins
const SPI_CS: u8 = 1;
const SPI_DC: u8 = 9;
const BACKLIGHT: u8 = 13;

// Display
const W: i32 = 320;
const H: i32 = 240;

fn main() -> ExitCode {
    let gpio = Gpio::new().unwrap();
    let dc = gpio.get(SPI_DC).unwrap().into_output();
    let cs = gpio.get(SPI_CS).unwrap().into_output();
    let mut backlight = gpio.get(BACKLIGHT).unwrap().into_output();

    let mut delay = Delay::new();

    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss1, 60_000_000_u32, Mode::Mode0).unwrap();
    let di = SPIInterface::new(spi, dc, cs);

    let mut display = Builder::st7789(di)
        // width and height are switched on porpuse because of the orientation
        .with_display_size(H as u16, W as u16)
        // this orientation applies for the Display HAT Mini by Pimoroni
        .with_orientation(mipidsi::Orientation::LandscapeInverted(true))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, None::<OutputPin>)
        .unwrap();

    // Text
    let char_w = 6;
    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let text = "Hello World ^_^";
    let mut text_x = W;

    // Alternating color
    let mut colors = [Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE]
        .into_iter()
        .cycle();

    // Clear the display initially
    display.clear(colors.nth(0).unwrap()).unwrap();

    // Turn on backlight
    backlight.set_high();

    loop {
        // Fill the display with alternating colors
        display.clear(colors.next().unwrap()).unwrap();

        // Draw text
        let right = Text::new(text, Point::new(text_x, H / 2), text_style)
            .draw(&mut display)
            .unwrap();
        text_x = if right.x <= 0 { W } else { text_x - char_w };

        // Wait for some time
        sleep(Duration::from_millis(250));
    }
}
