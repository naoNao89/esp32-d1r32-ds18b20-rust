#![no_std]
#![no_main]

//! Wemos D1 R32 (ESP32) + DS18B20 on GPIO14 (D7 on silkscreen).
//! Wiring: VDD->3V3, GND->GND, DQ->GPIO14, 4.7k pull-up DQ<->3V3.
//! Logs stream over UART0 (CH340 USB-serial) at 115200 baud.

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{DriveMode, Flex, InputConfig, OutputConfig, Pull},
    main,
};
use esp_println::println;

// ---------- minimal 1-wire bit-bang ----------

struct OneWire<'a> {
    pin: Flex<'a>,
    d: Delay,
}

#[derive(Debug)]
enum OwError {
    NoPresence,
    CrcMismatch,
}

impl<'a> OneWire<'a> {
    fn new(mut pin: Flex<'a>, d: Delay) -> Self {
        pin.apply_input_config(&InputConfig::default().with_pull(Pull::Up));
        pin.apply_output_config(
            &OutputConfig::default()
                .with_drive_mode(DriveMode::OpenDrain)
                .with_pull(Pull::Up),
        );
        pin.set_input_enable(true);
        pin.set_output_enable(true);
        pin.set_high();
        Self { pin, d }
    }

    fn write_bit(&mut self, bit: bool) {
        self.pin.set_low();
        if bit {
            self.d.delay_micros(6);
            self.pin.set_high();
            self.d.delay_micros(64);
        } else {
            self.d.delay_micros(60);
            self.pin.set_high();
            self.d.delay_micros(10);
        }
    }

    fn read_bit(&mut self) -> bool {
        self.pin.set_low();
        self.d.delay_micros(6);
        self.pin.set_high();
        self.d.delay_micros(9);
        let b = self.pin.is_high();
        self.d.delay_micros(55);
        b
    }

    fn write_byte(&mut self, mut byte: u8) {
        for _ in 0..8 {
            self.write_bit(byte & 1 == 1);
            byte >>= 1;
        }
    }

    fn read_byte(&mut self) -> u8 {
        let mut b = 0u8;
        for i in 0..8 {
            if self.read_bit() {
                b |= 1 << i;
            }
        }
        b
    }

    fn reset(&mut self) -> Result<(), OwError> {
        self.pin.set_low();
        self.d.delay_micros(480);
        self.pin.set_high();
        self.d.delay_micros(70);
        let presence = self.pin.is_low();
        self.d.delay_micros(410);
        if presence { Ok(()) } else { Err(OwError::NoPresence) }
    }
}

fn crc8(data: &[u8]) -> u8 {
    let mut crc = 0u8;
    for &b in data {
        let mut x = b;
        for _ in 0..8 {
            let mix = (crc ^ x) & 0x01;
            crc >>= 1;
            if mix != 0 {
                crc ^= 0x8C;
            }
            x >>= 1;
        }
    }
    crc
}

// ---------- DS18B20 ----------

const SKIP_ROM: u8 = 0xCC;
const CONVERT_T: u8 = 0x44;
const READ_SCRATCHPAD: u8 = 0xBE;

fn ds18b20_read_celsius(ow: &mut OneWire) -> Result<f32, OwError> {
    ow.reset()?;
    ow.write_byte(SKIP_ROM);
    ow.write_byte(CONVERT_T);
    // 12-bit conversion can take up to 750 ms
    ow.d.delay_millis(800);

    ow.reset()?;
    ow.write_byte(SKIP_ROM);
    ow.write_byte(READ_SCRATCHPAD);

    let mut scratch = [0u8; 9];
    for s in scratch.iter_mut() {
        *s = ow.read_byte();
    }
    if crc8(&scratch[..8]) != scratch[8] {
        return Err(OwError::CrcMismatch);
    }
    let raw = i16::from_le_bytes([scratch[0], scratch[1]]);
    Ok(raw as f32 / 16.0)
}

// ---------- entry ----------

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_println::logger::init_logger_from_env();

    let delay = Delay::new();
    let dq = Flex::new(peripherals.GPIO14);
    let mut ow = OneWire::new(dq, delay);

    println!("D1 R32 + DS18B20 on GPIO14 (D7) starting...");

    loop {
        match ds18b20_read_celsius(&mut ow) {
            Ok(c) => println!("temp = {:.3} C", c),
            Err(e) => println!("read error: {:?}", e),
        }
        delay.delay_millis(1000);
    }
}
