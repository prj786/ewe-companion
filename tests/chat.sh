#!/usr/bin/env bash
# tests/chat.sh: the chat path end to end against tests/mock-llm.py: the
# offline matcher, a question answered by an OpenAI-compatible provider, an
# ACTION reply turned into a command, the model picked from /v1/models, and
# what the provider was sent (the system prompt, the history; no key).
#   cargo build && tests/chat.sh
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/debug}"
W="$(mktemp -d)"; chmod 700 "$W"
export EWE_COMPANION_SOCK="$W/c.sock" XDG_STATE_HOME="$W/state" MOCK_LOG="$W/mock.jsonl"
PORT=$((20000 + RANDOM % 20000))
pass=0 fail=0
ok() { echo "ok    $*"; pass=$((pass + 1)); }
no() { echo "FAIL  $*"; fail=$((fail + 1)); }
check() { if eval "$1"; then ok "$2"; else no "$2  [$3]"; fi; }
field() { python3 -c 'import json,sys; print(json.load(sys.stdin).get(sys.argv[1]) or "")' "$1"; }
cleanup() { kill "$MP" "$DP" 2>/dev/null; wait 2>/dev/null; rm -rf "$W"; }
trap cleanup EXIT

python3 "$ROOT/tests/mock-llm.py" "$PORT" & MP=$!
env -u HYPRLAND_INSTANCE_SIGNATURE EWE_COMPANION_DEBUG=1 "$BIN/ewe-companiond" >"$W/daemon.log" 2>&1 & DP=$!
for _ in $(seq 100); do [ -S "$EWE_COMPANION_SOCK" ] && break; sleep 0.02; done
sleep 0.2

# no provider: commands still match offline; a question says how to pick one
out=$("$BIN/ewe-companion" ask "what is 2+2")
check '[ "$(echo "$out" | field kind)" = "info" ]' "no provider: a question says how to pick one" "$out"
out=$("$BIN/ewe-companion" ask "dnd on")
check '[ "$(echo "$out" | field command)" = "dnd" ]' "no provider: \"dnd on\" is still a command (offline)" "$out"
out=$("$BIN/ewe-companion" ask "open kitty in nowhere-zz-folder")
check '[ "$(echo "$out" | field kind)" = "error" ] && echo "$out" | grep -q "find a folder called nowhere-zz-folder"' \
      "no provider: a folder that is not here says so (and runs nothing)" "$out"
# shell commands are proposals: shown with a token, run only on the click
js() { python3 -c 'import json,sys; r=json.load(sys.stdin); print(eval(sys.argv[1], {"r": r}))' "$1"; }
out=$("$BIN/ewe-companion" ask '$ echo hello-companion')
tok=$(echo "$out" | js '(r.get("confirm") or {}).get("token","")')
check '[ -n "$tok" ] && [ "$(echo "$out" | js "r[\"confirm\"][\"command\"]")" = "echo hello-companion" ]' \
      "\$ command: proposed with a token, nothing run yet" "$out"
out=$("$BIN/ewe-companion" debug confirm "$tok")
check '[ "$(echo "$out" | field kind)" = "done" ] && [ "$(echo "$out" | field output)" = "hello-companion" ]' \
      "the Run click runs it and shows what it printed" "$out"
out=$("$BIN/ewe-companion" debug confirm "$tok")
check 'echo "$out" | grep -q "no longer waiting"' "a proposal runs once" "$out"
out=$("$BIN/ewe-companion" ask '$ rm -rf ~')
check '[ "$(echo "$out" | field kind)" = "error" ] && echo "$out" | grep -q "won.t run" && [ "$(echo "$out" | js "r.get(\"confirm\")")" = "None" ]' \
      "a machine killer is refused, never offered" "$out"
out=$("$BIN/ewe-companion" ask '$ sudo pacman -Sy firefox')
check 'echo "$out" | grep -q "partial upgrades"' "a partial upgrade is refused (rule 10)" "$out"
out=$("$BIN/ewe-companion" ask '$ sudo pacman -Syu')
check '[ "$(echo "$out" | js "r[\"confirm\"][\"terminal\"]")" = "True" ] && [ "$(echo "$out" | js "r[\"confirm\"][\"risk\"]")" = "danger" ]' \
      "a root command is risky and goes to a terminal (its password)" "$out"
out=$("$BIN/ewe-companion" ask "how do I turn on light mode")
check '[ "$(echo "$out" | js "(r.get(\"confirm\") or {}).get(\"command\")")" = "ewe-theme scheme apply ewe-light" ]' \
      "no provider: the knowledge pack offers its own command with Run" "$out"
out=$("$BIN/ewe-companion" ask "how do I change my wallpaper?")
check 'echo "$out" | python3 -c "import json,sys; r=json.load(sys.stdin); sys.exit(0 if r[\"kind\"]==\"answer\" and r[\"text\"].startswith(\"Settings → Wallpaper\") and r[\"action\"][\"label\"]==\"Open Settings\" and r[\"action\"][\"args\"][\"page\"]==\"wallpaper\" else 1)"' \
      "no provider: an ewe question is answered from the knowledge pack, with its button" "$out"

# a custom OpenAI-compatible provider, no key, no model: the first from /v1/models
python3 - <<PY
import json, socket, os
s = socket.socket(socket.AF_UNIX); s.connect(os.environ["EWE_COMPANION_SOCK"])
s.sendall((json.dumps({"type": "cli", "verb": "settings", "args": {"provider": "custom", "model": "", "base_url": "http://127.0.0.1:$PORT/v1"}}) + "\n").encode())
s.recv(65536)
PY
out=$("$BIN/ewe-companion" ask "what is 2+2")
check '[ "$(echo "$out" | field kind)" = "answer" ] && [ "$(echo "$out" | field text)" = "4" ]' "a question → the provider's answer" "$out"
out=$("$BIN/ewe-companion" ask "keep things quiet for a while")
check '[ "$(echo "$out" | field command)" = "dnd" ]' "an ACTION reply → the dnd command (it needs the island, so it says so)" "$out"
check 'python3 -c "import json,sys; r=[json.loads(l) for l in open(sys.argv[1])]; b=r[-1][\"body\"]; sys.exit(0 if b[\"model\"]==\"mock-1\" and \"ACTION\" in b[\"messages\"][0][\"content\"] and any(m[\"content\"]==\"what is 2+2\" for m in b[\"messages\"]) else 1)" "$MOCK_LOG"' \
      "the provider got the model, the system prompt and the history" "$(tail -c 300 "$MOCK_LOG")"
check 'python3 -c "import json,sys; sys.exit(0 if all(json.loads(l)[\"auth\"] is None for l in open(sys.argv[1])) else 1)" "$MOCK_LOG"' \
      "no key configured: no Authorization header sent" ""
out=$("$BIN/ewe-companion" ask "make it light please")
tok=$(echo "$out" | js '(r.get("confirm") or {}).get("token","")')
check '[ -n "$tok" ] && [ "$(echo "$out" | field text)" = "Switch to light mode:" ]' \
      "a model's run_shell becomes a proposal, not a run" "$out"
out=$("$BIN/ewe-companion" debug confirm "$tok")
check '[ "$(echo "$out" | field output)" = "light-on" ]' "…and runs on the click" "$out"
out=$("$BIN/ewe-companion" ask "open the pod bay doors in hal-zz")
check '[ "$(echo "$out" | field text)" = "echo: open the pod bay doors in hal-zz" ]' \
      "words that name no app or folder go to the provider as a question" "$out"
out=$("$BIN/ewe-companion" ask "where do I turn on bluetooth")
check 'python3 -c "import json,sys; b=[json.loads(l) for l in open(sys.argv[1])][-1][\"body\"]; p=b[\"messages\"][0][\"content\"]; sys.exit(0 if \"# What you know about ewe\" in p and \"## Pair a Bluetooth device\" in p and \"# This computer now\" in p and \"<!--\" not in p else 1)" "$MOCK_LOG"' \
      "a provider is told the primer, the topics that fit, and this computer now" "$(tail -c 300 "$MOCK_LOG")"
check '[ "$(echo "$out" | python3 -c "import json,sys; print((json.load(sys.stdin).get(\"action\") or {}).get(\"label\",\"\"))")" = "Open Bluetooth" ]' \
      "the model's answer still carries the topic's button" "$out"
out=$("$BIN/ewe-companion" ask "  ")
check '[ "$(echo "$out" | field ok)" = "False" ] || [ "$(echo "$out" | field error)" != "" ]' "empty text is refused" "$out"

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
