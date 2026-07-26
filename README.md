# Claws Mouse Control

Linux control app for the **Rajin Blaze Claws Gaming Suite** mouse (YJX-CHIP
2.4GHz receiver, USB VID `0xa8a5` / PID `0x2255`). The official software is
Windows-only (Tauri + WebView2, using the browser's WebHID API), which doesn't
work on Linux since WebKitGTK has no WebHID support. This is a from-scratch
Linux driver built by reverse-engineering the USB protocol from a packet
capture of the official app, talking to the mouse directly over `hidraw`.

## What's here

- **`claws-app/`** — a Tauri 2 desktop app (Rust backend + HTML/CSS/JS
  frontend) with a GUI for DPI and polling rate control.
- **`clawsctl.py`** — a dependency-free Python CLI doing the same thing, for
  scripting or headless use.

## Usage

### GUI

```
cd claws-app
cargo tauri build
./src-tauri/target/release/app
```

### CLI

```
python3 clawsctl.py info
python3 clawsctl.py dpi
python3 clawsctl.py dpi 3
python3 clawsctl.py dpi 3 1600
python3 clawsctl.py poll 4
```

### Permissions

Both need read/write access to the vendor `hidraw` interface. Add a udev rule
so it doesn't require root:

```
# /etc/udev/rules.d/99-claws-mouse.rules
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="a8a5", ATTRS{idProduct}=="2255", MODE="0660", GROUP="wheel"
```

Reload udev and replug the receiver:

```
sudo udevadm control --reload-rules
```

## Protocol notes

The receiver exposes three HID interfaces: a standard boot mouse, a standard
boot keyboard, and a vendor-specific interface (interface 2, 64-byte IN/OUT
reports, no report ID) used for configuration. All control traffic goes
through that third interface.

- Host→device commands are prefixed `0x55`, device→host replies `0xaa`, with
  the command byte echoed back at offset 1.
- `0x03` — get device model/firmware string.
- `0x04` — get serial number (UTF-16LE).
- `0x0e` — get the active profile: `byte[10]` = polling rate level (1-4 →
  125/250/500/1000 Hz), `byte[12]` = active DPI stage (1-6), `bytes[13:25]` =
  the 6-stage DPI table as little-endian `uint16` raw DPI values.
- `0x0f` — set the active profile; same 64-byte layout as the `0x0e` reply.
  Every field must be sent even to change just one, so read the current
  profile first and only change the field you want.
- Physical DPI-button presses emit an unsolicited `0xaa 0xfa 0xae ...`
  notification on the same interface with the new stage at offset 9.

Battery level and per-key/macro remapping commands weren't identified — a
range scan of unknown command IDs against the DPI-set command (`0x0f`) sent
with malformed/zeroed parameters can put the sensor in a bad state (movement
stops, clicks keep working) that only clears on a power cycle. Reads (`0x5X`
range probes with no side effects observed) seem safe; blind writes to
undocumented commands don't.
