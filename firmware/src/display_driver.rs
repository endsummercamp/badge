// this driver is from https://github.com/almindor/st7789
// vendored because it needed a few changes to work with the current version of embedded-graphics

// associated re-typing not supported in rust yet
#![allow(clippy::type_complexity)]

//! This crate provides a ST7789 driver to connect to TFT displays.

use core::iter::once;

use display_interface::DataFormat::{self, U16BEIter, U8Iter};
use display_interface::WriteOnlyDataCommand;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[repr(u8)]
// pub enum Instruction {
//     NOP = 0x00,
//     SWRESET = 0x01,
//     RDDID = 0x04,
//     RDDST = 0x09,
//     SLPIN = 0x10,
//     SLPOUT = 0x11,
//     PTLON = 0x12,
//     NORON = 0x13,
//     INVOFF = 0x20,
//     INVON = 0x21,
//     DISPOFF = 0x28,
//     DISPON = 0x29,
//     CASET = 0x2A,
//     RASET = 0x2B,
//     RAMWR = 0x2C,
//     RAMRD = 0x2E,
//     PTLAR = 0x30,
//     VSCRDER = 0x33,
//     TEOFF = 0x34,
//     TEON = 0x35,
//     MADCTL = 0x36,
//     VSCAD = 0x37,
//     COLMOD = 0x3A,
//     VCMOFSET = 0xC5,
// }

/*
#define TFT_NOP     0x00
#define TFT_SWRST   0x01

#define TFT_SLPIN   0x10
#define TFT_SLPOUT  0x11

#define TFT_INVOFF  0x20
#define TFT_INVON   0x21

#define TFT_DISPOFF 0x28
#define TFT_DISPON  0x29

#define TFT_CASET   0x2A
#define TFT_PASET   0x2B
#define TFT_RAMWR   0x2C

#define TFT_RAMRD   0x2E

#define TFT_MADCTL  0x36

#define TFT_MAD_MY  0x80
#define TFT_MAD_MX  0x40
#define TFT_MAD_MV  0x20
#define TFT_MAD_ML  0x10
#define TFT_MAD_RGB 0x00
#define TFT_MAD_BGR 0x08
#define TFT_MAD_MH  0x04
#define TFT_MAD_SS  0x02
#define TFT_MAD_GS  0x01
*/

pub enum Instruction {
    NOP = 0x00,
    SWRESET = 0x01,
    SLPIN = 0x10,
    SLPOUT = 0x11,
    INVOFF = 0x20,
    INVON = 0x21,
    DISPOFF = 0x28,
    DISPON = 0x29,
    CASET = 0x2A,
    RASET = 0x2B,
    RAMWR = 0x2C,
    RAMRD = 0x2E,
    MADCTL = 0x36,
}

///
/// ST7789 driver to connect to TFT displays.
///
pub struct ST7789<DI, RST, BL>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin,
    BL: OutputPin,
{
    // Display interface
    di: DI,
    // Reset pin.
    rst: Option<RST>,
    // Backlight pin,
    bl: Option<BL>,
    // Current orientation
    orientation: Orientation,
}

///
/// Display orientation.
///
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Orientation {
    Portrait = 0b0000_0000,         // no inverting
    Landscape = 0b0110_0000,        // invert column and page/column order
    PortraitSwapped = 0b1100_0000,  // invert page and column order
    LandscapeSwapped = 0b1010_0000, // invert page and page/column order
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait
    }
}

///
/// Tearing effect output setting.
///
#[derive(Copy, Clone)]
pub enum TearingEffect {
    /// Disable output.
    Off,
    /// Output vertical blanking information.
    Vertical,
    /// Output horizontal and vertical blanking information.
    HorizontalAndVertical,
}

///
/// An error holding its source (pins or SPI)
///
#[derive(Debug)]
pub enum Error<PinE> {
    DisplayError,
    Pin(PinE),
}

impl<DI, RST, BL, PinE> ST7789<DI, RST, BL>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    ///
    /// Creates a new ST7789 driver instance
    ///
    /// # Arguments
    ///
    /// * `di` - a display interface for talking with the display
    /// * `rst` - display hard reset pin
    /// * `bl` - backlight pin
    /// * `size_x` - x axis resolution of the display in pixels
    /// * `size_y` - y axis resolution of the display in pixels
    ///
    pub fn new(di: DI, rst: Option<RST>, bl: Option<BL>) -> Self {
        Self {
            di,
            rst,
            bl,
            orientation: Orientation::default(),
        }
    }

    ///
    /// Runs commands to initialize the display
    ///
    /// # Arguments
    ///
    /// * `delay_source` - mutable reference to a delay provider
    ///
    pub fn init(&mut self, delay_source: &mut impl DelayNs) -> Result<(), Error<PinE>> {
        self.hard_reset(delay_source)?;
        if let Some(bl) = self.bl.as_mut() {
            bl.set_low().map_err(Error::Pin)?;
            delay_source.delay_us(10_000);
            bl.set_high().map_err(Error::Pin)?;
        }

        self.write_command(Instruction::SWRESET)?; // reset display
        delay_source.delay_us(150_000);
        self.write_command(Instruction::SLPOUT)?; // turn off sleep
        delay_source.delay_us(10_000);

        /*

            writecommand(0xE0); // Positive Gamma Control
            writedata(0x00);
            writedata(0x03);
            writedata(0x09);
            writedata(0x08);
            writedata(0x16);
            writedata(0x0A);
            writedata(0x3F);
            writedata(0x78);
            writedata(0x4C);
            writedata(0x09);
            writedata(0x0A);
            writedata(0x08);
            writedata(0x16);
            writedata(0x1A);
            writedata(0x0F);

            writecommand(0XE1); // Negative Gamma Control
            writedata(0x00);
            writedata(0x16);
            writedata(0x19);
            writedata(0x03);
            writedata(0x0F);
            writedata(0x05);
            writedata(0x32);
            writedata(0x45);
            writedata(0x46);
            writedata(0x04);
            writedata(0x0E);
            writedata(0x0D);
            writedata(0x35);
            writedata(0x37);
            writedata(0x0F);

            writecommand(0XC0); // Power Control 1
            writedata(0x17);
            writedata(0x15);

            writecommand(0xC1); // Power Control 2
            writedata(0x41);

            writecommand(0xC5); // VCOM Control
            writedata(0x00);
            writedata(0x12);
            writedata(0x80);

            writecommand(TFT_MADCTL); // Memory Access Control
            writedata(0x48);          // MX, BGR

            writecommand(0x3A); // Pixel Interface Format
        #if defined (TFT_PARALLEL_8_BIT) || defined (TFT_PARALLEL_16_BIT) || defined (RPI_DISPLAY_TYPE)
            writedata(0x55);  // 16-bit colour for parallel
        #else
            writedata(0x66);  // 18-bit colour for SPI
        #endif

            writecommand(0xB0); // Interface Mode Control
            writedata(0x00);

            writecommand(0xB1); // Frame Rate Control
            writedata(0xA0);

            writecommand(0xB4); // Display Inversion Control
            writedata(0x02);

            writecommand(0xB6); // Display Function Control
            writedata(0x02);
            writedata(0x02);
            writedata(0x3B);

            writecommand(0xB7); // Entry Mode Set
            writedata(0xC6);

            writecommand(0xF7); // Adjust Control 3
            writedata(0xA9);
            writedata(0x51);
            writedata(0x2C);
            writedata(0x82);

            writecommand(TFT_SLPOUT);  //Exit Sleep
        delay(120);

            writecommand(TFT_DISPON);  //Display on
        delay(25);

                 */

        self.write_command_raw(0xE0)?; // Positive Gamma Control
        self.write_data(&[
            0x00, 0x03, 0x09, 0x08, 0x16, 0x0A, 0x3F, 0x78, 0x4C, 0x09, 0x0A, 0x08, 0x16, 0x1A,
            0x0F,
        ])?;
        self.write_command_raw(0xE1)?; // Negative Gamma Control
        self.write_data(&[
            0x00, 0x16, 0x19, 0x03, 0x0F, 0x05, 0x32, 0x45, 0x46, 0x04, 0x0E, 0x0D, 0x35, 0x37,
            0x0F,
        ])?;
        self.write_command_raw(0xC0)?; // Power Control 1
        self.write_data(&[0x17, 0x15])?;
        self.write_command_raw(0xC1)?; // Power Control 2
        self.write_data(&[0x41])?;
        self.write_command_raw(0xC5)?; // VCOM Control
        self.write_data(&[0x00, 0x12, 0x80])?;
        self.write_command(Instruction::MADCTL)?; // Memory Access Control
        self.write_data(&[0x48])?;
        self.write_command_raw(0x3A)?; // Pixel Interface Format
        self.write_data(&[0x66])?; // 18-bit colour for SPI

        self.write_command_raw(0xB0)?; // Interface Mode Control
        self.write_data(&[0x00])?;

        self.write_command_raw(0xB1)?; // Frame Rate Control
        self.write_data(&[0xA0])?;

        self.write_command_raw(0xB4)?; // Display Inversion Control
        self.write_data(&[0x02])?;

        self.write_command_raw(0xB6)?; // Display Function Control
        self.write_data(&[0x02, 0x02, 0x3B])?;

        self.write_command_raw(0xB7)?; // Entry Mode Set
        self.write_data(&[0xC6])?;

        self.write_command_raw(0xF7)?; // Adjust Control 3
        self.write_data(&[0xA9, 0x51, 0x2C, 0x82])?;

        self.write_command(Instruction::SLPOUT)?; // turn off sleep
        delay_source.delay_us(10_000);
        self.write_command(Instruction::DISPON)?; // turn on display
        delay_source.delay_us(10_000);

        Ok(())
    }

    ///
    /// Performs a hard reset using the RST pin sequence
    ///
    /// # Arguments
    ///
    /// * `delay_source` - mutable reference to a delay provider
    ///
    pub fn hard_reset(&mut self, delay_source: &mut impl DelayNs) -> Result<(), Error<PinE>> {
        if let Some(rst) = self.rst.as_mut() {
            rst.set_high().map_err(Error::Pin)?;
            delay_source.delay_us(100); // ensure the pin change will get registered
            rst.set_low().map_err(Error::Pin)?;
            delay_source.delay_us(100); // ensure the pin change will get registered
            rst.set_high().map_err(Error::Pin)?;
            delay_source.delay_us(100); // ensure the pin change will get registered
        }

        Ok(())
    }

    ///
    /// Returns currently set orientation
    ///
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    ///
    /// Sets display orientation
    ///
    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), Error<PinE>> {
        self.write_command(Instruction::MADCTL)?;
        self.write_data(&[orientation as u8])?;
        self.orientation = orientation;
        Ok(())
    }

    ///
    /// Sets a pixel color at the given coords.
    ///
    /// # Arguments
    ///
    /// * `x` - x coordinate
    /// * `y` - y coordinate
    /// * `color` - the Rgb565 color value
    ///
    pub fn set_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), Error<PinE>> {
        self.set_address_window(x, y, x, y)?;
        self.write_command(Instruction::RAMWR)?;
        self.di
            .send_data(U16BEIter(&mut once(color)))
            .map_err(|_| Error::DisplayError)?;

        Ok(())
    }

    ///
    /// Sets pixel colors in given rectangle bounds.
    ///
    /// # Arguments
    ///
    /// * `sx` - x coordinate start
    /// * `sy` - y coordinate start
    /// * `ex` - x coordinate end
    /// * `ey` - y coordinate end
    /// * `colors` - anything that can provide `IntoIterator<Item = u16>` to iterate over pixel data
    ///
    pub fn set_pixels<T>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: T,
    ) -> Result<(), Error<PinE>>
    where
        T: IntoIterator<Item = u16>,
    {
        self.set_address_window(sx, sy, ex, ey)?;
        self.write_command(Instruction::RAMWR)?;
        self.di
            .send_data(U16BEIter(&mut colors.into_iter()))
            .map_err(|_| Error::DisplayError)
    }

    ///
    /// Sets scroll offset "shifting" the displayed picture
    /// # Arguments
    ///
    /// * `offset` - scroll offset in pixels
    ///
    // pub fn set_scroll_offset(&mut self, offset: u16) -> Result<(), Error<PinE>> {
    //     self.write_command(Instruction::VSCAD)?;
    //     self.write_data(&offset.to_be_bytes())
    // }

    ///
    /// Release resources allocated to this driver back.
    /// This returns the display interface and the RST pin deconstructing the driver.
    ///
    pub fn release(self) -> (DI, Option<RST>, Option<BL>) {
        (self.di, self.rst, self.bl)
    }

    fn write_command(&mut self, command: Instruction) -> Result<(), Error<PinE>> {
        self.di
            .send_commands(U8Iter(&mut once(command as u8)))
            .map_err(|_| Error::DisplayError)?;
        Ok(())
    }

    fn write_command_raw(&mut self, command: u8) -> Result<(), Error<PinE>> {
        self.di
            .send_commands(U8Iter(&mut once(command)))
            .map_err(|_| Error::DisplayError)?;
        Ok(())
    }

    fn write_data(&mut self, data: &[u8]) -> Result<(), Error<PinE>> {
        self.di
            .send_data(U8Iter(&mut data.iter().cloned()))
            .map_err(|_| Error::DisplayError)
    }

    // Sets the address window for the display.
    pub fn set_address_window(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
    ) -> Result<(), Error<PinE>> {
        self.write_command(Instruction::CASET)?;
        self.write_data(&sx.to_be_bytes())?;
        self.write_data(&ex.to_be_bytes())?;
        self.write_command(Instruction::RASET)?;
        self.write_data(&sy.to_be_bytes())?;
        self.write_data(&ey.to_be_bytes())
    }

    //
    // Configures the tearing effect output.
    //
    // pub fn set_tearing_effect(&mut self, tearing_effect: TearingEffect) -> Result<(), Error<PinE>> {
    //     match tearing_effect {
    //         TearingEffect::Off => self.write_command(Instruction::TEOFF),
    //         TearingEffect::Vertical => {
    //             self.write_command(Instruction::TEON)?;
    //             self.write_data(&[0])
    //         }
    //         TearingEffect::HorizontalAndVertical => {
    //             self.write_command(Instruction::TEON)?;
    //             self.write_data(&[1])
    //         }
    //     }
    // }
}

#[derive(Debug)]
pub enum FbWriteError {
    Error,
}
pub trait FramebufferTarget {
    fn eat_framebuffer(&mut self, buf: &[u8]) -> Result<(), FbWriteError>;
}

impl<DI, RST, BL, PinE> FramebufferTarget for ST7789<DI, RST, BL>
where
    DI: WriteOnlyDataCommand,
    RST: OutputPin<Error = PinE>,
    BL: OutputPin<Error = PinE>,
{
    fn eat_framebuffer(&mut self, buf: &[u8]) -> Result<(), FbWriteError> {
        self.write_command(Instruction::RAMWR)
            .map_err(|_| FbWriteError::Error)?;

        self.di
            .send_data(DataFormat::U8(buf))
            .map_err(|_| FbWriteError::Error)
    }
}
