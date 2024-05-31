
use display_interface_spi::SPIInterface;

use esp_idf_hal::{
    delay::Ets,
    gpio::{InputPin, Output, OutputPin, PinDriver},
    ledc::{LedcChannel, LedcTimer},
    peripheral::Peripheral,
    prelude::*,
    spi::{Dma, SpiAnyPins, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
};

use esp_idf_hal::peripherals;
use log::info;

use crate::display_driver::FramebufferTarget;

/*

#define TFT_WIDTH  320
#define TFT_HEIGHT 480

#define TFT_MISO 18
#define TFT_MOSI 7
#define TFT_SCLK 10
#define TFT_CS   9  // Chip select control pin
#define TFT_DC   8  // Data Command control pin
#define TFT_RST  19

*/

mod display_driver;

#[allow(clippy::too_many_arguments)]
pub fn prepare_display<SPI: SpiAnyPins>(
    spi: impl Peripheral<P = SPI> + 'static,
    sdo: impl Peripheral<P = impl OutputPin> + 'static,
    sdi: Option<impl Peripheral<P = impl InputPin> + 'static>,
    sclk: impl Peripheral<P = impl OutputPin> + 'static,
    cs: Option<impl Peripheral<P = impl OutputPin> + 'static>,
    rst: impl Peripheral<P = impl OutputPin> + 'static,
    dc: impl Peripheral<P = impl OutputPin> + 'static,
    bl: impl Peripheral<P = impl OutputPin> + 'static,
    ledc_timer: impl Peripheral<P = impl LedcTimer> + 'static,
    ledc_channel: impl Peripheral<P = impl LedcChannel> + 'static,
) -> display_driver::ST7789<
    SPIInterface<
        SpiDeviceDriver<'static, SpiDriver<'static>>,
        PinDriver<'static, impl OutputPin, Output>,
    >,
    esp_idf_hal::gpio::PinDriver<'static, impl OutputPin, esp_idf_hal::gpio::Output>,
    esp_idf_hal::gpio::PinDriver<'static, impl OutputPin, esp_idf_hal::gpio::Output>,
> {
    let config = esp_idf_hal::spi::config::Config::new()
        .baudrate(20.MHz().into())
        .data_mode(esp_idf_hal::spi::config::MODE_0)
        .queue_size(1);
    let device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        sdi,
        cs,
        &SpiDriverConfig::new().dma(Dma::Auto(4096)),
        &config,
    )
    .unwrap();

    let pin_dc = PinDriver::output(dc).unwrap();

    let spi_interface = SPIInterface::new(device, pin_dc);

    // let ledc_config = esp_idf_svc::hal::ledc::config::TimerConfig::new().frequency(25.kHz().into());
    // let timer = LedcTimerDriver::new(ledc_timer, &ledc_config).unwrap();

    // let backlight_pwm = LedcDriver::new(ledc_channel, timer, bl).unwrap();
    // backlight_pwm.set_duty(backlight_pwm.get_max_duty()).unwrap();

    let rst_pin = PinDriver::output(rst).unwrap();
    let bl_pin = PinDriver::output(bl).unwrap();

    display_driver::ST7789::new(spi_interface, Some(rst_pin), Some(bl_pin))
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Initialize the peripherals
    let peripherals = peripherals::Peripherals::take().unwrap();

    log::info!("hello");

    let mut display = prepare_display(
        peripherals.spi2,
        peripherals.pins.gpio13,
        Some(peripherals.pins.gpio1),
        peripherals.pins.gpio14,
        Some(peripherals.pins.gpio10),
        peripherals.pins.gpio11,
        peripherals.pins.gpio12,
        peripherals.pins.gpio15,
        peripherals.ledc.timer0,
        peripherals.ledc.channel0,
    );

    info!("Display prepared");

    let mut delay = Ets;

    display.hard_reset(&mut delay).unwrap();
    display.init(&mut delay).unwrap();

    log::info!("Display initialized");

    const W: usize = 60;
    const H: usize = 60;

    display
        .set_address_window(10, 10, 9 + W as u16, 10 + H as u16)
        .unwrap();

    let mut raw_framebuffer_0 = [0u8; W * H * 3];

    for (x, y) in (0..W).flat_map(|x| (0..H).map(move |y| (x, y))) {
        let i = y * W + x;

        match x {
            0..=20 => {
                raw_framebuffer_0[i * 3] = 0;
                raw_framebuffer_0[i * 3 + 1] = 255;
                raw_framebuffer_0[i * 3 + 2] = 0;
            }
            21..=40 => {
                raw_framebuffer_0[i * 3] = 255;
                raw_framebuffer_0[i * 3 + 1] = 255;
                raw_framebuffer_0[i * 3 + 2] = 255;
            }
            41..=60 => {
                raw_framebuffer_0[i * 3] = 255;
                raw_framebuffer_0[i * 3 + 1] = 0;
                raw_framebuffer_0[i * 3 + 2] = 0;
            }
            _ => {
                raw_framebuffer_0[i * 3] = 255;
                raw_framebuffer_0[i * 3 + 1] = 255;
                raw_framebuffer_0[i * 3 + 2] = 255;
            }
        }
    }

    loop {
        display
            .eat_framebuffer(raw_framebuffer_0.as_slice())
            .unwrap();
    }
}
