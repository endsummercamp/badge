use std::ffi::c_void;

use display_interface_spi::SPIInterface;

use embedded_gfx::framebuffer::DmaReadyFramebuffer;
use embedded_graphics::pixelcolor::Rgb565;
use esp_idf_hal::{
    delay::Ets,
    gpio::{self, IOPin, InputPin, Output, OutputPin, PinDriver},
    ledc::{self, LedcChannel, LedcTimer},
    peripheral::Peripheral,
    prelude::*,
    spi::{self, Dma, SpiAnyPins, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
};


use esp_idf_hal::peripherals;
use embedded_graphics::draw_target::DrawTarget;
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

    let mut display = prepare_display(
        peripherals.spi2,
        peripherals.pins.gpio3,
        Some(peripherals.pins.gpio10),
        peripherals.pins.gpio2,
        Some(peripherals.pins.gpio7),
        peripherals.pins.gpio5,
        peripherals.pins.gpio8,
        peripherals.pins.gpio4,
        peripherals.ledc.timer0,
        peripherals.ledc.channel0,
    );

    info!("Display prepared");
    
    let mut delay = Ets;

    display.hard_reset(&mut delay).unwrap();
    display.init(&mut delay).unwrap();


    log::info!("Display initialized");

    // display.set_pixels(100, 100, 150, 150, vec![0, 50*50]).unwrap();

    // display.set_pixel(10, 10, 0).unwrap();

    // loop {
    //     display.set_pixel(10, 10, 0).unwrap();

    //     esp_idf_hal::delay::Ets::delay_ms(100);

    //     display.set_pixel(10, 10, 65535).unwrap();

    //     esp_idf_hal::delay::Ets::delay_ms(100);
    
    //     info!("Looping");
    //     }


    let mut raw_framebuffer_0 = [0u16; 32 * 48];


    let as_ptr = raw_framebuffer_0.as_mut_ptr();

    display.set_address_window(100, 100, 132, 148).unwrap();


    let mut fbuf = DmaReadyFramebuffer::<32, 48>::new(as_ptr as *mut c_void, true);

    fbuf.clear(Rgb565::new(0b111111, 0b111111, 0b111111)).unwrap();

    display.eat_framebuffer(fbuf.as_slice()).unwrap();

    log::info!("Hello, world!");
}
