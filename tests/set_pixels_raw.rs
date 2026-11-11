use std::{cell::RefCell, convert::Infallible, rc::Rc};

use embedded_hal::{
    delay::DelayNs,
    digital::{self, OutputPin},
    spi::{self, Operation, SpiDevice},
};
use mipidsi::{
    interface::{Interface, InterfaceKind, SpiInterface},
    models::{ILI9486Rgb666, ST7789},
    options::{Orientation, Rotation},
    Builder, SetPixelsRawError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Command(u8),
    Data(Vec<u8>),
}

type Log = Rc<RefCell<Vec<Event>>>;

/// SPI device that logs every write, tagged with the state of the DC pin.
struct LogSpi {
    log: Log,
    dc_high: Rc<RefCell<bool>>,
}

impl spi::ErrorType for LogSpi {
    type Error = Infallible;
}

impl SpiDevice for LogSpi {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        for op in operations {
            if let Operation::Write(data) = op {
                if data.is_empty() {
                    continue;
                }
                let event = if *self.dc_high.borrow() {
                    Event::Data(data.to_vec())
                } else {
                    assert_eq!(data.len(), 1);
                    Event::Command(data[0])
                };
                self.log.borrow_mut().push(event);
            }
        }
        Ok(())
    }
}

struct DcPin(Rc<RefCell<bool>>);

impl digital::ErrorType for DcPin {
    type Error = Infallible;
}

impl OutputPin for DcPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        *self.0.borrow_mut() = false;
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        *self.0.borrow_mut() = true;
        Ok(())
    }
}

struct NoDelay;

impl DelayNs for NoDelay {
    fn delay_ns(&mut self, _ns: u32) {}
}

fn spi_interface<'a>(log: &Log, buffer: &'a mut [u8]) -> SpiInterface<'a, LogSpi, DcPin> {
    let dc_high = Rc::new(RefCell::new(false));
    let spi = LogSpi {
        log: log.clone(),
        dc_high: dc_high.clone(),
    };
    SpiInterface::new(spi, DcPin(dc_high), buffer)
}

#[test]
fn spi_sends_buffer_unchanged_in_a_single_write() {
    let log = Log::default();
    let mut buffer = [0u8; 16];
    let mut display = Builder::new(ST7789, spi_interface(&log, &mut buffer))
        .init(&mut NoDelay)
        .unwrap();
    log.borrow_mut().clear();

    // 3x2 pixels, larger than the 16 byte interface buffer
    let data: Vec<u8> = (0..3 * 2 * 2).map(|i| i as u8).collect();
    display.set_pixels_raw(10, 20, 12, 21, &data).unwrap();

    assert_eq!(
        *log.borrow(),
        [
            Event::Command(0x2A),
            Event::Data(vec![0, 10, 0, 12]),
            Event::Command(0x2B),
            Event::Data(vec![0, 20, 0, 21]),
            Event::Command(0x2C),
            Event::Data(data),
        ]
    );
}

#[test]
fn spi_large_buffer_is_not_split() {
    let log = Log::default();
    let mut buffer = [0u8; 8];
    let mut display = Builder::new(ST7789, spi_interface(&log, &mut buffer))
        .init(&mut NoDelay)
        .unwrap();
    log.borrow_mut().clear();

    let data = vec![0xAB; 240 * 320 * 2];
    display.set_pixels_raw(0, 0, 239, 319, &data).unwrap();

    let log = log.borrow();
    assert_eq!(log.last(), Some(&Event::Data(data)));
}

#[test]
fn display_offset_is_applied() {
    let log = Log::default();
    let mut buffer = [0u8; 16];
    let mut display = Builder::new(ST7789, spi_interface(&log, &mut buffer))
        .display_size(135, 240)
        .display_offset(52, 40)
        .init(&mut NoDelay)
        .unwrap();
    log.borrow_mut().clear();

    display.set_pixels_raw(0, 0, 0, 0, &[0x12, 0x34]).unwrap();

    let log = log.borrow();
    assert_eq!(log[1], Event::Data(vec![0, 52, 0, 52]));
    assert_eq!(log[3], Event::Data(vec![0, 40, 0, 40]));
}

#[test]
fn rgb666_uses_three_bytes_per_pixel() {
    let log = Log::default();
    let mut buffer = [0u8; 16];
    let mut display = Builder::new(ILI9486Rgb666, spi_interface(&log, &mut buffer))
        .init(&mut NoDelay)
        .unwrap();

    assert!(matches!(
        display.set_pixels_raw(0, 0, 1, 0, &[0; 4]),
        Err(SetPixelsRawError::InvalidLength)
    ));
    display.set_pixels_raw(0, 0, 1, 0, &[0; 6]).unwrap();
}

#[test]
fn invalid_regions_are_rejected_without_sending_anything() {
    let log = Log::default();
    let mut buffer = [0u8; 16];
    let mut display = Builder::new(ST7789, spi_interface(&log, &mut buffer))
        .display_size(240, 320)
        .init(&mut NoDelay)
        .unwrap();
    log.borrow_mut().clear();

    let data = [0u8; 4];
    for (sx, sy, ex, ey) in [
        (239, 0, 240, 0), // past the right edge
        (0, 319, 1, 320), // past the bottom edge
        (5, 0, 4, 0),     // sx > ex
        (0, 5, 1, 4),     // sy > ey
    ] {
        assert!(
            matches!(
                display.set_pixels_raw(sx, sy, ex, ey, &data),
                Err(SetPixelsRawError::OutOfBounds)
            ),
            "region ({sx}, {sy}) - ({ex}, {ey})"
        );
    }

    assert!(matches!(
        display.set_pixels_raw(0, 0, 1, 0, &[0; 3]),
        Err(SetPixelsRawError::InvalidLength)
    ));
    assert!(matches!(
        display.set_pixels_raw(0, 0, 1, 0, &[0; 5]),
        Err(SetPixelsRawError::InvalidLength)
    ));

    assert!(log.borrow().is_empty());
}

#[test]
fn bounds_follow_orientation() {
    let log = Log::default();
    let mut buffer = [0u8; 16];
    let mut display = Builder::new(ST7789, spi_interface(&log, &mut buffer))
        .display_size(240, 320)
        .init(&mut NoDelay)
        .unwrap();

    let data = [0u8; 2];
    assert!(matches!(
        display.set_pixels_raw(300, 0, 300, 0, &data),
        Err(SetPixelsRawError::OutOfBounds)
    ));

    display
        .set_orientation(Orientation::default().rotate(Rotation::Deg90))
        .unwrap();
    display.set_pixels_raw(300, 0, 300, 0, &data).unwrap();
    assert!(matches!(
        display.set_pixels_raw(0, 300, 0, 300, &data),
        Err(SetPixelsRawError::OutOfBounds)
    ));
}

/// Interface that doesn't override `send_raw_pixels`.
struct DefaultImplInterface(Vec<u8>);

impl Interface for DefaultImplInterface {
    type Word = u8;
    type Error = Infallible;

    const KIND: InterfaceKind = InterfaceKind::Serial4Line;

    fn send_command(&mut self, _command: u8, _args: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error> {
        self.0.extend(pixels.into_iter().flatten());
        Ok(())
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        _pixel: [Self::Word; N],
        _count: u32,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[test]
fn default_implementation_forwards_to_send_pixels() {
    let mut display = Builder::new(ST7789, DefaultImplInterface(Vec::new()))
        .init(&mut NoDelay)
        .unwrap();

    let data = [1, 2, 3, 4, 5, 6, 7, 8];
    display.set_pixels_raw(0, 0, 1, 1, &data).unwrap();

    let (di, _, _) = display.release();
    assert_eq!(di.0, data);
}
