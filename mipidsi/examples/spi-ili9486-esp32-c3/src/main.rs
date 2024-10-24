#![no_std]
#![no_main]

use embedded_hal_bus::spi::ExclusiveDevice;
/* --- Needed by ESP32-c3 --- */
use esp_backtrace as _;
use hal::{
    delay::Delay,
    gpio::{Io, Level, Output},
    prelude::*,
    rtc_cntl::Rtc,
    spi::{master::Spi, SpiMode},
    timer::timg::TimerGroup,
};
/* -------------------------- */

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Primitive, PrimitiveStyle, Triangle},
};

// Provides the parallel port and display interface builders
use display_interface_spi::SPIInterface;

// Provides the Display builder
use mipidsi::{models::ILI9486Rgb565, Builder};

use fugit::RateExtU32;

#[entry]
fn main() -> ! {
    let peripherals = hal::init(hal::Config::default());
    // let peripherals = Peripherals::take();
    // let system = SystemControl::new(peripherals.SYSTEM);
    // let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.LPWR);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    let mut wdt1 = timer_group1.wdt;
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    // Define the delay struct, needed for the display driver
    let mut delay = Delay::new();

    // Define the Data/Command select pin as a digital output
    let dc = Output::new(io.pins.gpio7, Level::Low);
    // Define the reset pin as digital outputs and make it high
    let mut rst = Output::new(io.pins.gpio8, Level::Low);
    rst.set_high();

    // Define the SPI pins and create the SPI interface
    let sck = io.pins.gpio12;
    let miso = io.pins.gpio11;
    let mosi = io.pins.gpio13;
    let cs = io.pins.gpio10;
    let spi = Spi::new(peripherals.SPI2, 100_u32.kHz(), SpiMode::Mode0).with_pins(
        sck,
        mosi,
        miso,
        hal::gpio::NoPin,
    );

    let cs_output = Output::new(cs, Level::High);
    let spi_device = ExclusiveDevice::new_no_delay(spi, cs_output).unwrap();

    // Define the display interface with no chip select
    let di = SPIInterface::new(spi_device, dc);

    // Define the display from the display interface and initialize it
    let mut display = Builder::new(ILI9486Rgb565, di)
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();

    // Make the display all black
    display.clear(Rgb565::BLACK).unwrap();

    // Draw a smiley face with white eyes and a red mouth
    draw_smiley(&mut display).unwrap();

    loop {
        // Do nothing
    }
}

fn draw_smiley<T: DrawTarget<Color = Rgb565>>(display: &mut T) -> Result<(), T::Error> {
    // Draw the left eye as a circle located at (50, 100), with a diameter of 40, filled with white
    Circle::new(Point::new(50, 100), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw the right eye as a circle located at (50, 200), with a diameter of 40, filled with white
    Circle::new(Point::new(50, 200), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw an upside down red triangle to represent a smiling mouth
    Triangle::new(
        Point::new(130, 140),
        Point::new(130, 200),
        Point::new(160, 170),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(display)?;

    // Cover the top part of the mouth with a black triangle so it looks closed instead of open
    Triangle::new(
        Point::new(130, 150),
        Point::new(130, 190),
        Point::new(150, 170),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(display)?;

    Ok(())
}
