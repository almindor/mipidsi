// This example is made for the Raspberry Pi Pico, using the `rp-hal`
// It demonstrates how to set up an ili9341 display with the Rgb666 color space
// using a parallel port

/* --- Needed by RPI Pico --- */
#![no_std]
#![no_main]
use bsp::entry;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac,
    sio::Sio,
    watchdog::Watchdog,
};
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;
/* -------------------------- */

use embedded_graphics::{
    // Provides the necessary functions to draw on the display
    draw_target::DrawTarget,
    // Provides colors from the Rgb666 color space
    pixelcolor::Rgb666,
    prelude::RgbColor,
};

// Provides the parallel port and display interface builders
use display_interface_parallel_gpio::{Generic8BitBus, PGPIO8BitInterface};

// Provides the Display builder
use mipidsi::Builder;

#[entry]
fn main() -> ! {
    // Define the pico's singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // Define the pico's clocks, needed for the delay
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Define the delay struct, needed for the display driver
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Define the pins, needed to define the display interface
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Define the reset and write enable pins as digital outputs and make them high
    let rst = pins
        .gpio7
        .into_push_pull_output_in_state(gpio::PinState::High);
    let wr = pins
        .gpio5
        .into_push_pull_output_in_state(gpio::PinState::High);

    // Define the Data/Command select pin as a digital output
    let dc = pins.gpio6.into_push_pull_output();

    // Define the pins used for the parallel interface as digital outputs
    let lcd_d0 = pins.gpio15.into_push_pull_output();
    let lcd_d1 = pins.gpio14.into_push_pull_output();
    let lcd_d2 = pins.gpio13.into_push_pull_output();
    let lcd_d3 = pins.gpio12.into_push_pull_output();
    let lcd_d4 = pins.gpio11.into_push_pull_output();
    let lcd_d5 = pins.gpio10.into_push_pull_output();
    let lcd_d6 = pins.gpio9.into_push_pull_output();
    let lcd_d7 = pins.gpio8.into_push_pull_output();

    // Define the parallel bus with the previously defined parallel port pins
    let bus = Generic8BitBus::new((
        lcd_d0, lcd_d1, lcd_d2, lcd_d3, lcd_d4, lcd_d5, lcd_d6, lcd_d7,
    ))
    .unwrap();

    // Define the display interface from a generic 8 bit bus, a Data/Command select pin and a write enable pin
    let di = PGPIO8BitInterface::new(bus, dc, wr);

    // Define the display from the display bus, set the color order as Bgr and initialize it with
    // the delay struct and the reset pin
    let mut display = Builder::ili9341_rgb666(di)
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .init(&mut delay, Some(rst))
        .unwrap();

    // Set the display all red
    display.clear(Rgb666::RED).unwrap();

    loop {
        // Do nothing
    }
}
