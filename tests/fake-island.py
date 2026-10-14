#!/usr/bin/env python3
"""A scripted island for the relay tests (stdlib only).

Connects to the companion socket as `island`, prints one line per message
it receives (`<type> <id-or-key> <extra>`), and for every card:

  --ack            sends card.shown (without it the daemon must give up
                   after 800 ms)
  --answer A       then answers with action A (allow, deny, always:0,
                   terminal, later …)
  --answers JSON   the question answers to send with --answer answer
  --delay S        seconds between the ack and the answer (default 0.1)
  --env JSON       an env message to send after the welcome

Socket: $EWE_COMPANION_SOCK, else $XDG_RUNTIME_DIR/ewe-companion.sock.
"""
import argparse
import json
import os
import socket
import sys
import threading
import time

ap = argparse.ArgumentParser()
ap.add_argument("--ack", action="store_true")
ap.add_argument("--answer")
ap.add_argument("--answers")
ap.add_argument("--delay", type=float, default=0.1)
ap.add_argument("--env")
a = ap.parse_args()

path = os.environ.get("EWE_COMPANION_SOCK") or os.path.join(os.environ["XDG_RUNTIME_DIR"], "ewe-companion.sock")
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
for _ in range(50):
    try:
        s.connect(path)
        break
    except OSError:
        time.sleep(0.05)
else:
    print("connect-failed", flush=True)
    sys.exit(1)

lock = threading.Lock()


def send(obj):
    obj = {"v": 1, **obj}
    with lock:
        s.sendall((json.dumps(obj) + "\n").encode())


send({"type": "hello", "client": "island", "pid": os.getpid()})
if a.env:
    send({"type": "env", **json.loads(a.env)})


def answer(cid):
    time.sleep(a.delay)
    msg = {"type": "card.answer", "id": cid, "action": a.answer}
    if a.answers:
        msg["answers"] = json.loads(a.answers)
    send(msg)


buf = b""
while True:
    chunk = s.recv(65536)
    if not chunk:
        print("closed", flush=True)
        break
    buf += chunk
    while b"\n" in buf:
        line, buf = buf.split(b"\n", 1)
        m = json.loads(line)
        t = m.get("type")
        if t == "card":
            print("card", m["id"], m.get("kind"), m.get("title"), flush=True)
            if a.ack:
                send({"type": "card.shown", "id": m["id"]})
                if a.answer:
                    threading.Thread(target=answer, args=(m["id"],), daemon=True).start()
        elif t == "card.close":
            print("card.close", m["id"], m.get("reason"), flush=True)
        elif t == "peek":
            print("peek", m.get("key"), m.get("line"), flush=True)
        elif t == "state":
            print("state", m.get("presence"), len(m.get("pending", [])), len(m.get("sessions", [])), flush=True)
        else:
            print(t, flush=True)
