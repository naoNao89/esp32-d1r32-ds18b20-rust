#!/usr/bin/env bash
# Build, flash, and monitor the D1 R32 over CH340 at 115200.
# Override PORT env var if your /dev/tty.usbserial-* differs.
set -euo pipefail

PORT="${PORT:-/dev/tty.usbserial-110}"
BIN="target/xtensa-esp32-none-elf/release/esp32-d1r32-ds18b20"

# shellcheck disable=SC1091
. "$HOME/export-esp.sh"

cargo build --release
espflash flash --port "$PORT" "$BIN"

# Ensure pyserial is importable for the monitor below
if ! python3 -c 'import serial' 2>/dev/null; then
  echo "--- installing pyserial ---"
  python3 -m pip install --quiet --break-system-packages pyserial \
    || python3 -m pip install --quiet --user pyserial
fi

exec python3 - "$PORT" <<'PY'
import sys, serial, time
port = sys.argv[1]
s = serial.Serial(port, 115200, timeout=0.5)
s.setRTS(True); time.sleep(0.1); s.setDTR(False); s.setRTS(False)
print(f"--- monitoring {port} @115200, Ctrl-C to quit ---", flush=True)
try:
    while True:
        d = s.read(256)
        if d:
            print(d.decode("utf-8", errors="replace"), end="", flush=True)
except KeyboardInterrupt:
    pass
PY
