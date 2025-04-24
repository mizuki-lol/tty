#![no_std]
#![no_main]

mod baudot;

use baudot::BaudotStream;
use embedded_hal::digital::OutputPin;
// use panic_halt as _;
use core::panic::PanicInfo;
use rp2040_hal::{
    fugit::MicrosDurationU32,
    gpio::{bank0::Gpio10, FunctionSioOutput, Pin, PullDown},
    pac,
    timer::Alarm,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    Clock, Sio, Timer,
};
use rp_pico::hal::fugit::{ExtU32, RateExtU32};
use rp_pico::{entry, hal};

static mut ERR: Option<Pin<Gpio10, FunctionSioOutput, PullDown>> = None;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
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

    let uart_pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
    let mut current_loop_write = pins.gpio15.into_push_pull_output();
    let current_loop_read = pins.gpio13.into_pull_down_input();
    let err_pin = pins.gpio10.into_push_pull_output();
    unsafe {
        ERR = Some(err_pin);
    }
    let mut led = pins.gpio25.into_push_pull_output();

    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(300.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    uart.write_full_blocking(b"Hello World!\r\n");

    current_loop_write.set_high().unwrap();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut alarm = timer.alarm_0().unwrap();
    alarm.disable_interrupt();
    alarm.schedule(10.millis()).unwrap();

    let mut stream = BaudotStream::new(current_loop_read, current_loop_write);

    loop {
        if alarm.finished() {
            let schedule1 = stream.poll_write();
            let (current_loop_state, read, schedule2) = stream.poll_read();
            _ = led.set_state(current_loop_state.into());
            if let Some(read) = read {
                _ = uart.write_raw(&[read]);
            }
            alarm
                .schedule(match (schedule1, schedule2) {
                    (Some(s1), Some(s2)) => s1.max(s2),
                    (Some(s), None) | (None, Some(s)) => s,
                    (None, None) => MicrosDurationU32::millis(1),
                })
                .unwrap();
        }
        let mut buf = [0; 32]; // this is enough to hold the entire pico uart read buffer
        let nread = match uart.read_raw(&mut buf) {
            Ok(0) => continue, // continue on empty reads
            Ok(n) => n,
            Err(_) => continue, // silently continue on errors, (maybe fixme)
        };
        let (str, len) = translate(buf, nread);
        str[0..len].iter().for_each(|&c| stream.queue_write(c));
    }
}

fn translate(string: [u8; 32], strlen: usize) -> ([u8; 64], usize) {
    let mut output = [0; 64];
    let mut offset = 0;
    for (idx, c) in string[..strlen].iter().enumerate() {
        match c {
            b'\n' | b'\r' => {
                if idx >= 1
                    && (string[idx - 1] == b'\r' && *c == b'\n'
                        || string[idx - 1] == b'\n' && *c == b'\r')
                {
                    continue;
                }
                output[idx + offset] = b'\r';
                offset += 1;
                output[idx + offset] = b'\n';
            }
            c => output[idx + offset] = *c,
        }
    }
    (output, strlen + offset)
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(mut e) = unsafe { ERR.take() } {
        _ = e.set_high();
    }
    loop {}
}
