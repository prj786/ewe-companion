#!/usr/bin/env bash
# tests/chat.sh: the chat path end to end against tests/mock-llm.py: the
# offline matcher, a question answered by an OpenAI-compatible provider, an
# ACTION reply turned into a command, the model picked from /v1/models, and
# what the provider was sent (the system prompt, the history; no key). Then
# Safe, ewe browser and the finder (TRIO.md C1) against the fakes in
# tests/fakes/ and a scripted island (tests/fake-island.py --do): never the
# real ewe-safe or ewe-browser, and the finder walks a scratch home. Then
# wave 2: a site's search, the browser's new verbs, Wi-Fi, Bluetooth,
# airplane mode, keeping awake, the calculator, timers (across a daemon
# restart) and the proposals, with every desktop tool a PATH shim from
# tests/fakes/ (nmcli, ewe-bt, qs, notify-send, ewe-theme, gio): never the
# owner's real radios, shell, theme or trash. Then "Shaun gets wiser": the
# clock and dates, units, this computer's state (read for real, read-only),
# the network (nmcli and ip fakes), windows and workspaces (a hyprctl fake:
# never the owner's compositor), what Shaun remembers, and typos.
#   cargo build && tests/chat.sh
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BIN:-$ROOT/target/debug}"
W="$(mktemp -d)"; chmod 700 "$W"
# the fakes are copies, so a check can change one without touching the repo
mkdir -p "$W/bin" "$W/fakes" && cp "$ROOT"/tests/fakes/* "$W/bin/"
export EWE_COMPANION_SOCK="$W/c.sock" XDG_STATE_HOME="$W/state" MOCK_LOG="$W/mock.jsonl" \
       EWE_COMPANION_SAFE="$W/bin/ewe-safe" EWE_COMPANION_BROWSER="$W/bin/ewe-browser" EWE_FAKE_DIR="$W/fakes"
# the daemon's PATH starts with the fakes: a desktop command must never
# reach the real nmcli, ewe-bt, qs, notify-send, ewe-theme, gio, hyprctl or ip
DPATH="$W/bin:$PATH"
for t in nmcli ewe-bt qs notify-send ewe-theme gio hyprctl ip; do
  if [ "$(PATH="$DPATH" command -v "$t")" != "$W/bin/$t" ]; then
    echo "FAIL  the fake $t is not first on the daemon's PATH: refusing to run"; exit 1
  fi
done
H="$W/home"
mkdir -p "$H/Documents/Taxes" "$H/Projects/ewe/ewe-safe/src" "$H/Projects/ewe/node_modules/ewe-safe" \
         "$H/.hidden/ewe-safe" "$H/a/b/c/deep-folder"
touch "$H/Documents/Taxes/tax-2025.pdf" "$H/Documents/Taxes/tax-notes.txt" "$H/Documents/resume.odt"
PORT=$((20000 + RANDOM % 20000))
pass=0 fail=0
ok() { echo "ok    $*"; pass=$((pass + 1)); }
no() { echo "FAIL  $*"; fail=$((fail + 1)); }
check() { if eval "$1"; then ok "$2"; else no "$2  [$3]"; fi; }
field() { python3 -c 'import json,sys; print(json.load(sys.stdin).get(sys.argv[1]) or "")' "$1"; }
IP=""
# a process called firefox (a copy of sleep, off the PATH) that the fake
# Hyprland's windows belong to: "kill firefox" must find it, and never end it
mkdir -p "$W/procs" && cp "$(command -v sleep)" "$W/procs/firefox"
"$W/procs/firefox" 600 & FP=$!
export FAKE_HYPR_PID="$FP"
cleanup() { kill "$MP" "$DP" $IP "$FP" 2>/dev/null; wait 2>/dev/null; rm -rf "$W"; }
trap cleanup EXIT

python3 "$ROOT/tests/mock-llm.py" "$PORT" & MP=$!
daemon() {
  env -u HYPRLAND_INSTANCE_SIGNATURE -u WAYLAND_DISPLAY -u XDG_DATA_HOME HOME="$H" PATH="$DPATH" \
    EWE_COMPANION_DEBUG=1 "$BIN/ewe-companiond" >>"$W/daemon.log" 2>&1 & DP=$!
}
daemon
for _ in $(seq 100); do [ -S "$EWE_COMPANION_SOCK" ] && break; sleep 0.02; done
sleep 0.2

# no provider: commands still match offline; a question says how to pick one
out=$("$BIN/ewe-companion" ask "who wrote the odyssey")
check '[ "$(echo "$out" | field kind)" = "info" ]' "no provider: a question says how to pick one" "$out"
out=$("$BIN/ewe-companion" ask "what is 15% of 80")
check '[ "$(echo "$out" | field command)" = "calc" ] && [ "$(echo "$out" | field text)" = "15% of 80 = 12" ]' \
      "no provider: a sum is worked out offline (calc)" "$out"
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
check 'echo "$out" | python3 -c "import json,sys; r=json.load(sys.stdin); sys.exit(0 if r[\"kind\"]==\"answer\" and r[\"text\"].startswith(\"ewe Settings → Wallpaper\") and r[\"action\"][\"label\"]==\"Open ewe Settings\" and r[\"action\"][\"args\"][\"page\"]==\"wallpaper\" else 1)"' \
      "no provider: an ewe question is answered from the knowledge pack, with its button" "$out"

# a custom OpenAI-compatible provider, no key, no model: the first from /v1/models
python3 - <<PY
import json, socket, os
s = socket.socket(socket.AF_UNIX); s.connect(os.environ["EWE_COMPANION_SOCK"])
s.sendall((json.dumps({"type": "cli", "verb": "settings", "args": {"provider": "custom", "model": "", "base_url": "http://127.0.0.1:$PORT/v1"}}) + "\n").encode())
s.recv(65536)
PY
out=$("$BIN/ewe-companion" ask "what is 2+2, in words")
check '[ "$(echo "$out" | field kind)" = "answer" ] && [ "$(echo "$out" | field text)" = "4" ]' "a question → the provider's answer" "$out"
out=$("$BIN/ewe-companion" ask "keep things quiet for a while")
check '[ "$(echo "$out" | field command)" = "dnd" ]' "an ACTION reply → the dnd command (it needs the island, so it says so)" "$out"
check 'python3 -c "import json,sys; r=[json.loads(l) for l in open(sys.argv[1])]; b=r[-1][\"body\"]; sys.exit(0 if b[\"model\"]==\"mock-1\" and \"ACTION\" in b[\"messages\"][0][\"content\"] and any(m[\"content\"]==\"what is 2+2, in words\" for m in b[\"messages\"]) else 1)" "$MOCK_LOG"' \
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
check 'python3 -c "import json,sys; b=[json.loads(l) for l in open(sys.argv[1])][-1][\"body\"]; p=b[\"messages\"][0][\"content\"]; q=b[\"messages\"][-1][\"content\"]; sys.exit(0 if \"# What you know about ewe\" in p and \"## Pair a Bluetooth device\" in q and \"# This computer now\" in q and \"<!--\" not in p + q else 1)" "$MOCK_LOG"' \
      "a provider is told the primer (system prompt), and with the question the topics that fit and this computer now" "$(tail -c 300 "$MOCK_LOG")"
check '[ "$(echo "$out" | python3 -c "import json,sys; print((json.load(sys.stdin).get(\"action\") or {}).get(\"label\",\"\"))")" = "Open Bluetooth" ]' \
      "the model's answer still carries the topic's button" "$out"
out=$("$BIN/ewe-companion" ask "  ")
check '[ "$(echo "$out" | field ok)" = "False" ] || [ "$(echo "$out" | field error)" != "" ]' "empty text is refused" "$out"

# ── Safe, ewe browser and the finder (TRIO.md C1) ──────────────────────────
# a scripted island answers `do` (ewe browser's acts, Safe's unlock prompt)
python3 "$ROOT/tests/fake-island.py" --do --env '{"wayland": "wayland-test", "hypr": "fake-hypr-sig"}' >"$W/island.log" 2>&1 & IP=$!
for _ in $(seq 50); do "$BIN/ewe-companion" status | grep -q '"island":true' && break; sleep 0.05; done
js() { python3 -c 'import json,sys; r=json.load(sys.stdin); print(eval(sys.argv[1], {"r": r, "json": json}))' "$1"; }
slog() { tail -n 1 "$W/fakes/safe.log"; }
blog() { cat "$W/fakes/browser.log" 2>/dev/null; }

# Safe: one login copies at once; Safe copies, the reply holds no value
out=$("$BIN/ewe-companion" ask "copy my discord password")
check '[ "$(echo "$out" | field text)" = "Copied the Discord password (sam). It clears in 30 s." ]' \
      "Safe: one login → Safe copies it, the island says what" "$out"
check 'slog | grep -q "\"copy\", \"4f9c0d1e-0000-4000-8000-000000000003\", \"--field\", \"password\"" && slog | grep -q "\"wayland\": \"wayland-test\""' \
      "…through ewe-safe copy <id> --field password, on the island's Wayland display" "$(slog)"
out=$("$BIN/ewe-companion" ask "copy my github password")
check '[ "$(echo "$out" | js "len(r[\"rows\"])")" = "2" ] && [ "$(echo "$out" | js "r[\"rows\"][1][\"actions\"][0][\"args\"][\"id\"]")" = "4f9c0d1e-0000-4000-8000-000000000002" ]' \
      "Safe: several logins → rows (title, username) whose click copies that one" "$out"
out=$("$BIN/ewe-companion" debug run safe_copy '{"id": "4f9c0d1e-0000-4000-8000-000000000002", "field": "password", "title": "github.com", "username": "sam-work"}')
check '[ "$(echo "$out" | field text)" = "Copied the github.com password (sam-work). It clears in 30 s." ]' \
      "…a row's click copies it" "$out"
out=$("$BIN/ewe-companion" ask "copy my github code")
check 'echo "$out" | grep -q "Copied the GitHub one-time code (sam)"' "Safe: \"code\" means the one with a one-time code" "$out"
out=$("$BIN/ewe-companion" ask "copy my netflix password")
check '[ "$(echo "$out" | field kind)" = "error" ] && [ "$(echo "$out" | field text)" = "ewe Safe has no login for netflix." ]' \
      "Safe: nothing for those words says so" "$out"
out=$("$BIN/ewe-companion" ask "how many passwords do I have")
check '[ "$(echo "$out" | field text)" = "ewe Safe is unlocked · 3 items · in sync." ]' "Safe: counts and state, offline" "$out"
out=$("$BIN/ewe-companion" ask "lock my safe")
check '[ "$(echo "$out" | field text)" = "ewe Safe is locked." ]' "Safe: lock" "$out"
out=$("$BIN/ewe-companion" ask "copy my discord password")
check '[ "$(echo "$out" | js "r[\"action\"][\"label\"]")" = "Unlock ewe Safe" ] && [ "$(echo "$out" | js "r[\"action\"][\"args\"][\"then\"][\"command\"]")" = "safe_copy" ]' \
      "Safe locked: the reply offers Unlock Safe, keeping the copy for after" "$out"
then=$(echo "$out" | js 'json.dumps(r["action"]["args"])')
out=$("$BIN/ewe-companion" debug run safe_unlock "$then")
check '[ "$(echo "$out" | field text)" = "Type your master password in ewe Safe'"'"'s window." ] && grep -q "do safe_unlock" "$W/island.log"' \
      "Unlock Safe opens Safe's own prompt, through the island" "$out"
for _ in $(seq 60); do grep -q "chat.reply done Copied the Discord password" "$W/island.log" && break; sleep 0.1; done
check 'grep -q "chat.reply done Copied the Discord password (sam)" "$W/island.log"' \
      "…and once Safe is unlocked, the waiting copy runs and says so" "$(tail -n 3 "$W/island.log")"
# a command that would print a password (or a login's name) is never offered:
# its output would go back into the conversation (C3)
out=$("$BIN/ewe-companion" ask '$ ewe-safe read "safe://Personal/GitHub/password"')
check '[ "$(echo "$out" | field kind)" = "error" ] && echo "$out" | grep -q "ewe Safe.s values never go through the companion" && [ "$(echo "$out" | js "r.get(\"confirm\")")" = "None" ]' \
      "\$ ewe-safe read …: refused, never offered" "$out"
out=$("$BIN/ewe-companion" ask "print my github password")
check '[ "$(echo "$out" | field kind)" = "error" ] && [ "$(echo "$out" | js "r.get(\"confirm\")")" = "None" ] && ! grep -q "\"reveal\"" "$W/fakes/safe.log"' \
      "a model's run_shell of ewe-safe reveal: refused, Safe never asked" "$out"
# Safe goes away while its prompt is open: what waited is dropped, and said
"$BIN/ewe-companion" ask "lock my safe" >/dev/null
out=$("$BIN/ewe-companion" debug run safe_unlock '{"then": {"command": "safe_copy", "args": {"query": "discord"}}}')
touch "$W/fakes/safe-gone"
for _ in $(seq 40); do grep -q "chat.reply error The unlock did not finish" "$W/island.log" && break; sleep 0.1; done
check 'grep -q "chat.reply error The unlock did not finish: ewe Safe stopped answering, so I dropped what was waiting" "$W/island.log"' \
      "Safe gone during the unlock: the waiting copy is dropped, and the island is told" "$(tail -n 2 "$W/island.log")"
rm -f "$W/fakes/safe-gone"; echo '{"locked": false}' >"$W/fakes/safe.json"
out=$("$BIN/ewe-companion" ask "what did I just ask you")
check 'python3 -c "import json,sys; b=[json.loads(l) for l in open(sys.argv[1])][-1][\"body\"]; t=json.dumps(b[\"messages\"][1:]); sys.exit(0 if \"(a request to ewe Safe)\" in t and \"sam\" not in t and \"Discord\" not in t else 1)" "$MOCK_LOG"' \
      "names stay local: the next question to a model carries no title or username" "$(tail -c 400 "$MOCK_LOG")"

# ewe browser through its command line; acts run in the island
out=$("$BIN/ewe-companion" ask "search for sheep breeds")
check '[ "$(echo "$out" | field text)" = "Searching the web for sheep breeds · ewe Browser" ] && [ -n "$(echo "$out" | field undo)" ]' \
      "browser: search → ewe-browser search, with its Undo" "$out"
check 'blog | python3 -c "import json,sys; sys.exit(0 if any(a[:2]==[\"call\",\"search\"] and json.loads(a[2])[\"query\"]==\"sheep breeds\" for a in map(json.loads, sys.stdin)) else 1)" && grep -q "do browser" "$W/island.log"' \
      "…run by the island (a browser it starts belongs to the desktop)" "$(blog | tail -n 2)"
out=$("$BIN/ewe-companion" ask "open github.com")
check '[ "$(echo "$out" | field text)" = "Opened github.com · ewe Browser" ]' "browser: open github.com" "$out"
out=$("$BIN/ewe-companion" ask "find tax in my history")
check '[ "$(echo "$out" | js "r[\"rows\"][0][\"title\"]")" = "Tax guide 2026" ] && [ "$(echo "$out" | js "r[\"rows\"][0][\"actions\"][0][\"command\"]")" = "open_url" ]' \
      "browser: history → rows with Open" "$out"
out=$("$BIN/ewe-companion" ask "show my tabs")
check '[ "$(echo "$out" | field text)" = "2 tabs are open in ewe Browser:" ] && [ "$(echo "$out" | js "r[\"rows\"][0][\"actions\"][0][\"label\"]")" = "Go to" ]' \
      "browser: tabs → rows with Go to (web pages only)" "$out"
out=$("$BIN/ewe-companion" ask "go to the youtube tab")
check '[ "$(echo "$out" | field text)" = "Switched to YouTube" ] && [ -n "$(echo "$out" | field undo)" ]' \
      "browser: the YouTube tab → tab activate, with Undo" "$out"
out=$("$BIN/ewe-companion" ask "play massive attack teardrop")
check '[ "$(echo "$out" | field text)" = "Playing Teardrop · Massive Attack" ]' "browser: play → the library first" "$out"
out=$("$BIN/ewe-companion" ask "what is this page about?")
check '[ "$(echo "$out" | js "r[\"action\"][\"label\"]")" = "Do it" ] && echo "$out" | grep -q "asking first" && ! blog | grep -q evil.example' \
      "a page's text is untrusted: the ACTION it provoked is a button, never a run (C4)" "$out"
check 'python3 -c "import json,sys; b=[json.loads(l) for l in open(sys.argv[1])][-1][\"body\"]; u=b[\"messages\"][-1][\"content\"]; sys.exit(0 if \"<untrusted source=\\\"web page\\\"\" in u and u.rstrip().endswith(\"</untrusted>\") else 1)" "$MOCK_LOG"' \
      "…and it went to the model wrapped in <untrusted>" "$(tail -c 300 "$MOCK_LOG")"
# an older ewe browser (no command line): the page opens the old way
touch "$W/fakes/browser-old"; touch -d '2 seconds ago' "$W/bin/ewe-browser"
out=$("$BIN/ewe-companion" ask "search for wool")
check '[ "$(echo "$out" | field text)" = "Searching the web for wool · ewe Browser" ] && grep -q "do browse {\"url\": \"https://duckduckgo.com/?q=wool\"}" "$W/island.log"' \
      "browser without a command line: search falls back to opening the URL" "$out"
out=$("$BIN/ewe-companion" ask "show my tabs")
check 'echo "$out" | grep -q "no command line yet"' "…and history or tabs say why they can't" "$out"
rm -f "$W/fakes/browser-old"; touch "$W/bin/ewe-browser"

# the finder: a scratch home, hidden folders and build trees skipped
out=$("$BIN/ewe-companion" ask "find my resume")
check '[ "$(echo "$out" | js "r[\"rows\"][0][\"title\"]")" = "resume.odt" ] && [ "$(echo "$out" | js "[a[\"label\"] for a in r[\"rows\"][0][\"actions\"]]")" = "['"'"'Open'"'"', '"'"'Show in folder'"'"', '"'"'Copy path'"'"']" ]' \
      "finder: find my resume → a row with Open, Show in folder, Copy path" "$out"
out=$("$BIN/ewe-companion" ask "where is the ewe-safe folder")
check '[ "$(echo "$out" | js "len(r[\"rows\"])")" = "1" ] && [ "$(echo "$out" | js "r[\"rows\"][0][\"actions\"][3][\"label\"]")" = "Open terminal here" ]' \
      "finder: the ewe-safe folder (not node_modules', not hidden) + Open terminal here" "$out"
out=$("$BIN/ewe-companion" ask "find pdfs about tax")
check '[ "$(echo "$out" | js "[x[\"title\"] for x in r[\"rows\"]]")" = "['"'"'tax-2025.pdf'"'"']" ]' "finder: pdfs about tax → only the pdf" "$out"
out=$("$BIN/ewe-companion" ask "open the deep-folder folder")
check '[ "$(echo "$out" | field text)" = "Opened ~/a/b/c/deep-folder" ]' "open a folder deeper than places look: the finder finds it" "$out"
out=$("$BIN/ewe-companion" ask "find zzqx nothing here")
check '[ "$(echo "$out" | field text)" = "echo: find zzqx nothing here" ]' "finder: nothing found → the words go on as a question" "$out"
out=$("$BIN/ewe-companion" find resume)
check '[ "$(echo "$out" | js "r[\"results\"][0][\"path\"]")" = "$H/Documents/resume.odt" ]' "ewe-companion find: JSON paths (rule 5)" "$out"

# ── wave 2: a site's own search, and the browser's new verbs ───────────────
tok() { echo "$1" | js 'r.get("undo") or ""'; }
blast() { tail -n "${1:-1}" "$W/fakes/browser.log"; }
out=$("$BIN/ewe-companion" ask "search men's shoes on zara")
check '[ "$(echo "$out" | field text)" = "Searching www.zara.com for men'"'"'s shoes · ewe Browser" ] && [ -n "$(tok "$out")" ]' \
      "site search: search men's shoes on zara → the site's own search, with Undo" "$out"
check 'blast | python3 -c "import json,sys; a=json.loads(sys.stdin.read()); sys.exit(0 if a[:2]==[\"call\",\"search\"] and json.loads(a[2])=={\"query\":\"men'"'"'s shoes\",\"site\":\"zara\"} else 1)"' \
      "…as ewe-browser call search {query, site}" "$(blast)"
out=$("$BIN/ewe-companion" ask "search hats on knitzz")
check '[ "$(echo "$out" | field text)" = "ewe Browser does not know knitzz, so I searched the web for hats on knitzz · ewe Browser" ]' \
      "site search: unknown-site → a plain search of the whole phrase, and it says so" "$out"
# the last call the browser got, as (tool, its JSON arguments)
lastcall() { blast | python3 -c 'import json,sys; a=json.loads(sys.stdin.read()); print(a[1], json.dumps(json.loads(a[2]), sort_keys=True))'; }
check '[ "$(lastcall)" = "search {\"query\": \"hats on knitzz\"}" ]' "…the plain search got the whole phrase" "$(blast)"
touch "$W/fakes/browser-v03"; touch -d '3 seconds ago' "$W/bin/ewe-browser"
out=$("$BIN/ewe-companion" ask "search men's shoes on zara")
check 'echo "$out" | grep -q "This version of ewe Browser cannot search inside a site yet, so I searched the web for men.s shoes on zara"' \
      "site search: a browser without --site (0.3) → the whole phrase, said so" "$out"
out=$("$BIN/ewe-companion" ask "switch to my work workspace in the browser")
check 'echo "$out" | grep -q "cannot do that yet: update it in Komble"' "workspaces on a 0.3 browser: update it, says so" "$out"
rm -f "$W/fakes/browser-v03"; touch "$W/bin/ewe-browser"
out=$("$BIN/ewe-companion" ask "bookmark this page")
check '[ "$(echo "$out" | field text)" = "Bookmarked Sheep breeds of the Caucasus" ] && [ -n "$(tok "$out")" ]' \
      "browser: bookmark this page → bookmark_add of the page in front, with Undo" "$out"
out=$("$BIN/ewe-companion" debug undo "$(tok "$out")")
check '[ "$(echo "$out" | field text)" = "Undone" ] && blast | grep -q "\"bookmark\", \"remove\", \"https://sheep.example/breeds\""' \
      "…its Undo removes that bookmark (the browser's own step)" "$(blast)"
out=$("$BIN/ewe-companion" ask "find pasta in my bookmarks")
check '[ "$(echo "$out" | js "r[\"rows\"][0][\"title\"]")" = "Pasta recipes" ] && [ "$(echo "$out" | js "r[\"rows\"][0][\"actions\"][0][\"command\"]")" = "open_url" ]' \
      "browser: bookmarks → rows with Open" "$out"
out=$("$BIN/ewe-companion" ask "go back a page")
check '[ "$(echo "$out" | field text)" = "Back a page · ewe Browser" ] && blast | grep -qx "\[\"back\"\]" && [ -n "$(tok "$out")" ]' \
      "browser: go back a page → ewe-browser back, with Undo (forward)" "$out"
out=$("$BIN/ewe-companion" ask "reload the page")
check '[ "$(echo "$out" | field text)" = "Reloaded the page · ewe Browser" ] && blast | grep -q "tab_reload"' "browser: reload the page" "$out"
out=$("$BIN/ewe-companion" ask "reopen the closed tab")
check '[ "$(echo "$out" | field text)" = "Reopened github.com · ewe Browser" ]' "browser: reopen the closed tab" "$out"
out=$("$BIN/ewe-companion" ask "mute the youtube tab")
check '[ "$(echo "$out" | field text)" = "Muted YouTube · ewe Browser" ] && [ -n "$(tok "$out")" ]' "browser: mute the YouTube tab, with Undo" "$out"
out=$("$BIN/ewe-companion" ask "close this tab")
check 'echo "$out" | js "r[\"confirm\"][\"command\"]" | grep -q "call tab_close" && ! grep -q "\"call\", \"tab_close\"" "$W/fakes/browser.log"' \
      "browser: close this tab → a proposal (the browser's tier 2), never a run" "$out"
touch "$W/fakes/browser-lies"; touch -d '4 seconds ago' "$W/bin/ewe-browser"
out=$("$BIN/ewe-companion" ask "close the youtube tab")
check 'echo "$out" | js "r[\"confirm\"][\"command\"]" | grep -q "call tab_close" && ! grep -q "\"call\", \"tab_close\"" "$W/fakes/browser.log"' \
      "close a tab is a proposal even when the browser calls it tier 1" "$out"
rm -f "$W/fakes/browser-lies"; touch "$W/bin/ewe-browser"
out=$("$BIN/ewe-companion" ask "switch to the firefox workspace")
check '[ "$(echo "$out" | field command)" != "browser_workspace" ]' "a workspace without \"browser\" is the desktop's" "$out"
out=$("$BIN/ewe-companion" ask "switch to my work workspace in the browser")
check '[ "$(echo "$out" | field text)" = "Switched to the Work workspace · ewe Browser" ] && blast | grep -q "\"workspace\", \"activate\", \"work\"" && [ -n "$(tok "$out")" ]' \
      "browser: switch to my work workspace → workspace activate, with Undo" "$out"
out=$("$BIN/ewe-companion" ask "switch to my sheepzz workspace in the browser")
check '[ "$(echo "$out" | field text)" = "ewe Browser has no workspace called sheepzz." ]' "browser: a workspace it does not have says so" "$out"
out=$("$BIN/ewe-companion" ask "new workspace called travel")
check '[ "$(echo "$out" | field text)" = "Made the workspace travel · ewe Browser" ]' "browser: a new workspace called travel" "$out"
out=$("$BIN/ewe-companion" debug undo "$(tok "$out")")
check 'blast | grep -q "\"workspace\", \"remove\", \"ws-new\""' "…its Undo removes the workspace it made" "$(blast)"
out=$("$BIN/ewe-companion" ask "next browser workspace")
check '[ "$(echo "$out" | field text)" = "Switched to the Work workspace · ewe Browser" ] && blast | grep -qx "\[\"workspace\", \"next\"\]"' \
      "browser: next browser workspace" "$out"
out=$("$BIN/ewe-companion" ask "show my browser workspaces")
check '[ "$(echo "$out" | js "len(r[\"rows\"])")" = "2" ] && [ "$(echo "$out" | js "r[\"rows\"][1][\"actions\"][0][\"args\"][\"name\"]")" = "ws-2" ]' \
      "browser: the workspaces → rows with Switch" "$out"
out=$("$BIN/ewe-companion" ask "open github.com in my work workspace")
check '[ "$(echo "$out" | field text)" = "Opened github.com in the work workspace · ewe Browser" ] && [ "$(lastcall)" = "open {\"targets\": [\"https://github.com\"], \"workspace\": \"work\"}" ]' \
      "browser: open a site in a named workspace" "$out"
out=$("$BIN/ewe-companion" ask "what did i download")
check '[ "$(echo "$out" | js "[a[\"label\"] for a in r[\"rows\"][0][\"actions\"]]")" = "['"'"'Open'"'"', '"'"'Show in folder'"'"']" ] && [ "$(echo "$out" | js "len(r[\"rows\"][1][\"actions\"])")" = "1" ]' \
      "browser: downloads → rows (Open + Show in folder when finished)" "$out"
did=$(echo "$out" | js 'r["rows"][0]["actions"][1]["args"]["id"]')
out=$("$BIN/ewe-companion" debug run browser_download "{\"id\": \"$did\", \"action\": \"show\", \"path\": \"/home/you/Downloads/archlinux.iso\"}")
check '[ "$(echo "$out" | field text)" = "Showed the download in its folder" ] && blast | grep -q "\"download\", \"show\", \"$did\""' \
      "…Show in folder → ewe-browser download show <id>" "$(blast)"
touch "$W/fakes/browser-closed"
out=$("$BIN/ewe-companion" debug run browser_download "{\"id\": \"$did\", \"action\": \"show\", \"path\": \"/etc/shadow\"}")
check '[ "$(echo "$out" | field text)" = "Opened /home/you/Downloads" ] && ! grep -q "/etc" "$W/island.log"' \
      "…the browser closed: its own list's path for the id, never the path passed in" "$out"
out=$("$BIN/ewe-companion" debug run browser_download '{"id": "d0000000-0000-4000-8000-00000000dead", "action": "open", "path": "/etc/shadow"}')
check '[ "$(echo "$out" | field kind)" = "error" ] && ! grep -q "/etc" "$W/island.log"' \
      "…an id the browser does not know opens nothing" "$out"
rm -f "$W/fakes/browser-closed"
out=$("$BIN/ewe-companion" ask "open the browser's settings")
check '[ "$(echo "$out" | field text)" = "Opened Settings · ewe Browser" ] && blast | grep -q "app://settings/"' "browser: its own Settings page" "$out"

# ── wave 2: the desktop, against PATH shims ────────────────────────────────
st() { python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get(sys.argv[2]))' "$W/fakes/$1" "$2" 2>/dev/null; }
out=$("$BIN/ewe-companion" ask "turn off wifi")
check '[ "$(echo "$out" | field text)" = "Wi-Fi off" ] && [ "$(st nmcli.json wifi)" = "False" ]' "Wi-Fi off (nmcli radio wifi off)" "$out"
out=$("$BIN/ewe-companion" debug undo "$(tok "$out")")
check '[ "$(echo "$out" | field text)" = "Wi-Fi on" ] && [ "$(st nmcli.json wifi)" = "True" ]' "…and its Undo turns it back on" "$out"
out=$("$BIN/ewe-companion" ask "wifi on")
check '[ "$(echo "$out" | field text)" = "Wi-Fi is already on" ]' "Wi-Fi already on: said, nothing run" "$out"
out=$("$BIN/ewe-companion" ask "turn bluetooth off")
check '[ "$(echo "$out" | field text)" = "Bluetooth off" ] && [ "$(st bt.json powered)" = "False" ]' "Bluetooth off (ewe-bt power off)" "$out"
out=$("$BIN/ewe-companion" ask "connect my airpods")
check '[ "$(echo "$out" | field text)" = "Bluetooth is off." ] && [ "$(echo "$out" | js "r[\"action\"][\"label\"]")" = "Turn on Bluetooth" ]' \
      "a device while Bluetooth is off → Turn on Bluetooth" "$out"
"$BIN/ewe-companion" debug run bluetooth '{"on": true}' >/dev/null
out=$("$BIN/ewe-companion" ask "connect my airpods")
check '[ "$(echo "$out" | field text)" = "Connected Sam'"'"'s AirPods Pro" ] && tail -n 1 "$W/fakes/bt.log" | grep -q "\"connect\", \"AA:BB:CC:00:00:01\"" && [ -n "$(tok "$out")" ]' \
      "connect my airpods → ewe-bt connect <its MAC>, with Undo" "$out"
out=$("$BIN/ewe-companion" debug undo "$(tok "$out")")
check '[ "$(echo "$out" | field text)" = "Disconnected Sam'"'"'s AirPods Pro" ]' "…its Undo disconnects it" "$out"
out=$("$BIN/ewe-companion" ask "connect my sony")
check '[ "$(echo "$out" | js "len(r[\"rows\"])")" = "2" ] && [ "$(echo "$out" | js "r[\"rows\"][0][\"actions\"][0][\"args\"][\"mac\"]")" = "AA:BB:CC:00:00:02" ]' \
      "two devices fit → rows whose click connects that one" "$out"
out=$("$BIN/ewe-companion" ask "airplane mode on")
check '[ "$(echo "$out" | field text)" = "Airplane mode on · Wi-Fi and Bluetooth off" ] && [ "$(st nmcli.json wifi)" = "False" ] && [ "$(st bt.json powered)" = "False" ]' \
      "airplane mode on: Wi-Fi and Bluetooth off" "$out"
out=$("$BIN/ewe-companion" ask "airplane mode off")
check '[ "$(echo "$out" | field text)" = "Airplane mode off · Wi-Fi and Bluetooth on" ] && [ "$(st nmcli.json wifi)" = "True" ] && [ "$(st bt.json powered)" = "True" ]' \
      "airplane mode off: both back as they were" "$out"
out=$("$BIN/ewe-companion" ask "keep the screen awake")
check 'echo "$out" | grep -q "The screen stays awake" && [ "$(st insomnia.json on)" = "True" ] && [ -n "$(tok "$out")" ]' \
      "keep the screen awake → qs ipc call ewe.insomnia on, with Undo" "$out"
"$BIN/ewe-companion" debug undo "$(tok "$out")" >/dev/null
check '[ "$(st insomnia.json on)" = "False" ]' "…its Undo turns Insomnia off" ""
touch "$W/fakes/insomnia-gone"
out=$("$BIN/ewe-companion" ask "keep the screen awake")
check 'echo "$out" | grep -q "needs the Insomnia plugin" && [ "$(echo "$out" | js "r[\"action\"][\"label\"]")" = "Open Plugins" ]' \
      "no Insomnia plugin → says so, with Open Plugins" "$out"
rm -f "$W/fakes/insomnia-gone"
out=$("$BIN/ewe-companion" ask "install obs studio")
check '[ "$(echo "$out" | field text)" = "Opened Komble'"'"'s Discover, searching for obs studio" ] && grep -q "do open {\"what\": \"komble\", \"page\": \"search=obs studio\"}" "$W/island.log"' \
      "install obs studio → Komble's search (it only opens)" "$out"

# proposals: shown with Run, never run
out=$("$BIN/ewe-companion" ask "switch to light mode")
check '[ "$(echo "$out" | js "r[\"confirm\"][\"command\"]")" = "ewe-theme scheme apply ewe-light" ] && ! grep -q apply "$W/fakes/theme.log"' \
      "light mode → ewe-theme scheme apply ewe-light, offered with Run, not run" "$out"
echo '{"current": "ewe-light"}' >"$W/fakes/theme.json"
out=$("$BIN/ewe-companion" ask "switch to light mode")
check '[ "$(echo "$out" | field text)" = "Ewe Light is on: the desktop is already light" ] && [ "$(echo "$out" | js "r.get(\"confirm\")")" = "None" ]' \
      "light mode while light: said, nothing offered" "$out"
out=$("$BIN/ewe-companion" ask "restart the computer")
check '[ "$(echo "$out" | js "r[\"confirm\"][\"command\"]")" = "~/.config/hypr/scripts/power.sh reboot" ] && [ "$(echo "$out" | js "r[\"confirm\"][\"risk\"]")" = "danger" ]' \
      "restart → ewe's power.sh reboot, offered as risky" "$out"
out=$("$BIN/ewe-companion" ask "log out")
check '[ "$(echo "$out" | js "r[\"confirm\"][\"command\"]")" = "~/.config/hypr/scripts/power.sh logout" ] && [ "$(echo "$out" | js "r[\"confirm\"][\"risk\"]")" = "danger" ]' \
      "log out → power.sh logout, offered as risky" "$out"
out=$("$BIN/ewe-companion" ask "empty the trash")
check '[ "$(echo "$out" | field text)" = "The trash is already empty" ]' "an empty trash: said, nothing offered" "$out"
mkdir -p "$H/.local/share/Trash/files" && touch "$H/.local/share/Trash/files/old.txt"
out=$("$BIN/ewe-companion" ask "empty the trash")
check '[ "$(echo "$out" | js "r[\"confirm\"][\"command\"]")" = "gio trash --empty" ] && echo "$out" | grep -q "The 1 item in it goes for good" && [ ! -e "$W/fakes/gio.log" ]' \
      "empty the trash → gio trash --empty, offered with Run, not run" "$out"

# ── Shaun gets wiser: offline answers, the window manager, memory ─────────
# the clock and dates (the real clock; a zone from /usr/share/zoneinfo)
out=$("$BIN/ewe-companion" ask "what time is it")
check '[ "$(echo "$out" | field command)" = "clock" ] && echo "$out" | field text | grep -Eq "^It.s [0-9]{2}:[0-9]{2}, (Mon|Tues|Wednes|Thurs|Fri|Satur|Sun)day [0-9]{1,2} [A-Z][a-z]+ [0-9]{4}$"' \
      "what time is it → the clock, offline" "$out"
out=$("$BIN/ewe-companion" ask "what time is it in tokyo")
check 'echo "$out" | field text | grep -Eq "^In Tokyo it.s [0-9]{2}:[0-9]{2}, .*\(Asia/Tokyo, "' \
      "what time is it in tokyo → date with TZ=Asia/Tokyo in the child" "$out"
out=$("$BIN/ewe-companion" ask "what time is it in narniazz")
check '[ "$(echo "$out" | field text)" = "echo: what time is it in narniazz" ]' "a place with no zone goes to the model" "$out"
out=$("$BIN/ewe-companion" ask "how many days until christmas")
check '[ "$(echo "$out" | field command)" = "date_math" ] && echo "$out" | field text | grep -q "^Christmas is "' \
      "how many days until christmas → date_math" "$out"
out=$("$BIN/ewe-companion" ask "what's the date in 30 days")
check 'echo "$out" | field text | grep -q "^In 30 days it.s "' "the date in 30 days" "$out"
# units and sums; a currency is never guessed
out=$("$BIN/ewe-companion" ask "convert 10 km to miles")
check '[ "$(echo "$out" | field text)" = "10 km = 6.21 miles" ]' "convert 10 km to miles" "$out"
out=$("$BIN/ewe-companion" ask "20 c in f")
check '[ "$(echo "$out" | field text)" = "20 °C = 68 °F" ]' "20 c in f" "$out"
out=$("$BIN/ewe-companion" ask "100 usd in eur")
check '[ "$(echo "$out" | field text)" = "echo: 100 usd in eur" ]' "a currency goes to the model, never guessed" "$out"
out=$("$BIN/ewe-companion" ask "sqrt 144")
check '[ "$(echo "$out" | field text)" = "sqrt 144 = 12" ]' "sqrt 144" "$out"
# this computer, read for real (read-only)
out=$("$BIN/ewe-companion" ask "how much battery do i have")
check '[ "$(echo "$out" | field command)" = "system_info" ] && echo "$out" | field text | grep -Eq "^(Battery [0-9]+ %|This computer has no battery|There is a battery)"' \
      "battery → /sys/class/power_supply" "$out"
out=$("$BIN/ewe-companion" ask "how much ram is free")
check 'echo "$out" | field text | grep -Eq "^Memory: .* in use, .* free"' "memory → /proc/meminfo" "$out"
out=$("$BIN/ewe-companion" ask "what's using my memory")
check 'echo "$out" | field text | grep -q "uses the most memory" && [ "$(echo "$out" | js "len(r[\"rows\"])")" -ge 1 ]' \
      "what's using my memory → the programs, as rows" "$out"
check '[ "$(echo "$out" | js "r[\"rows\"][0][\"actions\"][0][\"command\"] + str(r[\"rows\"][0][\"actions\"][0][\"args\"][\"force\"])")" = "force_quitFalse" ]' \
      "…each program's row has Quit (force_quit, not forced)" "$out"
out=$("$BIN/ewe-companion" debug run force_quit '{"app": "firefox", "force": false}')
check '[ "$(echo "$out" | js "(r.get(\"confirm\") or {}).get(\"command\", \"\")")" = "pkill -x firefox" ]' \
      "…and Quit only offers pkill -x with Run: nothing is stopped by the click on the row" "$out"
out=$("$BIN/ewe-companion" ask "what's using my cpu")
check 'echo "$out" | field text | grep -q "^The processor is [0-9]* % busy"' "what's using my cpu → two samples of /proc" "$out"
out=$("$BIN/ewe-companion" ask "how long has my computer been on")
check 'echo "$out" | field text | grep -q "^Up .*, since "' "uptime" "$out"
out=$("$BIN/ewe-companion" ask "how much disk space is left")
check 'echo "$out" | field text | grep -q "% full"' "disk space left → statvfs" "$out"
out=$("$BIN/ewe-companion" ask "why is my laptop slow")
check '[ "$(echo "$out" | js "r[\"rows\"][0][\"title\"]")" = "Processor" ] && [ "$(echo "$out" | js "r[\"rows\"][1][\"title\"]")" = "Memory" ]' \
      "why is my laptop slow → the overview, offline" "$out"
# the network: nmcli and ip fakes; the network's name and the addresses stay
# on the island
out=$("$BIN/ewe-companion" ask "am i connected to the internet")
check 'echo "$out" | field text | grep -q "^You.re online, over Wi-Fi. Your local addresses are below; your public address would need asking a website" && echo "$out" | grep -q "192.168.1.23" && echo "$out" | grep -q HomeNet && ! echo "$out" | grep -q fe80' \
      "am I online → nmcli says how, ip -j addr the local addresses (no public IP)" "$out"
mkdir -p "$H/Downloads" && head -c 1000000 /dev/zero >"$H/Downloads/huge-zz-download.iso"
out=$("$BIN/ewe-companion" ask "what's eating my disk")
check 'echo "$out" | field text | grep -q "^What takes space, biggest first: " && echo "$out" | grep -q "huge-zz-download.iso" && echo "$out" | js "[x[\"title\"] for x in r[\"rows\"]]" | grep -q "The trash"' \
      "what's eating my disk → the categories and Downloads' biggest files" "$out"
out=$("$BIN/ewe-companion" ask "what did I ask you just now")
check 'python3 -c "import json,sys; b=[json.loads(l) for l in open(sys.argv[1])][-1][\"body\"]; t=json.dumps(b[\"messages\"][1:]); sys.exit(0 if \"HomeNet\" not in t and \"192.168.1.23\" not in t and \"huge-zz-download\" not in t and \"Disk use, biggest first\" in t else 1)" "$MOCK_LOG"' \
      "the network's name, the addresses and Downloads' files never reach the model" "$(tail -c 400 "$MOCK_LOG")"
# workspaces and windows: the hyprctl fake, with the island's signature
hlast() { tail -n 1 "$W/fakes/hyprctl.log"; }
hst() { python3 -c 'import json,sys; s=json.load(open(sys.argv[1])); print(eval(sys.argv[2], {"s": s}))' "$W/fakes/hypr.json" "$1"; }
out=$("$BIN/ewe-companion" ask "switch to workspace 3")
check '[ "$(echo "$out" | field text)" = "Workspace 3" ] && hlast | grep -q "\"hl.dsp.focus({ workspace = 3 })\"" && hlast | grep -q "\"sig\": \"fake-hypr-sig\""' \
      "switch to workspace 3 → hl.dsp.focus, with the island's HYPRLAND_INSTANCE_SIGNATURE" "$out"
out=$("$BIN/ewe-companion" debug undo "$(tok "$out")")
check '[ "$(echo "$out" | field text)" = "Workspace 1" ]' "…its Undo goes back to workspace 1" "$out"
out=$("$BIN/ewe-companion" ask "next workspace")
check '[ "$(echo "$out" | field text)" = "Workspace 2" ]' "next workspace → the next one in use on this display" "$out"
out=$("$BIN/ewe-companion" ask "swich to worksapce 4")
check '[ "$(echo "$out" | field text)" = "Workspace 4" ]' "a typo gets a second try (swich to worksapce 4)" "$out"
"$BIN/ewe-companion" ask "workspace 1" >/dev/null
out=$("$BIN/ewe-companion" ask "move this window to workspace 2")
check '[ "$(echo "$out" | field text)" = "Moved Firefox to workspace 2" ] && hlast | grep -q "address:0xaaa1.*follow = false" && [ "$(hst "s[\"clients\"][0][\"workspace\"][\"id\"]")" = "2" ]' \
      "move this window to workspace 2 → by address, not following; the line names the app" "$out"
"$BIN/ewe-companion" debug undo "$(tok "$out")" >/dev/null
check '[ "$(hst "s[\"clients\"][0][\"workspace\"][\"id\"]")" = "1" ]' "…its Undo moves it back" "$(hlast)"
out=$("$BIN/ewe-companion" ask "make this window fullscreen")
check '[ "$(echo "$out" | field text)" = "Firefox is fullscreen" ] && [ "$(hst "s[\"clients\"][0][\"fullscreen\"]")" = "2" ]' \
      "make this window fullscreen → mode 0, set" "$out"
utok=$(tok "$out")
out=$("$BIN/ewe-companion" ask "fullscreen")
check '[ "$(echo "$out" | field text)" = "Firefox is fullscreen already" ]' "…already: said, nothing toggled" "$out"
"$BIN/ewe-companion" debug undo "$utok" >/dev/null
check '[ "$(hst "s[\"clients\"][0][\"fullscreen\"]")" = "0" ]' "…its Undo unsets it" "$(hlast)"
out=$("$BIN/ewe-companion" ask "float this window")
check '[ "$(echo "$out" | field text)" = "Firefox floats" ] && [ "$(hst "s[\"clients\"][0][\"floating\"]")" = "True" ]' "float this window" "$out"
"$BIN/ewe-companion" debug undo "$(tok "$out")" >/dev/null
out=$("$BIN/ewe-companion" ask "minimise this window")
check 'echo "$out" | field text | grep -q "^Minimised Firefox" && [ "$(hst "s[\"clients\"][0][\"workspace\"][\"name\"]")" = "special:minimized" ]' \
      "minimise this window → special:minimized" "$out"
out=$("$BIN/ewe-companion" ask "bring back the minimized window")
check '[ "$(echo "$out" | field text)" = "Brought back Firefox" ] && [ "$(hst "s[\"clients\"][0][\"workspace\"][\"id\"]")" = "1" ]' \
      "bring back the minimized window → onto the workspace in focus" "$out"
# closing asks first; only the button's one-time token closes
out=$("$BIN/ewe-companion" ask "close this window")
check '[ "$(echo "$out" | js "r[\"action\"][\"label\"]")" = "Close Firefox" ] && [ "$(echo "$out" | field kind)" = "answer" ] && ! grep -q "window.close" "$W/fakes/hyprctl.log"' \
      "close this window → a Close Firefox button (an offer, not done); nothing closed yet" "$out"
close=$(echo "$out" | js 'json.dumps(r["action"]["args"])')
out=$("$BIN/ewe-companion" debug run window '{"action": "close"}')
check '[ "$(echo "$out" | js "r[\"action\"][\"label\"]")" = "Close Firefox" ] && ! grep -q "window.close" "$W/fakes/hyprctl.log"' \
      "a model's window close (no token) only offers the button" "$out"
out=$("$BIN/ewe-companion" debug run window '{"action": "close", "token": "0123456789abcdef01234567"}')
check '! grep -q "window.close" "$W/fakes/hyprctl.log"' "…and a made-up token closes nothing" "$out"
out=$("$BIN/ewe-companion" debug run window "$close")
check '[ "$(echo "$out" | field text)" = "Closed Firefox" ] && hlast | grep -q "hl.dsp.window.close({ window = .*address:0xaaa1"' \
      "the button's click closes it (gracefully, by address)" "$out"
"$BIN/ewe-companion" debug run window "$close" >/dev/null
check '[ "$(grep -c "window.close" "$W/fakes/hyprctl.log")" = "1" ]' "…once" "$(grep window.close "$W/fakes/hyprctl.log")"
out=$("$BIN/ewe-companion" ask "close firefox")
check '[ "$(echo "$out" | js "r[\"action\"][\"label\"]")" = "Close Firefox" ] && ! echo "$out" | grep -q Recipes' \
      "close firefox → a button for its window; the title never shows" "$out"
# a frozen app: pkill shown with Run, never run
out=$("$BIN/ewe-companion" ask "firefox is frozen")
check '[ "$(echo "$out" | js "r[\"confirm\"][\"command\"]")" = "pkill -KILL -x firefox" ] && [ "$(echo "$out" | js "r[\"confirm\"][\"risk\"]")" = "danger" ] && kill -0 "$FP"' \
      "firefox is frozen → pkill -KILL -x firefox offered with Run (risky), not run" "$out"
out=$("$BIN/ewe-companion" ask "kill firefox")
check '[ "$(echo "$out" | js "r[\"confirm\"][\"command\"]")" = "pkill -x firefox" ] && kill -0 "$FP"' "kill firefox → pkill -x firefox, offered" "$out"
out=$("$BIN/ewe-companion" ask "kill nosuchappzz")
check '[ "$(echo "$out" | field text)" = "echo: kill nosuchappzz" ]' "an app that is not running goes to the model" "$out"
out=$("$BIN/ewe-companion" ask "opne firefox")
check '[ "$(echo "$out" | field command)" = "launch_app" ] && grep -q "do launch {\"app\": \"firefox\"" "$W/island.log"' \
      "a typo gets a second try (opne firefox)" "$out"
# memory: kept in companion.db, never a secret
out=$("$BIN/ewe-companion" ask "remember that my dentist is on friday")
check '[ "$(echo "$out" | field text)" = "I'"'"'ll remember that: my dentist is on friday" ] && [ -n "$(tok "$out")" ]' \
      "remember that … → kept, with Undo" "$out"
out=$("$BIN/ewe-companion" ask "remember my pin is 90817263")
check '[ "$(echo "$out" | field kind)" = "error" ] && echo "$out" | grep -q "looks like a secret. ewe Safe is the place for it"' \
      "a PIN is refused: ewe Safe is the place for it" "$out"
out=$("$BIN/ewe-companion" ask "what do you remember")
check '[ "$(echo "$out" | js "r[\"rows\"][0][\"title\"]")" = "my dentist is on friday" ] && [ "$(echo "$out" | js "r[\"rows\"][0][\"actions\"][0][\"label\"]")" = "Forget" ] && [ "$(echo "$out" | js "len(r[\"rows\"])")" = "1" ]' \
      "what do you remember → rows with Forget (the PIN was never kept)" "$out"
check '! cat "$W/state/ewe/companion/"companion.db* 2>/dev/null | grep -qa 90817263' "the refused PIN is not in the store" ""
out=$("$BIN/ewe-companion" ask "forget about my dentist")
check '[ "$(echo "$out" | field text)" = "Forgot: my dentist is on friday" ]' "forget about my dentist" "$out"
out=$("$BIN/ewe-companion" debug undo "$(tok "$out")")
check '[ "$(echo "$out" | field text)" = "I'"'"'ll remember that: my dentist is on friday" ]' "…its Undo puts it back" "$out"
out=$("$BIN/ewe-companion" ask "forget everything")
check '[ "$(echo "$out" | js "r[\"action\"][\"label\"]")" = "Forget it" ] && [ "$("$BIN/ewe-companion" ask "what do you remember" | js "len(r[\"rows\"] or [])")" = "1" ]' \
      "forget everything → asks first (a button), forgets nothing yet" "$out"
all=$(echo "$out" | js 'json.dumps(r["action"]["args"])')
out=$("$BIN/ewe-companion" debug run forget "$all")
check '[ "$(echo "$out" | field text)" = "Forgot the one thing I remembered." ]' "…its click forgets it all" "$out"
out=$("$BIN/ewe-companion" ask "remember to call mum at 5pm")
check '[ "$(echo "$out" | field command)" = "timer" ] && echo "$out" | grep -q "I.ll remind you at 17:00: call mum"' \
      "remember to … at 5pm is a reminder, not a memory" "$out"
"$BIN/ewe-companion" debug undo "$(tok "$out")" >/dev/null

# timers: a notification when due (a fake notify-send), cancellable, listed
out=$("$BIN/ewe-companion" ask "set a timer for 1 second")
check 'echo "$out" | grep -q "Timer set for 1 second" && [ -n "$(tok "$out")" ]' "a timer for 1 second, with Undo" "$out"
for _ in $(seq 50); do grep -q "1 second is up" "$W/fakes/notify.log" 2>/dev/null && break; sleep 0.1; done
check 'grep -q "\"--app-name=Shaun\".*\"Timer\", \"1 second is up.\"" "$W/fakes/notify.log"' \
      "…when due: a notification from Shaun" "$(cat "$W/fakes/notify.log" 2>/dev/null)"
out=$("$BIN/ewe-companion" ask "remind me in 10 minutes to water the plants")
check 'echo "$out" | grep -q "I.ll remind you in 10 minutes (.*): water the plants"' "a reminder in 10 minutes" "$out"
rtok=$(tok "$out")
out=$("$BIN/ewe-companion" ask "show my timers")
check '[ "$(echo "$out" | js "r[\"rows\"][0][\"title\"]")" = "water the plants" ] && [ "$(echo "$out" | js "r[\"rows\"][0][\"actions\"][0][\"label\"]")" = "Cancel" ]' \
      "the timers → rows with Cancel" "$out"
out=$("$BIN/ewe-companion" debug undo "$rtok")
check '[ "$(echo "$out" | field text)" = "Cancelled the reminder: water the plants" ]' "…the reminder's Undo cancels it" "$out"
out=$("$BIN/ewe-companion" ask "show my timers")
check '[ "$(echo "$out" | field text)" = "No timers or reminders are waiting." ]' "…and nothing waits any more" "$out"
out=$("$BIN/ewe-companion" ask "remind me in 5 minutes")
check 'echo "$out" | grep -q "I.ll remind you in 5 minutes (" && ! echo "$out" | grep -qi ": reminder"' \
      "a bare reminder: no made-up words" "$out"
"$BIN/ewe-companion" debug undo "$(tok "$out")" >/dev/null
# reminders survive a daemon restart (companion.db): the ones that fell due
# while the daemon was down fire at its next start, as ONE notification
"$BIN/ewe-companion" ask "remind me in 2 seconds to stretch" >/dev/null
"$BIN/ewe-companion" ask "remind me in 2 seconds to drink water" >/dev/null
kill "$DP"; wait "$DP" 2>/dev/null; sleep 3
check '! grep -q stretch "$W/fakes/notify.log"' "…the daemon stopped before the reminders were due" ""
daemon
for _ in $(seq 100); do grep -q "stretch" "$W/fakes/notify.log" 2>/dev/null && break; sleep 0.1; done
sleep 0.5
check '[ "$(grep -c "stretch\|drink water" "$W/fakes/notify.log")" = "1" ] && grep -q "\"Timers and reminders\", \"2 are due: stretch; drink water.\"" "$W/fakes/notify.log"' \
      "two reminders due while the daemon was down fire at its next start, as one notification" "$(tail -n 2 "$W/fakes/notify.log")"
# at most 50 wait at once
for _ in $(seq 50); do "$BIN/ewe-companion" debug run timer '{"seconds": 3600}' >/dev/null; done
out=$("$BIN/ewe-companion" ask "set a timer for 5 minutes")
check '[ "$(echo "$out" | field kind)" = "error" ] && echo "$out" | grep -q "50 timers and reminders are waiting already"' \
      "the 51st timer is refused" "$out"

echo
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
