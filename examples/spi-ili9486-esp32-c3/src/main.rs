#![no_std]
#![no_main]

/* --- Needed by ESP32-c3 --- */
use esp_backtrace as _;
use hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    spi::{Spi, SpiMode},
    timer::TimerGroup,
    Delay, Rtc, IO,
};
/* -------------------------- */

use embedded_graphics::{
    // Provides the necessary functions to draw on the display
    draw_target::DrawTarget,
    // Provides colors from the Rgb565 color space
    pixelcolor::Rgb565,
    prelude::RgbColor,
};

// Provides the parallel port and display interface builders
use display_interface_spi::SPIInterfaceNoCS;

// Provides the Display builder
use mipidsi::Builder;

use fugit::RateExtU32;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    // Define the delay struct, needed for the display driver
    let mut delay = Delay::new(&clocks);

    // Define the Data/Command select pin as a digital output
    let dc = io.pins.gpio7.into_push_pull_output();
    // Define the reset pin as digital outputs and make it high
    let mut rst = io.pins.gpio8.into_push_pull_output();
    rst.set_high().unwrap();

    // Define the SPI pins and create the SPI interface
    let sck = io.pins.gpio12;
    let miso = io.pins.gpio11;
    let mosi = io.pins.gpio13;
    let cs = io.pins.gpio10;
    let spi = Spi::new(
        peripherals.SPI2,
        sck,
        mosi,
        miso,
        cs,
        100_u32.kHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    // Define the display interface with no chip select
    let di = SPIInterfaceNoCS::new(spi, dc);

    // Define the display drom the display interface and initialize it
    let mut display = Builder::ili9486_rgb565(di)
        .init(&mut delay, Some(rst))
        .unwrap();

    // Make the display all black
    display.clear(Rgb565::BLACK).unwrap();

    // Turn SPRITE into an array of pixels in the right colorspace
    let sprite: [Rgb565; 256] = SPRITE.map(|(r, g, b)| Rgb565::new(r, g, b));

    // Draw the 16*16 sprite on the top left corner of the screen
    display.set_pixels(0, 0, 15, 15, sprite).unwrap();

    loop {
        // Do nothing
    }
}

// Contains all the pixel colors for a 16*16 sprite as a 256 value array of (r, g, b) values
const SPRITE: [(u8, u8, u8); 256] = [
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (255, 255, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (124, 252, 0),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 191, 255),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (0, 0, 0xCD),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (75, 0, 130),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (0xff, 0, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
    (255, 69, 0),
];
