#!/usr/bin/env bash
# tests/brain.sh: how Shaun thinks, end to end, against tests/mock-llm.py and
# a fake Claude Code (tests/fakes/claude): answers stream to an island that
# draws them (and not to one that does not), a look-up runs its reads and
# asks again, a reply with several ACTION lines runs them all with one Undo,
# two plain commands in one line run offline, the turn's notes (the time
# included) ride with the question while the system prompt stays the same,
# and Claude Code runs lean and warm: started ahead of the question (after an
# answer, and on the chat field's focus), so the question does not wait for
# it to start. Every desktop tool is a PATH shim from tests/fakes/: never the
# owner's real shell, radios or Claude login.
#   cargo build && tests/brain.sh
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/debug}"
W="$(mktemp -d)"; chmod 700 "$W"
mkdir -p "$W/bin" "$W/fakes" "$W/home" && cp "$ROOT"/tests/fakes/* "$W/bin/"
export EWE_COMPANION_SOCK="$W/c.sock" XDG_STATE_HOME="$W/state" MOCK_LOG="$W/mock.jsonl" \
       EWE_COMPANION_SAFE="$W/bin/ewe-safe" EWE_COMPANION_BROWSER="$W/bin/ewe-browser" EWE_FAKE_DIR="$W/fakes"
DPATH="$W/bin:$PATH"
for t in claude nmcli ewe-bt qs notify-send ewe-theme gio; do
  if [ "$(PATH="$DPATH" command -v "$t")" != "$W/bin/$t" ]; then
    echo "FAIL  the fake $t is not first on the daemon's PATH: refusing to run"; exit 1
  fi
done
PORT=$((20000 + RANDOM % 20000))
pass=0 fail=0
ok() { echo "ok    $*"; pass=$((pass + 1)); }
no() { echo "FAIL  $*"; fail=$((fail + 1)); }
check() { if eval "$1"; then ok "$2"; else no "$2  [$3]"; fi; }
field() { python3 -c 'import json,sys; print(json.load(sys.stdin).get(sys.argv[1]) or "")' "$1"; }
js() { python3 -c 'import json,sys; r=json.load(sys.stdin); print(eval(sys.argv[1], {"r": r, "json": json}))' "$1"; }
now() { date +%s.%N; }
secs() { python3 -c 'import sys; print(round(float(sys.argv[2]) - float(sys.argv[1]), 2))' "$1" "$2"; }
lt() { python3 -c 'import sys; sys.exit(0 if float(sys.argv[1]) < float(sys.argv[2]) else 1)' "$1" "$2"; }
IP=""
cleanup() { kill "$MP" "$DP" $IP 2>/dev/null; wait 2>/dev/null; rm -rf "$W"; }
trap cleanup EXIT

python3 "$ROOT/tests/mock-llm.py" "$PORT" & MP=$!
env -u HYPRLAND_INSTANCE_SIGNATURE -u WAYLAND_DISPLAY -u XDG_DATA_HOME HOME="$W/home" PATH="$DPATH" \
  EWE_COMPANION_DEBUG=1 FAKE_CLAUDE_START=1.5 "$BIN/ewe-companiond" >>"$W/daemon.log" 2>&1 & DP=$!
for _ in $(seq 100); do [ -S "$EWE_COMPANION_SOCK" ] && break; sleep 0.02; done
sleep 0.2

settings() {  # provider model base_url
  python3 - "$1" "$2" "$3" <<'PY'
import json, os, socket, sys
s = socket.socket(socket.AF_UNIX); s.connect(os.environ["EWE_COMPANION_SOCK"])
args = {"provider": sys.argv[1], "model": sys.argv[2], "base_url": sys.argv[3]}
s.sendall((json.dumps({"type": "cli", "verb": "settings", "args": args}) + "\n").encode())
s.recv(65536)
PY
}
island() {  # log, then fake-island.py's flags; waits for every --say to be answered
  local log="$1"; shift
  python3 "$ROOT/tests/fake-island.py" --do "$@" >"$log" 2>&1 & IP=$!
  for _ in $(seq 300); do grep -q "said-all" "$log" && break; sleep 0.05; done
}
stop_island() { kill $IP 2>/dev/null; wait $IP 2>/dev/null; IP=""; sleep 0.2; }

settings custom "" "http://127.0.0.1:$PORT/v1"

# ── streaming ───────────────────────────────────────────────────────────
STORY="Once upon a time a sheep called Shaun lived in a small panel under the top bar, and every day he answered questions a little faster than the day before."
island "$W/i1.log" --caps partial --say "tell me a long story" --say "how busy is it" --say "shush everything"
partials=$(grep -c '^chat.reply partial ' "$W/i1.log")
check '[ "$partials" -ge 2 ]' "an island that draws partial text sees the answer grow ($partials pieces)" "$(head -c 600 "$W/i1.log")"
check 'sed "/^chat.reply answer /q" "$W/i1.log" | grep "^chat.reply partial " | python3 -c "
import json, sys
story = sys.argv[1]
parts = [json.loads(l.split(\" \", 2)[2]) for l in sys.stdin if l.startswith(\"chat.reply partial \")]
sys.exit(0 if parts and all(story.startswith(p) for p in parts) and len(set(parts)) == len(parts) else 1)" "$STORY"' \
      "…every piece is the start of the answer, each longer than the last" ""
check 'grep -qxF "chat.reply answer $STORY" "$W/i1.log"' "…and the whole answer comes last, under the same id" "$(grep '^chat.reply answer' "$W/i1.log" | head -2)"
check '! grep "^chat.reply partial " "$W/i1.log" | grep -q ACTION' "a streaming answer never shows an ACTION line" ""

# ── a look-up: the reads run, the model is asked again with what they found ──
check 'grep -qxF "chat.reply answer From the look-up: calc: 6*7 = 42" "$W/i1.log"' \
      "a reply of looks only runs them and asks again: the answer rests on what they found" "$(grep '^chat.reply answer' "$W/i1.log")"
check 'python3 -c "
import json, sys
r = [json.loads(l)[\"body\"] for l in open(sys.argv[1])]
again = [b for b in r if \"What you looked up\" in b[\"messages\"][-1][\"content\"]]
sys.exit(0 if len(again) == 1 and \"<untrusted source=\\\"ewe\\\"\" in again[0][\"messages\"][-1][\"content\"] else 1)" "$MOCK_LOG"' \
      "…what the reads found went back once, wrapped as untrusted (C4)" ""

# ── several ACTION lines: all run, one Undo puts all of them back ─────────
check 'grep -q "^do dnd {\"on\": true}" "$W/i1.log" && grep -q "ewe.insomnia\", \"on\"" "$W/fakes/qs.log"' \
      "a reply with two ACTION lines runs both (Do not disturb, keep awake)" "$(grep '^do ' "$W/i1.log"; cat "$W/fakes/qs.log")"
stop_island

# an island that does not draw partial text gets the answer whole
island "$W/i2.log" --say "tell me a long story"
check '! grep -q "^chat.reply partial" "$W/i2.log" && grep -qxF "chat.reply answer $STORY" "$W/i2.log"' \
      "an island that does not say it draws partial text gets the answer whole" "$(head -c 400 "$W/i2.log")"
stop_island

python3 "$ROOT/tests/fake-island.py" --do >"$W/i3.log" 2>&1 & IP=$!
for _ in $(seq 50); do "$BIN/ewe-companion" status | grep -q '"island":true' && break; sleep 0.05; done
# both off first, so each has something to undo
"$BIN/ewe-companion" debug run keep_awake '{"on": false}' >/dev/null
"$BIN/ewe-companion" debug run dnd '{"on": false}' >/dev/null
out=$("$BIN/ewe-companion" ask "shush everything")
tok=$(echo "$out" | field undo)
check '[ -n "$tok" ] && [ "$(echo "$out" | js "len(r[\"text\"].splitlines())")" = "2" ]' \
      "…their lines come back together, with one Undo" "$out"
out=$("$BIN/ewe-companion" debug undo "$tok")
sleep 0.3
check 'grep -q "^do dnd {\"on\": false}" "$W/i3.log" && tail -n 3 "$W/fakes/qs.log" | grep -q "ewe.insomnia\", \"off\""' \
      "…and that Undo puts both back" "$out $(grep '^do ' "$W/i3.log")"

# ── two plain commands in one line, offline ───────────────────────────────
settings none "" ""
out=$("$BIN/ewe-companion" ask "turn on dnd and set a timer for 25 minutes")
tok=$(echo "$out" | field undo)
check '[ "$(echo "$out" | field kind)" = "done" ] && echo "$out" | grep -q "25 minutes" && [ -n "$tok" ]' \
      "no provider: \"dnd on and a 25 minute timer\" runs both, offline" "$out"
check '[ "$("$BIN/ewe-companion" ask "show my timers" | js "len(r.get(\"rows\") or [])")" = "1" ]' "…the timer is set" ""
out=$("$BIN/ewe-companion" debug undo "$tok")
check '[ "$("$BIN/ewe-companion" ask "show my timers" | js "len(r.get(\"rows\") or [])")" = "0" ] && grep -q "^do dnd {\"on\": false}" "$W/i3.log"' \
      "…and one Undo cancels the timer and turns Do not disturb off" "$out"
out=$("$BIN/ewe-companion" ask "search for salt and pepper")
check '[ "$(echo "$out" | field command)" = "web_search" ]' "a line that is one command whole stays one (search for salt and pepper)" "$out"
stop_island

# ── the notes ride with the question; the system prompt stays the same ────
settings custom "" "http://127.0.0.1:$PORT/v1"
"$BIN/ewe-companion" ask "where do I turn on bluetooth" >/dev/null
"$BIN/ewe-companion" ask "what is a good name for a sheep" >/dev/null
check 'python3 -c "
import json, sys
a, b = [json.loads(l)[\"body\"][\"messages\"] for l in open(sys.argv[1])][-2:]
q = b[-1][\"content\"]
ok = (a[0][\"content\"] == b[0][\"content\"]                  # one system prompt for both
      and \"# This computer now\" not in b[0][\"content\"]
      and q.startswith(\"<notes>\") and \"# This computer now\" in q and \"- now: \" in q
      and q.endswith(\"</notes>\\n\\nwhat is a good name for a sheep\")
      and \"## Pair a Bluetooth device\" in a[-1][\"content\"])
sys.exit(0 if ok else 1)" "$MOCK_LOG"' \
      "the system prompt is the same each time; the topics, the time and this computer ride with the question" \
      "$(tail -c 500 "$MOCK_LOG")"
check 'python3 -c "
import json, sys
h = [json.loads(l)[\"body\"][\"messages\"] for l in open(sys.argv[1])][-1]
sys.exit(0 if any(m[\"content\"] == \"where do I turn on bluetooth\" for m in h[1:-1]) else 1)" "$MOCK_LOG"' \
      "…and the history keeps the user's words, not the notes" ""

# ── Claude Code: lean, and warm ahead of the question ─────────────────────
settings claude-code "" ""
t0=$(now); out=$("$BIN/ewe-companion" ask "first question"); t1=$(now); cold=$(secs "$t0" "$t1")
check '[ "$(echo "$out" | field text)" = "echo: first question" ] && ! lt "$cold" 1.4' \
      "Claude Code cold: the question waits for it to start (${cold} s)" "$out"
sleep 2.2
t0=$(now); out=$("$BIN/ewe-companion" ask "second question"); t1=$(now); warm=$(secs "$t0" "$t1")
check '[ "$(echo "$out" | field text)" = "echo: second question" ] && lt "$warm" 1.0' \
      "…after an answer the next one is started ahead: the next question does not wait (${warm} s)" "$out"
check 'python3 -c "
import json, sys
starts = [json.loads(l) for l in open(sys.argv[1]) if \"argv\" in l]
a = starts[0][\"argv\"]
lean = all(f in a for f in [\"--disable-slash-commands\", \"--strict-mcp-config\", \"--no-session-persistence\",
                            \"--include-partial-messages\", \"--input-format\"])
sources = a[a.index(\"--setting-sources\") + 1] == \"\"
sys.exit(0 if lean and sources and \"--bare\" not in a and a[a.index(\"--model\") + 1] == \"haiku\"
         and all(s[\"internal\"] == \"1\" for s in starts) else 1)" "$W/fakes/claude.log"' \
      "Claude Code runs lean: no skills, MCP servers, settings or hooks of the user's, our relay quiet" \
      "$(head -c 600 "$W/fakes/claude.log")"
# the chat field's focus starts it too (another model: the waiting one does not fit)
settings claude-code sonnet ""
python3 "$ROOT/tests/fake-island.py" --do --caps partial --warm >"$W/i4.log" 2>&1 & IP=$!
sleep 2.4
t0=$(now); out=$("$BIN/ewe-companion" ask "third question"); t1=$(now); warm=$(secs "$t0" "$t1")
check '[ "$(echo "$out" | field text)" = "echo: third question" ] && lt "$warm" 1.0' \
      "chat.warm (the chat field's focus) starts Claude Code before the question (${warm} s)" "$out"
check 'grep "\"argv\"" "$W/fakes/claude.log" | tail -n 2 | head -n 1 | grep -q "\"sonnet\""' \
      "…with the model the settings name" "$(tail -n 3 "$W/fakes/claude.log")"
stop_island
check '! grep -qi "panicked\|error" "$W/daemon.log"' "the daemon logged no error" "$(grep -i "panicked\|error" "$W/daemon.log" | head -3)"

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
