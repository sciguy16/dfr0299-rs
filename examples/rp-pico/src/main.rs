#![cfg_attr(not(test), no_std)]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use dfr0299::Command;
use embedded_hal::digital::v2::OutputPin;
use embedded_time::duration::*;
use panic_halt as _;
use rp_pico::entry;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::Clock;

#[entry]
fn main() -> ! {
    info!("Start boot");

    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

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

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("Init SPI");

    let mut delay = cortex_m::delay::Delay::new(
        core.SYST,
        clocks.system_clock.freq().integer(),
    );

    let mut led = pins.led.into_push_pull_output();

    let uart_pins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        pins.gpio0.into_mode::<hal::gpio::FunctionUart>(),
        // UART RX (characters received by RP2040) on pin 2 (GPIO1)
        pins.gpio1.into_mode::<hal::gpio::FunctionUart>(),
    );

    let uart =
        hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
            .enable(
                hal::uart::common_configs::_9600_8_N_1,
                clocks.peripheral_clock.into(),
            )
            .unwrap();

    let mut buf = [0u8; 10];
    info!("Send RESET");
    Command::Reset.serialise(&mut buf).unwrap();
    uart.write_full_blocking(&buf);
    delay.delay_ms(500);

    info!("Send TRACK 1");
    Command::Track(1).serialise(&mut buf).unwrap();
    uart.write_full_blocking(&mut buf);

    loop {
        led.set_high().unwrap();

        led.set_low().unwrap();

        delay.delay_ms(500);
    }
}
