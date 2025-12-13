#![no_std]
#![no_main]

use tty::Teletype;

use core::fmt::Write;
use core::panic::PanicInfo;
use embedded_hal::digital::OutputPin;
use rp_pico::{
    entry,
    hal::{
        self,
        fugit::{ExtU32, MicrosDurationU32, RateExtU32},
        timer::Alarm,
        uart::{DataBits, StopBits, UartConfig, UartPeripheral},
        Clock, Sio, Timer,
    },
    pac::{self},
};

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
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (pins.gpio0.into_function(), pins.gpio1.into_function());
    let mut current_loop_write = pins.gpio16.into_push_pull_output();
    let current_loop_read = pins.gpio15.into_pull_down_input();
    let current_loop_read_rev = pins.gpio14.into_pull_down_input();
    let begin_call_write = pins.gpio17.into_push_pull_output();
    let mut led = pins.gpio25.into_push_pull_output();

    let mut uart = UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(300.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    writeln!(uart, "tty start\r").unwrap();

    current_loop_write.set_high().unwrap();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut read_alarm = timer.alarm_0().unwrap();
    let mut write_alarm = timer.alarm_1().unwrap();
    read_alarm.disable_interrupt();
    write_alarm.disable_interrupt();
    read_alarm.schedule(3.millis()).unwrap();
    write_alarm.schedule(3.millis()).unwrap();

    let mut tty = Teletype::new(
        current_loop_read,
        current_loop_read_rev,
        current_loop_write,
        begin_call_write,
    );

    let mut buf = [0; 32]; // this is enough to hold the entire pico uart read buffer

    // main loop
    loop {
        // Connect to peer
        'connection: loop {
            if tty.accept_conn(&mut timer) {
                break;
            }
            let nread = match uart.read_raw(&mut buf) {
                Ok(0) | Err(nb::Error::WouldBlock) => continue,
                Ok(n) => n,
                Err(e) => {
                    _ = write!(uart, "{e:?}");
                    continue;
                }
            };
            let digits = buf[0..nread]
                .iter()
                .filter(|c| c.is_ascii_digit())
                .map(|c| c - b'0');
            for d in digits {
                tty.dial(d, &mut timer);
                if tty.accept_conn(&mut timer) {
                    break 'connection;
                }
            }
        }

        writeln!(uart, "connected\r").unwrap();

        // Talk with peer
        while tty.is_connected() {
            if read_alarm.finished() {
                let (current_loop_state, read, sched) = tty.poll_read();
                _ = led.set_state(current_loop_state.into());
                if let Some(read) = read {
                    if read != 0 {
                        _ = uart.write_raw(&[read]);
                    }
                }
                read_alarm
                    .schedule(sched.unwrap_or(MicrosDurationU32::millis(3)))
                    .unwrap();
            }
            if write_alarm.finished() {
                let sched = tty.poll_write();
                write_alarm
                    .schedule(sched.unwrap_or(MicrosDurationU32::millis(3)))
                    .unwrap();
            }

            let nread = match uart.read_raw(&mut buf) {
                Ok(0) | Err(nb::Error::WouldBlock) => continue, // continue on empty reads
                Ok(n) => n,
                Err(e) => {
                    _ = writeln!(uart, "{e:?}\r");
                    continue;
                }
            };
            // Read end of transmission
            if buf[0..nread].iter().any(|&c| c == 4) {
                break;
            }
            let (str, len) = translate(buf, nread);
            str[0..len].iter().for_each(|&c| {
                tty.queue_write(c);
            });
        }

        writeln!(uart, "disconnected\r").unwrap();
    }
}

/// Translate lone \n sent by the computer to \r\n.
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
    loop {}
}
