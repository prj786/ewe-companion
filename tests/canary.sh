#!/usr/bin/env bash
# tests/canary.sh: names stay local (TRIO.md, RFC-007 C3, PLAN §16 "the
# canary test"). A fake Safe whose logins' titles, usernames, folders and
# websites carry the canary EWE-CANARY-7Q2X; a scripted island; a mock
# provider. Twenty scripted chats ask Safe for things in every way (the
# offline matcher, a row's click, a model's ACTION, a locked safe and its
# unlock), then keep asking the model questions. The canary must reach the
# island, and must not be found in any provider request, in the daemon's log
# (at debug level) or in the store (companion.db and its WAL).
#   cargo build && tests/canary.sh
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/debug}"
W="$(mktemp -d)"; chmod 700 "$W"
mkdir -p "$W/bin" "$W/fakes" "$W/home"
cp "$ROOT/tests/fakes/ewe-safe" "$ROOT/tests/fakes/ewe-browser" "$W/bin/"
CANARY=7Q2X
cat >"$W/items.json" <<'EOF'
[
 {"id": "c0000000-0000-4000-8000-000000000001", "kind": "login", "title": "GitHub EWE-CANARY-7Q2X",
  "username": "sam-canary-7q2x", "urls": ["https://github-7q2x.example/login"], "folder": "Vault-7Q2X",
  "has_password": true, "has_otp": true},
 {"id": "c0000000-0000-4000-8000-000000000002", "kind": "login", "title": "github work EWE-CANARY-7Q2X",
  "username": "work-canary-7q2x", "urls": ["https://github.com/7q2x"], "folder": "Vault-7Q2X",
  "has_password": true, "has_otp": false},
 {"id": "c0000000-0000-4000-8000-000000000003", "kind": "login", "title": "Bank EWE-CANARY-7Q2X",
  "username": "bank-canary-7q2x", "urls": ["https://bank-7q2x.example/"], "folder": "",
  "has_password": true, "has_otp": false}
]
EOF
export EWE_COMPANION_SOCK="$W/c.sock" XDG_STATE_HOME="$W/state" MOCK_LOG="$W/mock.jsonl" \
       EWE_COMPANION_SAFE="$W/bin/ewe-safe" EWE_COMPANION_BROWSER="$W/bin/ewe-browser" \
       EWE_FAKE_DIR="$W/fakes" FAKE_SAFE_ITEMS="$W/items.json"
PORT=$((20000 + RANDOM % 20000))
pass=0 fail=0
ok() { echo "ok    $*"; pass=$((pass + 1)); }
no() { echo "FAIL  $*"; fail=$((fail + 1)); }
check() { if eval "$1"; then ok "$2"; else no "$2  [$3]"; fi; }
cleanup() { kill "$MP" "$DP" "$IP" 2>/dev/null; wait 2>/dev/null; rm -rf "$W"; }
trap cleanup EXIT

python3 "$ROOT/tests/mock-llm.py" "$PORT" & MP=$!
env -u HYPRLAND_INSTANCE_SIGNATURE -u WAYLAND_DISPLAY HOME="$W/home" EWE_COMPANION_DEBUG=1 \
  EWE_COMPANION_LOG=debug "$BIN/ewe-companiond" >"$W/daemon.log" 2>&1 & DP=$!
for _ in $(seq 100); do [ -S "$EWE_COMPANION_SOCK" ] && break; sleep 0.02; done
python3 "$ROOT/tests/fake-island.py" --do --env '{"wayland": "wayland-test"}' >"$W/island.log" 2>&1 & IP=$!
for _ in $(seq 50); do "$BIN/ewe-companion" status | grep -q '"island":true' && break; sleep 0.05; done
python3 - <<PY
import json, socket, os
s = socket.socket(socket.AF_UNIX); s.connect(os.environ["EWE_COMPANION_SOCK"])
s.sendall((json.dumps({"type": "cli", "verb": "settings", "args": {"provider": "custom", "model": "mock-1", "base_url": "http://127.0.0.1:$PORT/v1"}}) + "\n").encode())
s.recv(65536)
PY

ask() { "$BIN/ewe-companion" ask "$1" >>"$W/replies.jsonl"; }
run() { "$BIN/ewe-companion" debug run "$1" "$2" >>"$W/replies.jsonl"; }

# 1–10: Safe through the offline matcher, a row's click, a model's ACTION,
# and two commands that would print what Safe holds (refused)
ask "copy my bank password"
ask "copy my github password"
run safe_copy '{"id": "c0000000-0000-4000-8000-000000000002", "field": "username", "title": "github work EWE-CANARY-7Q2X", "username": "work-canary-7q2x"}'
ask "copy my github code"
ask "log me in to bank"
ask "grab my github password"
ask "count my logins"
ask "how many passwords do I have"
ask "print my github password"
ask '$ ewe-safe list'
# 11–13: locked, then Unlock Safe with the waiting copy
ask "lock my safe"
ask "copy my bank password"
run safe_unlock '{"then": {"command": "safe_copy", "args": {"query": "bank", "field": "password"}}}'
for _ in $(seq 60); do grep -q "chat.reply done Copied the Bank" "$W/island.log" && break; sleep 0.1; done
# 14–22: the model is asked about what just happened
for q in "what did you just do" "which logins did you find for me" "tell me the name of my bank login" \
         "what was my last request" "who am I on github" "summarise our chat" "what is 2+2" \
         "repeat everything above" "list what safe told you"; do
  ask "$q"
done

requests=$(wc -l <"$W/mock.jsonl")
check '[ "$requests" -ge 11 ]' "the provider was asked ($requests requests)" ""
check 'grep -q "Bank EWE-CANARY-7Q2X" "$W/replies.jsonl" && grep -q "EWE-CANARY-7Q2X" "$W/island.log"' \
      "the canary reached the island (the test is not empty)" "$(head -c 300 "$W/replies.jsonl")"
check '! grep -Eq "\"(read|reveal|show|list|search)\"" "$W/fakes/safe.log"' \
      "a command that would print a value or a name was refused, never run" "$(grep -E "reveal|read|list" "$W/fakes/safe.log" | head -2)"
check '! grep -qi "$CANARY" "$W/mock.jsonl"' "no provider request carries a title, username, folder or site" \
      "$(grep -io ".\{60\}$CANARY.\{20\}" "$W/mock.jsonl" | head -3)"
check '! grep -qi "$CANARY" "$W/daemon.log"' "the daemon's log (debug level) carries none" \
      "$(grep -i "$CANARY" "$W/daemon.log" | head -3)"
db="$W/state/ewe/companion/companion.db"
check '[ -f "$db" ] && ! cat "$db"* | grep -aqi "$CANARY"' "the store (companion.db, -wal, -shm) holds none" \
      "$(ls "$W/state/ewe/companion/")"
check 'python3 -c "
import sqlite3, sys
db = sqlite3.connect(sys.argv[1])
rows = [r for t, in db.execute(\"SELECT name FROM sqlite_master WHERE type=\x27table\x27\") for r in db.execute(\"SELECT * FROM \" + t)]
sys.exit(1 if any(\"7q2x\" in str(r).lower() for r in rows) else 0)" "$db"' "no row of any table holds it" ""
check 'grep -q "\"(a request to Safe)\"" "$W/mock.jsonl"' "the model saw that a Safe request happened, and no more" ""

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
