//! # GPIO 'Blinky' Example
//!
//! This application demonstrates how to control a GPIO pin on the RP2040.
//!
//! It may need to be adapted to your particular board layout and/or pin assignment.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]
extern crate panic_halt;
extern crate embedded_hal;
extern crate rp2040_hal;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Alias for our HAL crate
use rp2040_hal as hal;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use hal::pac;

// Some traits we need
use embedded_hal::{digital::v2::{InputPin, OutputPin}, PwmPin};
use rp2040_hal::clocks::Clock;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
/// Note: This boot block is not necessary when using a rp-hal based BSP
/// as the BSPs already perform this step.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

/// The minimum PWM value (i.e. LED brightness) we want
const LOW: u16 = 0;

/// The maximum PWM value (i.e. LED brightness) we want
const HIGH: u16 = 25000;

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Entry point to our bare-metal application.
///
/// The `#[rp2040_hal::entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables and the spinlock are initialised.
///
/// The function configures the RP2040 peripherals, then toggles a GPIO pin in
/// an infinite loop. If there is an LED connected to that pin, it will blink.
#[rp2040_hal::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // Configure GPIO 0 as an output
    let mut out_pin = pins.gpio0.into_push_pull_output();

    // Init PWMs
    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    // Configure PWM4
    let pwm = &mut pwm_slices.pwm4;
    pwm.set_ph_correct();
    pwm.enable();

    // Output channel B on PWM4 to GPIO 25
    let channel = &mut pwm.channel_b;
    channel.output_to(pins.gpio25);

    // Configure GPIO 4 as an input
    let in_pin = pins.gpio4.into_pull_down_input();
    let mut toggle_button = false;

    // Output is the opposite of the input
    loop {
        // in_pin.is_high() = button is pressed down
        if in_pin.is_high().unwrap() {
            toggle_button = true;
        }
        if toggle_button {
            for _ in 0..5 {
                out_pin.set_high().unwrap();
                delay.delay_ms(400);
                out_pin.set_low().unwrap();
                delay.delay_ms(400);
            }
            toggle_button = false;
        } else {
            out_pin.set_high().unwrap();
            let duty_step = channel.get_max_duty() / HIGH;
            // Ramp brightness up
            for i in LOW..=HIGH {
                delay.delay_us(10);
                channel.set_duty(duty_step * i);
            }

            // Ramp brightness down
            for i in (LOW..=HIGH).rev() {
                delay.delay_us(10);
                channel.set_duty(duty_step * i);
            }

            delay.delay_ms(500);
        }
    }
}

// End of file
