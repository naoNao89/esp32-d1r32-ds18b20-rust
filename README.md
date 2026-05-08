# ESP32 D1 R32 + DS18B20 in Rust (no_std)

Reads a DS18B20 1-Wire temperature sensor on **GPIO14** (D7 silkscreen on this
D1 R32 variant) and prints `temp = XX.XXX C` once per second over UART0
(CH340 USB serial) at **115200 baud**.

## Wiring

| DS18B20 | D1 R32          |
|---------|-----------------|
| VDD     | 3V3             |
| GND     | GND             |
| DQ      | GPIO14 (D7)     |
| DQ↔3V3 | 4.7 kΩ pull-up  |

## One-time toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install espup espflash ldproxy
espup install
echo '. $HOME/export-esp.sh' >> ~/.zshrc
. $HOME/export-esp.sh
```

macOS: install the **CH340/CH341** driver from WCH if `/dev/tty.wchusbserial*`
does not appear.

## Build, flash, monitor

Plug the board in, then:

```bash
. $HOME/export-esp.sh
cargo build --release
espflash flash --port /dev/tty.usbserial-110 \
  target/xtensa-esp32-none-elf/release/esp32-d1r32-ds18b20
python3 -c 'import serial, time
s = serial.Serial("/dev/tty.usbserial-110", 115200, timeout=0.5)
s.setRTS(True); time.sleep(0.1); s.setDTR(False); s.setRTS(False)
while True:
    d = s.read(256)
    if d: print(d.decode("utf-8", errors="replace"), end="", flush=True)'
```

The `cargo run` runner with `espflash --monitor` opens a TUI that does not
play nicely with non-TTY shells; the Python `pyserial` snippet above is a
plain monitor that works everywhere.

Replace `/dev/tty.usbserial-110` with whatever `ls /dev/tty.usbserial-*`
shows on your machine. Install pyserial once via
`python3 -m pip install --break-system-packages pyserial`.

## Change the data pin

Edit `peripherals.GPIO14` in `src/main.rs`. D1 R32 silkscreen → ESP32 GPIO
(this board's variant): **D7 = GPIO14** (confirmed by scan). Other
commonly-mapped pins on D1 R32 clones: D2=26, D3=25, D4=17, D5=16, D6=27,
D8=12. Avoid strapping pins (GPIO0/2/12/15) for the 1-Wire line.

If unsure which silkscreen pin maps where on your specific clone, run a pin
scan: temporarily revert to the scanning `main` (see git history) which
probes GPIO 4/13/14/16/17/18/19/21/22/23/25/26/27 and prints which one has
the sensor.
