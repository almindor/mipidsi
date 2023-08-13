use display_interface_spi::SPIInterface;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
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

    let w = 240_u16;
    let h = 320_u16;
    let mut display = Builder::st7789(di)
        .with_display_size(w, h)
        // .with_orientation(mipidsi::Orientation::Landscape(false))
        .with_invert_colors(mipidsi::ColorInversion::Inverted)
        .init(&mut delay, None::<OutputPin>)
        .unwrap();

    // Turn on backlight
    backlight.set_low();
    sleep(Duration::from_millis(150));
    backlight.set_high();

    // Clear the display initially
    let style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::BLACK)
        .build();
    Rectangle::new(Point::new(0, 0), Size::new(w.into(), h.into()))
        .into_styled(style)
        .draw(&mut display)
        .unwrap();
    sleep(Duration::from_millis(150));

    loop {
        // Fill the display with red
        let style = PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build();
        Rectangle::new(Point::new(0, 0), Size::new(w.into(), h.into()))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        // Wait for some time
        sleep(Duration::from_millis(500));

        // Clear the display
        let style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLUE)
            .build();
        Rectangle::new(Point::new(0, 0), Size::new(w.into(), h.into()))
            .into_styled(style)
            .draw(&mut display)
            .unwrap();

        // Wait for some time
        sleep(Duration::from_millis(500));
    }
}
