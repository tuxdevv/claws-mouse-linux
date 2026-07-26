#!/usr/bin/env python3
"""Linux control tool for the Rajin Blaze Claws mouse (VID 0xa8a5 PID 0x2255).

Usage:
  clawsctl.py info               -- print device model/firmware string
  clawsctl.py dpi                -- print current stage/polling/DPI table
  clawsctl.py dpi <1-6>          -- switch active DPI stage
  clawsctl.py dpi <1-6> <value>  -- set the DPI value for a stage (50-12000)
  clawsctl.py poll <1-4>         -- switch polling rate level
  clawsctl.py status             -- print misc probed values (unconfirmed meaning)
"""
import sys, os, select, glob, struct

VID, PID = "a8a5", "2255"
POLL_HZ = {1: 125, 2: 250, 3: 500, 4: 1000}


def find_vendor_hidraw():
    for path in glob.glob("/sys/class/hidraw/hidraw*"):
        uevent = open(f"{path}/device/uevent").read()
        if f"V0000{VID.upper()}P0000{PID.upper()}" not in uevent.upper():
            continue
        if "input2" in uevent:
            return f"/dev/{os.path.basename(path)}"
    raise SystemExit("vendor HID interface not found (is the receiver plugged in?)")


def transact(dev, payload):
    payload = payload + bytes(64 - len(payload))
    fd = os.open(dev, os.O_RDWR)
    try:
        os.write(fd, payload)
        r, _, _ = select.select([fd], [], [], 1.0)
        return os.read(fd, 64) if r else None
    finally:
        os.close(fd)


def info(dev):
    reply = transact(dev, bytes.fromhex("5503"))
    if not reply:
        raise SystemExit("no reply from device")
    return reply[8:].split(b"\x00")[0].decode(errors="replace")


def get_full(dev):
    reply = transact(dev, bytes.fromhex("550e"))
    if not reply:
        raise SystemExit("no reply from device")
    table = list(struct.unpack("<6H", reply[13:25]))
    return reply[10], reply[12], table


def set_full(dev, polling, stage, table):
    payload = (
        bytes.fromhex("550fae0a2f0101010003")
        + bytes([polling, 0x06, stage])
        + struct.pack("<6H", *table)
        + bytes(24)
        + bytes.fromhex("ff010a00ff")
    )
    reply = transact(dev, payload)
    return bool(reply and reply[0] == 0xAA)


def set_dpi_stage(dev, stage):
    if not 1 <= stage <= 6:
        raise SystemExit("stage must be 1-6")
    polling, _, table = get_full(dev)
    return set_full(dev, polling, stage, table)


def set_dpi_value(dev, stage, value):
    if not 1 <= stage <= 6:
        raise SystemExit("stage must be 1-6")
    if not 50 <= value <= 12000:
        raise SystemExit("DPI value must be 50-12000")
    polling, active_stage, table = get_full(dev)
    table[stage - 1] = value
    return set_full(dev, polling, active_stage, table)


def set_polling(dev, level):
    if not 1 <= level <= 4:
        raise SystemExit("polling level must be 1-4")
    _, stage, table = get_full(dev)
    return set_full(dev, level, stage, table)


def status(dev):
    reply = transact(dev, bytes.fromhex("5550"))
    if reply:
        print(f"0x50 probe value: {reply[4]} (likely polling rate index, unconfirmed)")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    dev = find_vendor_hidraw()
    if sys.argv[1] == "info":
        print(info(dev))
    elif sys.argv[1] == "dpi":
        if len(sys.argv) > 3:
            set_dpi_value(dev, int(sys.argv[2]), int(sys.argv[3]))
            print(f"stage {sys.argv[2]} DPI set to {sys.argv[3]}")
        elif len(sys.argv) > 2:
            set_dpi_stage(dev, int(sys.argv[2]))
            print(f"active stage set to {sys.argv[2]}")
        else:
            polling, stage, table = get_full(dev)
            print(f"active stage: {stage}, polling: {POLL_HZ.get(polling, polling)}")
            for i, v in enumerate(table, 1):
                marker = " <-- active" if i == stage else ""
                print(f"  stage {i}: {v} DPI{marker}")
    elif sys.argv[1] == "poll":
        set_polling(dev, int(sys.argv[2]))
        print(f"polling level set to {sys.argv[2]}")
    elif sys.argv[1] == "status":
        status(dev)
    else:
        print(__doc__)
        sys.exit(1)
