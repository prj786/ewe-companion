# Testing the companion

How to check a change, from the unit tests up to the owner's desktop. Read
`AGENTS.md` first. The short version, after any change:

```sh
cargo build && cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all --check
tests/chat.sh && tests/relay-failsafe.sh && tests/canary.sh   # target/debug as it is: build first
tests/island.sh up --scale 1.8             # when anything visible changed: look at it
```

| layer | what | when |
|---|---|---|
| unit | `cargo test`: core (the protocol, the hook dialect, the matcher, the catalog, the pack, the guard) and the daemon | every change; CI |
| chat | `tests/chat.sh`: the chat path end to end against `tests/mock-llm.py`; Safe, ewe browser and the finder against `tests/fakes/` and a scripted island | chat, commands, providers, the pack, proposals, Safe, the browser, the finder; CI |
| canary | `tests/canary.sh`: Safe's names reach no provider, log or store | anything that touches Safe, the conversation, logs or the store; CI |
| relay | `tests/relay-failsafe.sh`: the hook relay's fail-safes over a real socket | the relay, cards, the hook dialect; CI |
| island | `tests/island.sh`: the plugin in ewe's nested Hyprland, screenshots, a virtual mouse | anything the island draws or does |
| real agent | `tests/island.sh claude …`: a real Claude Code session in a scratch project | hook changes, card wording |
| desktop | the owner's own session, on request | when the owner wants to try it |

## Careful: the test daemon acts for real

A debug daemon (`EWE_COMPANION_DEBUG=1`) runs on its own socket and state
dir, but the commands it runs act on **this machine**:

- volume, mute, brightness, media keys: `wpctl`, `brightnessctl`,
  `playerctl` on the owner's session (a typed "mute" once muted the owner's
  speakers);
- game mode: `gamemoded` and `hyprctl` against the compositor in `env.hypr`;
- lock ("lock", the Lock quick action): `loginctl lock-session` locks the
  owner's real screen;
- the Run click: the command, in the owner's home.

What the nested shell performs stays in the sandbox (opening Settings,
launching apps, Do not disturb, screenshots). Test with those, or with
`$ echo …` for Run, and put back anything you changed at once.

Wave 2's switches never reach the machine in a test: `tests/chat.sh`,
`tests/canary.sh` and `tests/island.sh` start the daemon with
`tests/fakes` first on its PATH, so Wi-Fi (`nmcli`), Bluetooth (`ewe-bt`),
Insomnia (`qs ipc`), a timer's `notify-send`, the theme (`ewe-theme`) and
the trash (`gio`) are fakes. The two scripts refuse to start when a shim is
not first on the PATH. Dark or light mode, power and the trash are
proposals: a test checks the command offered, never presses Run on it.

Also never:

- point a test at the owner's `~/.claude` (hooks are tested in a scratch
  project, below);
- point a test at the real `ewe-safe` or `ewe-browser`: they reach the
  owner's real safe and browser profile. Every script and the harness use
  `tests/fakes/` (below);
- run `colorscheme.sh` on the dev host;
- `pkill -f`/`pgrep -f` a pattern that is in your own command line (it
  matches your own shell); find a PID and check `readlink /proc/PID/exe`.

## Unit tests

`cargo test`. Pure rules live in `companion-core`, so most tests are there,
next to the code, in a `#[cfg(test)] mod tests`. Names are sentences
(`machine_killers_are_refused`, `questions_are_not_commands`): say what
must hold.

Tests that guard data, and fail when you add without finishing:

- `every_catalog_entry_validates_a_good_call`: one good call per catalog
  row (the count must match);
- `the_built_in_pack_parses`, `every_open_line_is_a_valid_command`,
  `every_run_line_is_one_the_guard_lets_through`: the knowledge pack;
- `questions_find_their_topic`, `small_talk_finds_nothing_confident`: the
  search, with real questions (add yours).

## tests/chat.sh

A debug daemon on a temp socket and `tests/mock-llm.py` (stdlib, an
OpenAI-compatible server on a random port), then `ewe-companion ask …` and
`debug confirm …`, checking each JSON reply:

```sh
out=$("$BIN/ewe-companion" ask "how do I change my wallpaper?")
check '[ "$(echo "$out" | field kind)" = "answer" ]' "the pack answers a wallpaper question" "$out"
```

`check '<shell condition>' "<what must hold>" "<what to show on failure>"`;
`field k` reads a top-level field; `js '<python expr over r>'` reads deeper.
The mock answers "4" to a sum, an `ACTION` line to "quiet" (dnd) and "make
it light" (a `run_shell` proposal), and echoes anything else; every request
is logged to `$MOCK_LOG` with the key written as `[auth]`, so a check can
prove what a provider was sent (and that no key reached a body). Teach the
mock a new reply when a feature needs one.

Only add checks for commands that are safe to run here (see *Careful*).

**Safe, ewe browser and the finder** (the second half of the script) never
touch the real ones:

- `tests/fakes/ewe-safe` and `tests/fakes/ewe-browser` stand in for the
  real command lines (copied into the work dir, so a check may change
  them). The daemon finds them through `EWE_COMPANION_SAFE` and
  `EWE_COMPANION_BROWSER`. They keep state and a log of every call in
  `$EWE_FAKE_DIR` (`safe.json`, `safe.log`, `browser.log`); a file
  `browser-old` there makes the browser answer like 0.1.0-beta (no command
  line), `browser-v03` like 0.3 (no site search, no workspaces, back,
  forward or download verbs), `browser-closed` makes the download verbs
  answer not-running, `browser-lies` reports closing a tab as tier 1, and
  touching the fake makes the daemon ask again; a file `safe-gone` makes Safe answer like a service that went
  away. The browser fake knows Contract 1 of wave 2 as the wave plan
  writes it (`search --site` with `unknown-site`, `workspace …`, `back`,
  `forward`, `download open|show`, `open --workspace`).
- PATH shims, also in `tests/fakes/`: `nmcli` (`nmcli.json`: Wi-Fi on or
  off), `ewe-bt` (`bt.json`: the adapter and three paired devices; a file
  `bt-none` means no adapter), `qs` (only `ipc call ewe.insomnia …`,
  `insomnia.json`; a file `insomnia-gone` means no plugin), `notify-send`
  (`notify.log`), `ewe-theme` (`scheme list`; `theme.json` names the
  current scheme) and `gio` (`gio.log`: a line there means a proposal ran,
  which a test must never do). Each logs its argv to `<name>.log`.
- `$FAKE_BROWSER_MARK` is added to every title, name and file name the
  browser and ewe-bt fakes hold (`tests/canary.sh` sets it to the canary).
- `tests/fake-island.py --do` answers the daemon's `do` requests: it runs
  `browser` and `safe_unlock` with the binary the daemon names (the fakes)
  and answers the rest ok, printing `do …` and `chat.reply …` lines.
- The daemon runs with `HOME` set to a scratch tree, so the finder's
  answers are known.
- `debug run <command> <args-json>` stands in for a button's click (a row's
  Copy, Unlock Safe), and `debug undo <token>` for the Undo under a reply.
- The last check stops the daemon with a reminder pending and starts it
  again: the reminder must fire from `companion.db`.

## tests/canary.sh

PLAN §16's canary for Safe (TRIO.md: names stay local). The fake Safe's
titles, usernames, folders and sites carry `EWE-CANARY-7Q2X`
(`FAKE_SAFE_ITEMS`); 22 scripted chats ask Safe for things through the
matcher, a row's click, a model's ACTION, a locked safe and its unlock, try
two commands that would print what Safe holds (refused, never run), then
ask the model about it. Since wave 2 the browser's tabs, bookmarks,
downloads and workspaces and ewe-bt's devices carry it too
(`FAKE_BROWSER_MARK`), with 19 more chats and a row's click, and the
canary must reach the island from them as well. The canary must reach the island (or the test proves
nothing), and must be in no request the mock provider got, no line of the
daemon's log at `EWE_COMPANION_LOG=debug`, and no byte of `companion.db`,
its WAL or any table row. Run it after any change to the chat, Safe, the
logs or the store.

## tests/relay-failsafe.sh

The relay's one rule: on any failure it prints nothing and exits 0 (C2),
and it prints an allow only for a human's answer (C1). The script runs a
debug daemon and `tests/fake-island.py` (a scripted island: `--ack`,
`--answer allow|deny|always:0|terminal`, `--answers JSON`, `--delay S`,
`--env JSON`) and feeds `tests/payloads/claude/*.json` to `ewe-companion
hook claude`. It covers no socket, a socket dir others can write, garbage
on stdin, no island, no ack, each answer's exact JSON, an Always Claude did
not offer, AskUserQuestion answers, a 2 MB payload, the relay hanging up,
the daemon or the island going away mid-request, a timeout
(`EWE_COMPANION_WAIT_MS`), the screen locked, sessions and the Stop peek,
and the CLI's rule 5.

`BIN=target/release tests/relay-failsafe.sh` runs it against a release
build (do this before deploying: the timing differs).

**New payloads.** `captured-*.json` came from a real session: set
`EWE_COMPANION_HOOK_LOG=<file>` in the scratch project's environment and the
relay appends every payload it sees. Strip anything personal (paths become
`/home/u/…`) before committing.

## tests/island.sh: the island without installing anything

It builds the debug binaries, starts a debug daemon on
`$XDG_RUNTIME_DIR/ewe-companion-island.sock` with its state under
`$ISLAND_WORK/state` (Safe and ewe browser are the fakes in `tests/fakes/`,
their state in `$ISLAND_WORK/fakes`, unless `EWE_COMPANION_SAFE` /
`EWE_COMPANION_BROWSER` name others), copies `../ewe-plugin-companion` into a scratch payload
and runs ewe's nested harness (`../ewe/.claude/skills/run-ewe/driver.sh`)
with it: a sandboxed shell on a headless output called SHOT. The live shell,
the live daemon, `~/.config` and `~/.claude` are never touched, and the
nested shell cannot reach the host's systemd units (`HS_NO_SYSTEMCTL`).

```sh
tests/island.sh up --scale 1.8          # the owner's laptop: 2880x1800 at 1.8
tests/island.sh ctl debug card permission     # or question / note
tests/island.sh shot card               # → $ISLAND_WORK/card.png; read it
tests/island.sh cards                   # waiting cards: id kind source
tests/island.sh ctl debug answer <id> deny
tests/island.sh ipc open                # the home panel (Super+A)
tests/island.sh type --enter "open kitty in documents"
tests/island.sh ctl say "Download done · archlinux.iso" --level success
tests/island.sh log | grep -i companion
tests/island.sh down
```

Options for `up`: `--light` (Ewe Light), `--glass` (bar opacity 80, so the
notch's blur shows), `--scale S` and `--mode WxH` (a hi-DPI output; bugs like
pixelated curves only show at the owner's 1.8), `--conf FILE` (a whole
`ewe.conf` of your own; it must enable the companion's settings, e.g.
`[plugins.settings] "ewe.companion" = {screen = "all"}`).

After a QML edit, `tests/island.sh sync` copies the plugin again and restarts
the nested shell (the daemon keeps its state; the island reconnects). After a
Rust edit, `tests/island.sh restart`. To try an edited knowledge pack, export
`EWE_COMPANION_KNOWLEDGE=$PWD/knowledge` before `up` or `restart`: the daemon
reads the pack once, so every pack edit needs a `restart`.

Defaults: `EWE_DIR=../ewe`, `PLUGIN_DIR=../ewe-plugin-companion`, and
`ISLAND_WORK` the work dir of the last `up` (else
`/tmp/ewe-companion-island`), so a later call without it finds the run. Each
work dir has its own socket. With two runs up at once (two sessions), give
`ISLAND_WORK` on every call. Agents: put it in your scratchpad, on every
call (shell variables do not survive between tool calls).

Needs: the ewe checkout's harness (Hyprland, `qs`, `grim`, `hyprctl`),
`wtype` for `type` and `key`, `python3`, and python-pillow for `measure`
and the numbers `hover` prints.

`cards` reads the scratch store: after a `restart`, cards the old daemon
held are still listed but no longer answerable.

**Reading shots.** A shot is the whole SHOT output at device pixels
(2880x1800 at 1.8). The notch hangs from the bar's centre. A suspiciously
small PNG (about 15 KB at 1920x1200) is a bare bar: the shell is up but the
plugin drew nothing; read `tests/island.sh log`. Always open and look
at the image before reporting what it shows.

### Hover, clicks and scrolling: the virtual mouse

The headless output has no pointer, so nothing ever hovers.
`tests/island.sh hover STEPS` adds one (`tests/vptr.py`, a stdlib
`zwlr_virtual_pointer_v1` client) and plays a steps file:

```
# a "#" starts a comment
warp 2550 86      # move to (x, y) in the nested layout's logical pixels
nudge             # a real motion event there (a warp alone hovers nothing)
sleep 0.6
mark icon1        # shot → $ISLAND_WORK/icon1.png, and the notch's bottom row
click
wheel 3           # three notches down (negative: up)
```

**Coordinates.** SHOT sits at x = 1920 in the nested layout. A point at
pixel (px, py) of a shot at scale S is `warp (1920 + px/S) (py/S)`. At 1.8:
the bar's centre is about (2720, 15), and the first quick-action icon of the
open notch about (2550, 86).

**Measuring.** `mark` (and `tests/island.sh measure PNG…`, needs
python-pillow) prints the lowest row that differs from the background down
the screen's centre line: the notch's bottom edge. Equal numbers across
marks mean the notch did not change size, which is how the hover bug
(2026-10-14) was proven fixed.

## A real Claude Code session

The hook path against the real thing, in a scratch project, never the
owner's `~/.claude`:

```sh
tests/island.sh up
tests/island.sh claude-project "$ISLAND_WORK/proj"     # hooks → this checkout's relay
tests/island.sh claude "$ISLAND_WORK/proj" "Create hello.txt containing hi. Do nothing else." \
  > "$ISLAND_WORK/claude.out" 2>&1 &
tests/island.sh cards                                   # a few seconds later: the card
tests/island.sh shot real-card
tests/island.sh ctl debug answer <id> allow             # the click
cat "$ISLAND_WORK/claude.out"                           # Claude's answer once it finishes
```

- `claude-project` writes `.claude/settings.local.json` from
  `integrations/claude-code`'s hooks with this checkout's binary, and turns
  the installed plugin (`ewe-companion@ewe`, from `agents install claude`)
  off for that project. Without that, both relays reach the scratch daemon
  and every card comes twice.
- `claude` runs `claude -p --permission-mode default --model haiku` with
  the scratch socket and without the parent session's `CLAUDECODE` and
  `CLAUDE_CODE_*` variables. It blocks while a card waits (up to 110 s, then
  Claude asks in its own way), so run it in the background.
- **Permission mode matters.** `bypassPermissions` (the owner's default)
  fires no `PermissionRequest`, so no card. `default` does.
- **Questions:** prompt Claude to ask you something with AskUserQuestion,
  then `ctl debug answer <id> answer '{"<question>": "<label>"}'` (the
  answers are the fourth word, JSON: a label, or a list of labels for a
  multi-select).

## The owner's desktop

Only when the owner asks to try a change (it restarts what they are using):

```sh
cargo build --release && BIN=target/release tests/relay-failsafe.sh
install -m755 target/release/ewe-companiond target/release/ewe-companion ~/.local/bin/
systemctl --user restart ewe-companion.service
systemctl --user restart ewe.service      # after QML changes only: the whole shell restarts
ewe-companion status                      # "island": true once the shell is back
journalctl --user -u ewe-companion -n 50  # the daemon's log: types, ids, timings
journalctl --user -u ewe -n 200 | grep -i companion    # the island's log lines
```

The owner's plugin is a symlink to `../ewe-plugin-companion`, so the shell
restart picks up the checkout as it is. Their settings: Komble → Plugins →
Companion → Options (`ewe.conf` `[plugins.settings]`).

## CI

`.github/workflows/ci.yml` on every push to `main` and every PR: fmt,
clippy `-D warnings --locked`, `cargo test --locked`, build,
`tests/relay-failsafe.sh`, `tests/chat.sh`, `tests/canary.sh`. The plugin repo has no CI; its
check is the island harness.
