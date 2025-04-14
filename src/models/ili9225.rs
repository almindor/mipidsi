use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_hal::delay::DelayNs;

use crate::dcs::{AddressMode, InterfaceExt};
use crate::options::{ColorOrder, Orientation, RefreshOrder, Rotation};
use crate::{
    interface::Interface,
    models::{Model, ModelInitError},
    options::ModelOptions,
};

/// ILI9225 display in Rgb565 color mode.
pub struct ILI9225Rgb565;

const ILI9225_POWER_CTRL1: u8 = 0x10;
const ILI9225_POWER_CTRL2: u8 = 0x11;
const ILI9225_POWER_CTRL3: u8 = 0x12;
const ILI9225_POWER_CTRL4: u8 = 0x13;
const ILI9225_POWER_CTRL5: u8 = 0x14;

const ILI9225_DRIVER_OUTPUT_CTRL: u8 = 0x01; // Driver Output Control
const ILI9225_LCD_AC_DRIVING_CTRL: u8 = 0x02; // LCD AC Driving Control
const ILI9225_ENTRY_MODE: u8 = 0x03; // Entry Mode
const ILI9225_DISP_CTRL1: u8 = 0x07; // Display Control 1
const ILI9225_BLANK_PERIOD_CTRL1: u8 = 0x08; // Blank Period Control
const ILI9225_FRAME_CYCLE_CTRL: u8 = 0x0B; // Frame Cycle Control
const ILI9225_INTERFACE_CTRL: u8 = 0x0C; // Interface Control
const ILI9225_OSC_CTRL: u8 = 0x0F; // Osc Control
const ILI9225_VCI_RECYCLING: u8 = 0x15; // Osc Control
const ILI9225_RAM_ADDR_SET1: u8 = 0x20; // Osc Control
const ILI9225_RAM_ADDR_SET2: u8 = 0x21; // Osc Control

const ILI9225_GATE_SCAN_CTRL: u8 = 0x30; // Gate Scan Control Register
const ILI9225_VERTICAL_SCROLL_CTRL1: u8 = 0x31; // Vertical Scroll Control 1 Register
const ILI9225_VERTICAL_SCROLL_CTRL2: u8 = 0x32; // Vertical Scroll Control 2 Register
const ILI9225_VERTICAL_SCROLL_CTRL3: u8 = 0x33; // Vertical Scroll Control 3 Register
const ILI9225_PARTIAL_DRIVING_POS1: u8 = 0x34; // Partial Driving Position 1 Register
const ILI9225_PARTIAL_DRIVING_POS2: u8 = 0x35; // Partial Driving Position 2 Register
const ILI9225_HORIZONTAL_WINDOW_ADDR1: u8 = 0x36; // Horizontal Address Start Position
const ILI9225_HORIZONTAL_WINDOW_ADDR2: u8 = 0x37; // Horizontal Address End Position
const ILI9225_VERTICAL_WINDOW_ADDR1: u8 = 0x38; // Vertical Address Start Position
const ILI9225_VERTICAL_WINDOW_ADDR2: u8 = 0x39; // Vertical Address End Position

const ILI9225_GAMMA_CTRL1: u8 = 0x50; // Gamma Control 1
const ILI9225_GAMMA_CTRL2: u8 = 0x51; // Gamma Control 2
const ILI9225_GAMMA_CTRL3: u8 = 0x52; // Gamma Control 3
const ILI9225_GAMMA_CTRL4: u8 = 0x53; // Gamma Control 4
const ILI9225_GAMMA_CTRL5: u8 = 0x54; // Gamma Control 5
const ILI9225_GAMMA_CTRL6: u8 = 0x55; // Gamma Control 6
const ILI9225_GAMMA_CTRL7: u8 = 0x56; // Gamma Control 7
const ILI9225_GAMMA_CTRL8: u8 = 0x57; // Gamma Control 8
const ILI9225_GAMMA_CTRL9: u8 = 0x58; // Gamma Control 9
const ILI9225_GAMMA_CTRL10: u8 = 0x59; // Gamma Control 10

impl Model for ILI9225Rgb565 {
    type ColorFormat = Rgb565;
    type AddressMode = ILI9225AddressMode;
    const FRAMEBUFFER_SIZE: (u16, u16) = (176, 220);

    fn init<DELAY, DI>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<Self::AddressMode, ModelInitError<DI::Error>>
    where
        DELAY: DelayNs,
        DI: Interface,
    {
        let madctl = ILI9225AddressMode::from(options);

        /* Set SS bit and direction output from S528 to S1 */
        di.write_raw(ILI9225_POWER_CTRL1, &[0x00, 0x00])?; // Set SAP,DSTB,STB
        di.write_raw(ILI9225_POWER_CTRL2, &[0x00, 0x00])?; // Set APON,PON,AON,VCI1EN,VC
        di.write_raw(ILI9225_POWER_CTRL3, &[0x00, 0x00])?; // Set BT,DC1,DC2,DC3
        di.write_raw(ILI9225_POWER_CTRL4, &[0x00, 0x00])?; // Set GVDD
        di.write_raw(ILI9225_POWER_CTRL5, &[0x00, 0x00])?; // Set VCOMH/VCOML voltage

        delay.delay_us(40_000);

        di.write_raw(ILI9225_POWER_CTRL1, &[0x00, 0x18])?; // Set APON,PON,AON,VCI1EN,VC
        di.write_raw(ILI9225_POWER_CTRL2, &[0x61, 0x21])?; // Set BT,DC1,DC2,DC3
        di.write_raw(ILI9225_POWER_CTRL3, &[0x00, 0x6F])?; // Set GVDD   /*007F 0088 */
        di.write_raw(ILI9225_POWER_CTRL4, &[0x49, 0x5F])?; // Set VCOMH/VCOML voltage
        di.write_raw(ILI9225_POWER_CTRL5, &[0x08, 0x00])?; // Set SAP,DSTB,STB
        delay.delay_us(10_000);
        di.write_raw(ILI9225_POWER_CTRL2, &[0x10, 0x3B])?; // Set APON,PON,AON,VCI1EN,VC
        delay.delay_us(50_000);

        di.write_raw(ILI9225_DRIVER_OUTPUT_CTRL, &[0x01, 0x1C])?; // set the display line number and display direction
        di.write_raw(ILI9225_LCD_AC_DRIVING_CTRL, &[0x01, 0x00])?; // set 1 line inversion
        di.write_raw(ILI9225_ENTRY_MODE, &[0x10, 0x30])?; // set GRAM write direction and BGR=1.
        di.write_raw(ILI9225_DISP_CTRL1, &[0x00, 0x00])?; // Display off
        di.write_raw(ILI9225_BLANK_PERIOD_CTRL1, &[0x08, 0x08])?; // set the back porch and front porch
        di.write_raw(ILI9225_FRAME_CYCLE_CTRL, &[0x11, 0x00])?; // set the clocks number per line
        di.write_raw(ILI9225_INTERFACE_CTRL, &[0x00, 0x00])?; // CPU interface
        di.write_raw(ILI9225_OSC_CTRL, &[0x0D, 0x01])?; // Set Osc  /*0e01*/
        di.write_raw(ILI9225_VCI_RECYCLING, &[0x00, 0x20])?; // Set VCI recycling
        di.write_raw(ILI9225_RAM_ADDR_SET1, &[0x00, 0x00])?; // RAM Address
        di.write_raw(ILI9225_RAM_ADDR_SET2, &[0x00, 0x00])?; // RAM Address

        /* Set GRAM area */
        di.write_raw(ILI9225_GATE_SCAN_CTRL, &[0x00, 0x00])?;
        di.write_raw(ILI9225_VERTICAL_SCROLL_CTRL1, &[0x00, 0xDB])?;
        di.write_raw(ILI9225_VERTICAL_SCROLL_CTRL2, &[0x00, 0x00])?;
        di.write_raw(ILI9225_VERTICAL_SCROLL_CTRL3, &[0x00, 0x00])?;
        di.write_raw(ILI9225_PARTIAL_DRIVING_POS1, &[0x00, 0xDB])?;
        di.write_raw(ILI9225_PARTIAL_DRIVING_POS2, &[0x00, 0x00])?;
        di.write_raw(ILI9225_HORIZONTAL_WINDOW_ADDR1, &[0x00, 0xAF])?;
        di.write_raw(ILI9225_HORIZONTAL_WINDOW_ADDR2, &[0x00, 0x00])?;
        di.write_raw(ILI9225_VERTICAL_WINDOW_ADDR1, &[0x00, 0xDB])?;
        di.write_raw(ILI9225_VERTICAL_WINDOW_ADDR2, &[0x00, 0x00])?;

        /* Set GAMMA curve */
        di.write_raw(ILI9225_GAMMA_CTRL1, &[0x00, 0x00])?;
        di.write_raw(ILI9225_GAMMA_CTRL2, &[0x08, 0x08])?;
        di.write_raw(ILI9225_GAMMA_CTRL3, &[0x08, 0x0A])?;
        di.write_raw(ILI9225_GAMMA_CTRL4, &[0x00, 0x0A])?;
        di.write_raw(ILI9225_GAMMA_CTRL5, &[0x0A, 0x08])?;
        di.write_raw(ILI9225_GAMMA_CTRL6, &[0x08, 0x08])?;
        di.write_raw(ILI9225_GAMMA_CTRL7, &[0x00, 0x00])?;
        di.write_raw(ILI9225_GAMMA_CTRL8, &[0x0A, 0x00])?;
        di.write_raw(ILI9225_GAMMA_CTRL9, &[0x07, 0x10])?;
        di.write_raw(ILI9225_GAMMA_CTRL10, &[0x07, 0x10])?;

        di.write_raw(ILI9225_DISP_CTRL1, &[0x00, 0x12])?;
        delay.delay_us(50_000);
        di.write_raw(ILI9225_DISP_CTRL1, &[0x10, 0x17])?;

        Ok(madctl)
    }

    fn update_address_window<DI>(
        di: &mut DI,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
    ) -> Result<(), DI::Error>
    where
        DI: Interface,
    {
        di.write_raw(0x37, &sx.to_be_bytes())?;
        di.write_raw(0x36, &ex.to_be_bytes())?;
        di.write_raw(0x39, &sy.to_be_bytes())?;
        di.write_raw(0x38, &ey.to_be_bytes())?;
        di.write_raw(0x20, &sx.to_be_bytes())?;
        di.write_raw(0x21, &sy.to_be_bytes())
    }

    fn sleep<DI, DELAY>(di: &mut DI, delay: &mut DELAY) -> Result<(), DI::Error>
    where
        DI: Interface,
        DELAY: DelayNs,
    {
        di.write_raw(ILI9225_DISP_CTRL1, &[0x00, 0x00])?;
        delay.delay_us(50_000);
        di.write_raw(ILI9225_POWER_CTRL2, &[0x00, 0x07])?;
        delay.delay_us(50_000);
        di.write_raw(ILI9225_POWER_CTRL1, &[0x0A, 0x01])
    }

    fn wake<DI, DELAY>(di: &mut DI, delay: &mut DELAY) -> Result<(), DI::Error>
    where
        DI: Interface,
        DELAY: DelayNs,
    {
        di.write_raw(ILI9225_POWER_CTRL1, &[0x0A, 0x00])?;
        di.write_raw(ILI9225_POWER_CTRL2, &[0x10, 0x3B])?;
        delay.delay_us(50_000);
        di.write_raw(ILI9225_DISP_CTRL1, &[0x10, 0x17])
    }

    fn write_memory_start<DI>(di: &mut DI) -> Result<(), DI::Error>
    where
        DI: Interface,
    {
        di.write_command(crate::dcs::WriteMemoryStartILI9225)
    }

}

/// Set Address Mode
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ILI9225AddressMode {
    rotation: u8,
    color_order: ColorOrder,
}

impl ILI9225AddressMode {
    /// Create New AddressMode for ILI9225, mirrored will been ignore ili9225 not supported it.
    pub const fn new(color_order: ColorOrder, orientation: Orientation) -> Self {
        let rotation = match orientation.rotation {
            Rotation::Deg0 => 0,
            Rotation::Deg90 => 1,
            Rotation::Deg180 => 2,
            Rotation::Deg270 => 3,
        };
        Self {
            rotation,
            color_order,
        }
    }

    
}

impl From<&ModelOptions> for ILI9225AddressMode {
    fn from(options: &ModelOptions) -> Self {
        Self::default()
            .with_color_order(options.color_order)
            .with_orientation(options.orientation)
            .with_refresh_order(options.refresh_order)
    }
}

impl AddressMode for ILI9225AddressMode {
    fn with_color_order(self, color_order: ColorOrder) -> Self {
        Self { color_order, ..self }
    }

    fn with_orientation(self, orientation: Orientation) -> Self {
        let rotation = match orientation.rotation {
            Rotation::Deg0 => 0,
            Rotation::Deg90 => 1,
            Rotation::Deg180 => 2,
            Rotation::Deg270 => 3,
        };
        Self { rotation, ..self }
    }

    fn with_refresh_order(self, _refresh_order: RefreshOrder) -> Self {
        self // Ignore it
    }

    fn send_commands<DI>(&self, di: &mut DI) -> Result<(), DI::Error>
    where
        DI: Interface,
    {
        let rotation = self.rotation % 4; // Only accept 0-3

        // Command 1: DRIVER_OUTPUT_CTRL (0x01)
        let driver_high_byte = match rotation {
            0 => 0x01, // 0°
            1 => 0x00, // 90°
            2 => 0x02, // 180°
            3 => 0x03, // 270°
            _ => 0x01, // Not reachable
        };
        let driver_params = [driver_high_byte, 0x1C];
        di.write_raw(0x01, &driver_params)?;

        // Command 2: ENTRY_MODE (0x03)
        let color_order_byte = if self.color_order == ColorOrder::Bgr { 0x10 } else { 0x00 };
        let entry_low_byte = if rotation == 1 || rotation == 3 { 0x38 } else { 0x30 };
        let entry_params = [color_order_byte, entry_low_byte];
        di.write_raw(0x03, &entry_params)?;

        Ok(())
    }
}