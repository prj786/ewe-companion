#!/usr/bin/env python3
"""A virtual mouse for ewe's nested harness (zwlr_virtual_pointer_v1).

The headless nested Hyprland (driver.sh up with HS_HEADLESS=1) has no
pointer device, so nothing ever gets a hover or a click. This client adds
one, with the standard library only, and plays a script of steps. Use it
through `tests/island.sh hover <steps-file>`, which moves the cursor and
takes the shots (docs/TESTING.md); on its own:

    tests/vptr.py <wayland-socket> <steps-file>

The socket is $XDG_RUNTIME_DIR/$NEST_WD (from the driver's state file).
Steps, one per line:
    warp X Y        prints "WARP X Y" and waits for a line on stdin: the
                    caller moves the cursor (driver.sh hc dispatch
                    'hl.dsp.cursor.move({ x = X, y = Y })')
    nudge           a 1 px motion and back: a real motion event, so the
                    surface under the cursor gets its hover
    click           left button press + release
    wheel N         N wheel notches (positive scrolls down)
    sleep S
    mark NAME       prints "MARK NAME" and waits for a line on stdin (the
                    caller takes a shot)
A "#" starts a comment, to the end of the line.
Found the notch's hover bug on 2026-10-14. Never point it at the live
session's socket: it would drive the real pointer.
"""
import os
import socket
import struct
import sys
import time

sock_path = sys.argv[1]
# a step per line; "#" starts a comment
steps = [w for w in (l.split("#", 1)[0].split() for l in open(sys.argv[2])) if w]
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(sock_path)
next_id = 2


def send(obj, op, payload=b""):
    s.sendall(struct.pack("<II", obj, ((8 + len(payload)) << 16) | op) + payload)


def wl_string(t):
    b = t.encode() + b"\0"
    return struct.pack("<I", len(b)) + b + b"\0" * ((4 - len(b) % 4) % 4)


def read_events(until_callback):
    """Read events until the wl_callback `until_callback` is done; return globals."""
    globs = {}
    buf = b""
    while True:
        while len(buf) < 8:
            buf += s.recv(65536)
        obj, word = struct.unpack("<II", buf[:8])
        size, op = word >> 16, word & 0xFFFF
        while len(buf) < size:
            buf += s.recv(65536)
        body, buf = buf[8:size], buf[size:]
        if obj == 2 and op == 0:  # wl_registry.global
            name, ln = struct.unpack("<II", body[:8])
            iface = body[8:8 + ln - 1].decode()
            off = 8 + ln + ((4 - ln % 4) % 4)
            (ver,) = struct.unpack("<I", body[off:off + 4])
            globs[iface] = (name, ver)
        if obj == until_callback and op == 0:
            return globs


send(1, 1, struct.pack("<I", 2))  # wl_display.get_registry -> 2
send(1, 0, struct.pack("<I", 3))  # wl_display.sync -> 3
globs = read_events(3)
name, ver = globs["zwlr_virtual_pointer_manager_v1"]
send(2, 0, struct.pack("<I", name) + wl_string("zwlr_virtual_pointer_manager_v1") + struct.pack("<II", min(ver, 2), 4))
send(4, 0, struct.pack("<II", 0, 5))  # create_virtual_pointer(seat=null) -> 5
send(1, 0, struct.pack("<I", 6))
read_events(6)
t0 = time.monotonic()


def now():
    return int((time.monotonic() - t0) * 1000) & 0xFFFFFFFF


def fixed(v):
    return int(v * 256)


def motion(dx, dy):
    send(5, 0, struct.pack("<Iii", now(), fixed(dx), fixed(dy)))
    send(5, 4)  # frame


def wheel(steps):
    # axis_source wheel, then axis + axis_discrete per notch (positive = down)
    for _ in range(abs(steps)):
        send(5, 5, struct.pack("<I", 0))  # axis_source(wheel)
        # axis_discrete(time, axis=vertical, value, discrete)
        send(5, 7, struct.pack("<IIii", now(), 0, fixed(15 if steps > 0 else -15), 1 if steps > 0 else -1))
        send(5, 4)
        time.sleep(0.03)


def button(state):
    send(5, 2, struct.pack("<III", now(), 0x110, state))  # BTN_LEFT
    send(5, 4)


time.sleep(0.5)  # let the seat gain its pointer
for st in steps:
    cmd = st[0]
    if cmd in ("warp", "mark"):
        print(cmd.upper(), *st[1:], flush=True)
        sys.stdin.readline()
    elif cmd == "nudge":
        motion(1, 0)
        time.sleep(0.05)
        motion(-1, 0)
    elif cmd == "click":
        button(1)
        time.sleep(0.05)
        button(0)
    elif cmd == "wheel":
        wheel(int(st[1]))
    elif cmd == "sleep":
        time.sleep(float(st[1]))
s.close()
