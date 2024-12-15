#![no_std]
#![no_main]

// The macro for our start-up function
use rp_pico::entry;

use rp_pico::hal;

use fugit::RateExtU32;
use panic_halt as _;
use rp2040_hal::{
    pac,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    Clock, Sio,
};

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then echoes any characters
/// received over USB Serial.
#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = rp2040_hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    // let uart_config = UartConfig::new();
    // Set up UART on GP0 and GP1 (Pico pins 1 and 2)
    let pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
    // Need to perform clock init before using UART or it will freeze.
    let uart = UartPeripheral::new(pac.UART0, pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(55.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    uart.write_full_blocking(b"Hello World!\r\n");

    loop {}
}
