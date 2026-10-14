#!/usr/bin/env bash
# tests/relay-failsafe.sh: the hook relay's fail-safes (PLAN §19), end to end
# over a real socket, against a scratch daemon and a scripted island.
#
# The one rule: whatever goes wrong, the relay prints nothing and exits 0,
# so Claude Code asks in its own terminal (C2). It prints an allow only for
# an answer the island sent (C1).
#
#   cargo build && tests/relay-failsafe.sh       BIN=<dir> to test a release build
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/debug}"
P="$ROOT/tests/payloads/claude"
W="$(mktemp -d)"
chmod 700 "$W"
export EWE_COMPANION_SOCK="$W/c.sock" XDG_STATE_HOME="$W/state"
pass=0 fail=0 DPID="" IPID=""

ok() { echo "ok    $*"; pass=$((pass + 1)); }
no() { echo "FAIL  $*"; fail=$((fail + 1)); }
check() { if eval "$1"; then ok "$2"; else no "$2  [$3]"; fi; }
ms() { date +%s%3N; }
hook() { "$BIN/ewe-companion" hook claude "$@"; }
# two JSON texts equal as values
jeq() { python3 -c 'import json,sys; sys.exit(0 if json.loads(sys.argv[1]) == json.loads(sys.argv[2]) else 1)' "$1" "$2" 2>/dev/null; }

daemon_up() {
  # no compositor: "is the agent's terminal focused?" must not ask the
  # session this test runs in (it is, and the finished peek would be held)
  env -u HYPRLAND_INSTANCE_SIGNATURE EWE_COMPANION_DEBUG=1 EWE_COMPANION_WAIT_MS="${1:-108000}" \
    "$BIN/ewe-companiond" >>"$W/daemon.log" 2>&1 &
  DPID=$!
  for _ in $(seq 100); do [ -S "$EWE_COMPANION_SOCK" ] && return 0; sleep 0.02; done
  echo "daemon did not come up"; cat "$W/daemon.log"; exit 1
}
daemon_down() { [ -n "$DPID" ] && kill "$DPID" 2>/dev/null; wait "$DPID" 2>/dev/null; DPID=""; }
island() {
  python3 "$ROOT/tests/fake-island.py" "$@" >"$W/island.log" 2>&1 &
  IPID=$!
  for _ in $(seq 100); do grep -q '^state' "$W/island.log" 2>/dev/null && return 0; sleep 0.02; done
  echo "island did not connect"; cat "$W/island.log"; exit 1
}
island_down() { [ -n "$IPID" ] && kill "$IPID" 2>/dev/null; wait "$IPID" 2>/dev/null; IPID=""; }
cleanup() { island_down; daemon_down; rm -rf "$W"; }
trap cleanup EXIT

[ -x "$BIN/ewe-companion" ] && [ -x "$BIN/ewe-companiond" ] || { echo "build first: cargo build"; exit 1; }

# ── nobody home ────────────────────────────────────────────────────────
t0=$(ms); out=$(hook PermissionRequest <"$P/permission-bash.json"); rc=$?; t=$(($(ms) - t0))
check '[ -z "$out" ] && [ $rc -eq 0 ] && [ $t -lt 50 ]' "no socket: nothing, exit 0, ${t} ms" "out=$out rc=$rc"

mkdir -m 777 "$W/open"
out=$(EWE_COMPANION_SOCK="$W/open/c.sock" hook PermissionRequest <"$P/permission-bash.json"); rc=$?
check '[ -z "$out" ] && [ $rc -eq 0 ]' "a socket dir others can write: refused" "out=$out"

out=$(echo 'not json' | hook PermissionRequest); rc=$?
check '[ -z "$out" ] && [ $rc -eq 0 ]' "garbage on stdin: nothing, exit 0" "out=$out"

daemon_up
t0=$(ms); out=$(hook PermissionRequest <"$P/permission-bash.json"); rc=$?; t=$(($(ms) - t0))
check '[ -z "$out" ] && [ $rc -eq 0 ] && [ $t -lt 500 ]' "daemon but no island: nothing, at once (${t} ms)" "out=$out"

island
t0=$(ms); out=$(hook PermissionRequest <"$P/permission-bash.json"); rc=$?; t=$(($(ms) - t0))
check '[ -z "$out" ] && [ $rc -eq 0 ] && [ $t -ge 700 ] && [ $t -lt 1500 ]' "island never acks: nothing after ${t} ms (800 ms budget)" "out=$out"
sleep 0.1
check 'grep -q "card.close .* no-ack" "$W/island.log"' "  … and the card closes (no-ack)" "$(tail -3 "$W/island.log")"
island_down

# ── a human answers ────────────────────────────────────────────────────
island --ack --answer allow
out=$(hook PermissionRequest <"$P/permission-bash.json")
check 'jeq "$out" "{\"hookSpecificOutput\":{\"hookEventName\":\"PermissionRequest\",\"decision\":{\"behavior\":\"allow\"}}}"' "Allow → exact allow JSON" "$out"
island_down

island --ack --answer deny
out=$(hook PermissionRequest <"$P/permission-bash.json")
check 'jeq "$out" "{\"hookSpecificOutput\":{\"hookEventName\":\"PermissionRequest\",\"decision\":{\"behavior\":\"deny\",\"message\":\"Denied from the ewe companion\"}}}"' "Deny → deny with a message" "$out"
island_down

island --ack --answer always:0
out=$(hook PermissionRequest <"$P/permission-bash.json")
want=$(python3 -c 'import json,sys; p=json.load(open(sys.argv[1])); print(json.dumps({"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow","updatedPermissions":[p["permission_suggestions"][0]]}}}))' "$P/permission-bash.json")
check 'jeq "$out" "$want"' "Always → Claude's own suggestion echoed back" "$out"
island_down

island --ack --answer always:7
out=$(hook PermissionRequest <"$P/permission-bash.json")
check '[ -z "$out" ]' "an Always Claude did not offer: nothing" "$out"
island_down

island --ack --answer answer --answers '{"Which test runner?":"Vitest"}'
out=$(hook PreToolUse <"$P/ask-question.json")
want=$(python3 -c 'import json,sys; p=json.load(open(sys.argv[1])); i=dict(p["tool_input"]); i["answers"]={"Which test runner?":"Vitest"}; print(json.dumps({"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow","updatedInput":i}}))' "$P/ask-question.json")
check 'jeq "$out" "$want"' "a question's answer → updatedInput with answers" "$out"
island_down

island --ack --answer answer --answers '{"Which test runner?":"Mocha"}'
out=$(hook PreToolUse <"$P/ask-question.json")
check '[ -z "$out" ]' "an answer that is not an option: nothing" "$out"
island_down

island --ack --answer terminal
out=$(hook PermissionRequest <"$P/permission-bash.json")
check '[ -z "$out" ]' "Answer in terminal: nothing (the terminal asks)" "$out"
island_down

island --ack --answer allow
big=$(python3 -c 'import json; print(json.dumps({"session_id":"big","cwd":"/w","hook_event_name":"PermissionRequest","tool_name":"Write","tool_input":{"file_path":"/w/big.txt","content":"x"*2_000_000}}))')
out=$(printf '%s' "$big" | hook PermissionRequest)
check 'jeq "$out" "{\"hookSpecificOutput\":{\"hookEventName\":\"PermissionRequest\",\"decision\":{\"behavior\":\"allow\"}}}"' "a 2 MB payload is trimmed and still answered" "$(printf '%s' "$out" | head -c 200)"
island_down

# a plain PreToolUse (not a question) never waits and never answers
island --ack --answer allow
out=$(echo '{"session_id":"s","cwd":"/w","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"ls"}}' | hook PreToolUse)
check '[ -z "$out" ]' "PreToolUse for Bash: never answered by us" "$out"
island_down

# ── things go away mid-request ─────────────────────────────────────────
island --ack
# the binary itself in the background (a function would fork a subshell and
# the kill would miss the relay): Claude cancels a hook by killing it
"$BIN/ewe-companion" hook claude PermissionRequest <"$P/permission-bash.json" >"$W/relay.out" &
RP=$!
sleep 0.5
kill "$RP" 2>/dev/null; wait "$RP" 2>/dev/null
sleep 0.3
check 'grep -q "card.close .* answered-elsewhere" "$W/island.log"' "the relay hangs up (answered in the terminal): the card closes" "$(tail -2 "$W/island.log")"
island_down

island --ack
( hook PermissionRequest <"$P/permission-bash.json" >"$W/relay.out"; echo "rc=$?" >>"$W/relay.out" ) &
RP=$!
sleep 0.5
t0=$(ms); daemon_down; wait "$RP"; t=$(($(ms) - t0))
check '[ "$(cat "$W/relay.out")" = "rc=0" ] && [ $t -lt 1000 ]' "the daemon stops mid-request: nothing, exit 0 (${t} ms)" "$(cat "$W/relay.out")"
island_down

daemon_up
island --ack
( hook PermissionRequest <"$P/permission-bash.json" >"$W/relay.out"; echo "rc=$?" >>"$W/relay.out" ) &
RP=$!
sleep 0.5
t0=$(ms); island_down; wait "$RP"; t=$(($(ms) - t0))
check '[ "$(cat "$W/relay.out")" = "rc=0" ] && [ $t -lt 1000 ]' "the island goes away mid-request: nothing (${t} ms)" "$(cat "$W/relay.out")"
daemon_down

daemon_up 1500
island --ack
t0=$(ms); out=$(hook PermissionRequest <"$P/permission-bash.json"); t=$(($(ms) - t0))
sleep 0.1
check '[ -z "$out" ] && [ $t -ge 1400 ] && [ $t -lt 3000 ] && grep -q "card.close .* timeout" "$W/island.log"' "nobody answers in time: nothing after ${t} ms, card closes (timeout)" "out=$out"
island_down
daemon_down

daemon_up
island --ack --answer allow --env '{"locked":true}'
t0=$(ms); out=$(hook PermissionRequest <"$P/permission-bash.json"); t=$(($(ms) - t0))
check '[ -z "$out" ] && [ $t -lt 500 ]' "the screen is locked: nothing, at once (${t} ms)" "out=$out"
island_down

# ── events that do not wait ────────────────────────────────────────────
island
t0=$(ms); out=$(hook SessionStart <"$P/session-start.json"); t=$(($(ms) - t0))
check '[ -z "$out" ] && [ $t -lt 300 ]' "SessionStart: nothing printed, ${t} ms" "out=$out"
s=$("$BIN/ewe-companion" sessions)
check 'echo "$s" | grep -q "fixture-1"' "  … and the session is listed" "$s"
hook Stop <"$P/stop.json" >/dev/null
sleep 0.4
check 'grep -q "peek claude:fixture-1:stop Claude finished in ewe-browser/" "$W/island.log"' "Stop → a finished peek" "$(grep peek "$W/island.log" | tail -1)"
island_down

# ── the CLI's own promise: JSON, exit 0 ────────────────────────────────
out=$("$BIN/ewe-companion" status); rc=$?
check 'echo "$out" | python3 -c "import json,sys; d=json.load(sys.stdin); sys.exit(0 if d[\"ok\"] and d[\"running\"] else 1)" && [ $rc -eq 0 ]' "status: JSON, running" "$out"
daemon_down
out=$("$BIN/ewe-companion" status); rc=$?
check 'jeq "$out" "{\"ok\":false,\"running\":false,\"error\":\"not-running\",\"version\":\"$(cat "$ROOT/VERSION")\"}" && [ $rc -eq 0 ]' "status with no daemon: not-running, exit 0" "$out"
out=$("$BIN/ewe-companion" frobnicate); rc=$?
check 'echo "$out" | grep -q "unknown verb" && [ $rc -eq 0 ]' "an unknown verb: JSON error, exit 0" "$out"

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
