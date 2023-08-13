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

const SPI_DC: u8 = 9;
const SPI_CS: u8 = 1;
const BACKLIGHT: u8 = 13;

fn main() -> ExitCode {
    let gpio = Gpio::new().unwrap();
    let dc = gpio.get(SPI_DC).unwrap().into_output();
    let cs = gpio.get(SPI_CS).unwrap().into_output();
    let mut backlight = gpio.get(BACKLIGHT).unwrap().into_output();

    let mut delay = Delay::new();

    let clock_speed = 60_000_000_u32;
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss1, clock_speed, Mode::Mode0).unwrap();
    let di = SPIInterface::new(spi, dc, cs);

    const W: i32 = 320;
    const H: i32 = 240;
    let mut display = Builder::st7789(di)
        // width and height are switched on porpuse because of the orientation
        .with_display_size(H as u16, W as u16)
        // this orientation applies for the Display HAT Mini by Pimoroni
        .with_orientation(mipidsi::Orientation::LandscapeInverted(true))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, None::<OutputPin>)
        .unwrap();

    // Text
    let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
    let text = "Hello World ^_^";
    let text_w = text.len() as i32 * 6;
    let mut text_x = W;

    // Anternating color
    let colors = [Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE];
    let mut color_index = 0;

    // Clear the display initially
    display.clear(colors[0]).unwrap();

    // Turn on backlight
    backlight.set_high();

    loop {
        // Text scroll
        text_x = (text_x - 6) % (W + text_w);
        if text_x < -text_w {
            text_x = W;
        }

        // Fill the display with red
        display.clear(colors[color_index]).unwrap();
        color_index = (color_index + 1) % colors.len();

        // Draw text
        Text::new(text, Point::new(text_x, H / 2), character_style)
            .draw(&mut display)
            .unwrap();

        // Wait for some time
        sleep(Duration::from_millis(250));
    }
}
