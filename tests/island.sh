#!/usr/bin/env bash
# tests/island.sh: try the island (plugin ewe.companion) without installing
# anything. A debug daemon on its own socket, and ewe's nested Hyprland
# harness (ewe/.claude/skills/run-ewe/driver.sh) running a sandboxed shell
# with this checkout's plugin in a scratch payload. The live shell, the live
# daemon, ~/.config and ~/.claude are never touched.
#
# CAREFUL: the scratch daemon still acts on THIS machine. "volume 60",
# "mute", "brightness up", media keys, game mode and "lock" run wpctl,
# brightnessctl, playerctl, gamemoded and loginctl for real (a typed "mute"
# once muted the owner's speakers; "lock" or the Lock button locks the real
# screen). A Run click runs the command for real. Test with commands that
# only the nested shell performs (open, launch, dnd, screenshot) and
# `$ echo …` for Run, or put back what you changed at once.
#
#   tests/island.sh up [--light] [--glass] [--scale 1.8] [--mode 2880x1800] [--conf FILE]
#                                   build, start the daemon and the nested shell
#   tests/island.sh sync            copy the plugin again, restart the nested shell
#                                   (the daemon keeps running; the island reconnects)
#   tests/island.sh restart         rebuild and restart the daemon (after Rust changes)
#   tests/island.sh ctl ARGS…       ewe-companion against the scratch daemon:
#                                     ctl debug card permission|question|note
#                                     ctl say "Download done" --level success
#                                     ctl ask "open kitty in documents"
#                                     ctl debug confirm <token> [run|terminal]
#   tests/island.sh cards           the cards waiting now (id kind source), from the
#                                   scratch store: the ids ctl debug answer takes (after a
#                                   restart, cards the old daemon held are listed but gone)
#   tests/island.sh ipc FN [ARG]    qs ipc call ewe.companion FN in the nested shell
#                                   (toggle, open, hide, status, ask "text")
#   tests/island.sh shot NAME       screenshot → $ISLAND_WORK/NAME.png (prints the path)
#   tests/island.sh type TEXT…      type into the focused field; --enter presses Return after
#   tests/island.sh key NAME        one key: Return, Escape, Tab …
#   tests/island.sh hover STEPS     a virtual mouse (tests/vptr.py); every `mark NAME`
#                                   in STEPS takes a shot and measures the notch
#   tests/island.sh measure PNG…    the notch's lowest row in pixels (needs python-pillow)
#   tests/island.sh log             the daemon's log, then the shell's
#   tests/island.sh claude-project DIR
#                                   a scratch project whose hooks run this checkout's
#                                   relay (.claude/settings.local.json from
#                                   integrations/claude-code); never the real ~/.claude
#   tests/island.sh claude DIR PROMPT…
#                                   claude -p --permission-mode default --model haiku in
#                                   DIR, against the scratch daemon. It blocks while a card
#                                   waits: run it in the background and answer with
#                                   ctl debug answer <card id> allow|deny
#   tests/island.sh down            stop both
#
# Pointer coordinates: the headless output SHOT sits at x = 1920 in the
# nested layout, so a point on it is (1920 + x, y) in logical pixels. At
# --scale 1.8 with 2880x1800 it is 1600x1000 logical: the bar's centre is
# about (2720, 15).
#
# Environment: EWE_DIR (an ewe checkout, default ../ewe), PLUGIN_DIR (default
# ../ewe-plugin-companion), ISLAND_WORK (default: the work dir of the last
# `up`, else /tmp/ewe-companion-island; give it on every call when two runs
# are up at once), EWE_COMPANION_KNOWLEDGE (an edited knowledge pack, passed
# to the daemon). Each work dir gets its own socket. Safe and ewe browser
# are the fakes in tests/fakes/ (state in $ISLAND_WORK/fakes) unless
# EWE_COMPANION_SAFE / EWE_COMPANION_BROWSER name others.
#
# Needs: the ewe checkout's harness (Hyprland, qs, grim, hyprctl), wtype for
# `type` and `key`, python3, and python-pillow for `measure` and `hover`'s
# measurements.
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(dirname "$HERE")"
EWE_DIR="${EWE_DIR:-$(cd "$REPO/.." && pwd)/ewe}"
PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$REPO/.." && pwd)/ewe-plugin-companion}"
# the work dir: ISLAND_WORK, else the last `up`'s (so a later call without it
# still finds the run), else the default
LAST="${XDG_RUNTIME_DIR:?no XDG_RUNTIME_DIR}/ewe-companion-island.last"
if [ -n "${ISLAND_WORK:-}" ]; then WORK="$ISLAND_WORK"
elif [ "${1:-}" != up ] && [ -s "$LAST" ]; then WORK="$(cat "$LAST")"
else WORK=/tmp/ewe-companion-island; fi
# a Unix socket path holds 108 bytes at most: keep it short, in the runtime
# dir, one per work dir so two runs never share a daemon
SOCK="$XDG_RUNTIME_DIR/ewe-companion-island-$(printf %s "$WORK" | md5sum | cut -c1-8).sock"
DRV="$EWE_DIR/.claude/skills/run-ewe/driver.sh"
BIN="$REPO/target/debug"
export HS_WORK="$WORK/hs" HS_OUT="$WORK"

die() { echo "island: $*" >&2; exit 1; }
drv() { "$DRV" "$@"; }
nest() { [ -f "$HS_WORK/state" ] || die "not up: tests/island.sh up"; . "$HS_WORK/state"; }

# the scratch daemon, found by its pid file and checked by its binary
# (after a rebuild the old binary is gone and its exe link ends in
# " (deleted)": without stripping it, `restart` left the old daemon running
# and the new one stepped aside, 2026-10-17)
daemon_pid() {
  local p exe
  p="$(cat "$WORK/daemon.pid" 2>/dev/null)" || return 1
  exe="$(readlink "/proc/$p/exe" 2>/dev/null)" || return 1
  [ "${exe% (deleted)}" = "$BIN/ewe-companiond" ] && echo "$p"
}
daemon_stop() {
  local p
  p="$(daemon_pid)" && kill "$p" 2>/dev/null
  rm -f "$WORK/daemon.pid"
}
daemon_start() {
  daemon_stop
  mkdir -p "$WORK/state" "$WORK/fakes"
  # Safe and ewe browser are the fakes in tests/fakes/ unless named: a test
  # never reaches the owner's real safe or browser profile
  export EWE_COMPANION_SAFE="${EWE_COMPANION_SAFE:-$HERE/fakes/ewe-safe}" \
         EWE_COMPANION_BROWSER="${EWE_COMPANION_BROWSER:-$HERE/fakes/ewe-browser}" \
         EWE_FAKE_DIR="${EWE_FAKE_DIR:-$WORK/fakes}"
  # env -u: the daemon must not reach the host's compositor until the
  # island's `env` names the nested one. PATH starts with tests/fakes: Wi-Fi,
  # Bluetooth, Insomnia (`qs ipc`), timers' notify-send, the theme and the
  # trash reach the fakes, never the owner's radios or live shell
  env -u HYPRLAND_INSTANCE_SIGNATURE EWE_COMPANION_DEBUG=1 EWE_COMPANION_SOCK="$SOCK" \
    PATH="$HERE/fakes:$PATH" XDG_STATE_HOME="$WORK/state" setsid "$BIN/ewe-companiond" >"$WORK/daemon.log" 2>&1 </dev/null &
  echo $! >"$WORK/daemon.pid"
  for _ in $(seq 100); do [ -S "$SOCK" ] && return 0; sleep 0.05; done
  die "the daemon did not start: $WORK/daemon.log"
}

build() { (cd "$REPO" && cargo build -q) || die "cargo build failed"; }

# the plugin goes into a scratch payload: the reserved ewe. id loads only
# from a payload, never as a hand-made plugin
payload() {
  rm -rf "$WORK/payload" && mkdir -p "$WORK/payload/ewe.companion"
  (cd "$PLUGIN_DIR" && tar --exclude=.git -cf - .) | tar -xf - -C "$WORK/payload/ewe.companion"
}

conf() {
  local scheme=ewe-dark opacity=100
  [ "${O_LIGHT:-0}" = 1 ] && scheme=ewe-light
  [ "${O_GLASS:-0}" = 1 ] && opacity=80
  cat >"$WORK/ewe.conf" <<EOF
[desktop.theme]
scheme = "$scheme"
bar_opacity = $opacity

[plugins.settings]
"ewe.companion" = {screen = "all"}
EOF
}

shell_up() {
  payload
  local c="$WORK/ewe.conf"
  [ -n "${O_CONF:-}" ] && c="$O_CONF"
  drv down >/dev/null 2>&1
  # HS_NO_SYSTEMCTL: the nested shell must never start or restart the host's
  # units (the plugin's ensureDaemon runs `systemctl --user start`)
  HS_HEADLESS=1 HS_CONF="$c" HS_PAYLOAD="$WORK/payload" HS_PLUGINS=1 HS_NO_SYSTEMCTL=1 \
    EWE_COMPANION_SOCK="$SOCK" timeout 180 "$DRV" up 2>&1 | tail -1
  if [ -n "${O_SCALE:-}" ]; then
    drv hc eval "hl.monitor({ output = \"SHOT\", mode = \"${O_MODE:-2880x1800}@60\", position = \"1920x0\", scale = $O_SCALE })" >/dev/null
  fi
  sleep 2
}

measure() {
  python3 - "$@" <<'PY'
import sys
from PIL import Image
for path in sys.argv[1:]:
    im = Image.open(path).convert("RGB")
    w, h = im.size
    cx = w // 2
    bg = im.getpixel((cx, h - 50))
    rows = [y for y in range(0, h - 60)
            if sum(abs(a - b) for a, b in zip(im.getpixel((cx, y)), bg)) > 12]
    print(f"{path}: notch bottom {max(rows) if rows else -1} px")
PY
}

cmd="${1:-}"; shift || true
case "$cmd" in
  up)
    [ -x "$DRV" ] || die "no harness at $DRV (set EWE_DIR)"
    [ -f "$PLUGIN_DIR/manifest.json" ] || die "no plugin at $PLUGIN_DIR (set PLUGIN_DIR)"
    O_LIGHT=0 O_GLASS=0 O_SCALE="" O_MODE="" O_CONF=""
    while [ $# -gt 0 ]; do
      case "$1" in
        --light) O_LIGHT=1 ;;
        --glass) O_GLASS=1 ;;
        --scale) O_SCALE="$2"; shift ;;
        --mode)  O_MODE="$2"; shift ;;
        --conf)  O_CONF="$(realpath "$2")"; shift ;;
        *) die "up: unknown option $1" ;;
      esac
      shift
    done
    mkdir -p "$WORK"
    echo "$WORK" >"$LAST"
    printf 'O_LIGHT=%s\nO_GLASS=%s\nO_SCALE=%s\nO_MODE=%s\nO_CONF=%s\n' \
      "$O_LIGHT" "$O_GLASS" "$O_SCALE" "$O_MODE" "$O_CONF" >"$WORK/options"
    build
    daemon_start
    conf
    shell_up
    # the island connects on its own a moment after the shell loads
    for _ in $(seq 20); do
      EWE_COMPANION_SOCK="$SOCK" "$BIN/ewe-companion" status | grep -q '"island":true' && break
      sleep 0.5
    done
    EWE_COMPANION_SOCK="$SOCK" "$BIN/ewe-companion" status
    ;;
  sync)
    [ -f "$WORK/options" ] || die "not up: tests/island.sh up"
    . "$WORK/options"
    conf
    shell_up
    ;;
  restart)
    build
    daemon_start
    echo "daemon restarted; the island reconnects within a few seconds"
    ;;
  ctl) EWE_COMPANION_SOCK="$SOCK" "$BIN/ewe-companion" "$@" ;;
  cards)
    python3 - "$WORK/state/ewe/companion/companion.db" <<'PY2'
import sqlite3, sys
db = sqlite3.connect(f"file:{sys.argv[1]}?mode=ro", uri=True)
for r in db.execute("SELECT id, kind, source FROM cards WHERE answered IS NULL AND reason IS NULL ORDER BY created"):
    print(*r)
PY2
    ;;
  ipc) drv ipc ewe.companion "$@" ;;
  shot)
    [ $# -ge 1 ] || die "shot NAME"
    drv shot "$1.png" >/dev/null && echo "$WORK/$1.png"
    ;;
  type)
    nest
    enter=0
    [ "${1:-}" = "--enter" ] && { enter=1; shift; }
    WAYLAND_DISPLAY="$NEST_WD" wtype -- "$*"
    [ "$enter" = 1 ] && WAYLAND_DISPLAY="$NEST_WD" wtype -k Return
    ;;
  key) nest; WAYLAND_DISPLAY="$NEST_WD" wtype -k "${1:?key NAME}" ;;
  hover)
    nest
    steps="${1:?hover STEPS-FILE}"
    have_pil=0; python3 -c 'import PIL' 2>/dev/null && have_pil=1
    coproc VP { python3 "$HERE/vptr.py" "$XDG_RUNTIME_DIR/$NEST_WD" "$steps"; }
    while IFS=' ' read -r -u "${VP[0]}" what a b; do
      case "$what" in
        WARP) drv hc dispatch "hl.dsp.cursor.move({ x = $a, y = $b })" >/dev/null ;;
        MARK)
          drv shot "$a.png" >/dev/null
          if [ "$have_pil" = 1 ]; then measure "$WORK/$a.png"; else echo "$WORK/$a.png"; fi
          ;;
      esac
      echo ok >&"${VP[1]}"
    done
    wait "$VP_PID" 2>/dev/null
    ;;
  measure) measure "$@" ;;
  log)
    echo "── daemon ($WORK/daemon.log)"; tail -n "${1:-40}" "$WORK/daemon.log" 2>/dev/null
    echo "── shell"; drv log 2>/dev/null | tail -n "${1:-40}"
    ;;
  claude-project)
    dir="${1:?claude-project DIR}"
    mkdir -p "$dir/.claude"
    [ -d "$dir/.git" ] || git -C "$dir" init -q
    # the hooks of the Claude Code plugin, pointed at this checkout's relay;
    # the installed plugin (agents install claude) is switched off here, or
    # both relays would reach the scratch daemon and every card would come twice
    python3 - "$REPO/integrations/claude-code" "$BIN/ewe-companion" \
      >"$dir/.claude/settings.local.json" <<'PY2'
import json, sys
root = sys.argv[1]
hooks = json.load(open(root + "/ewe-companion/hooks/hooks.json"))["hooks"]
for groups in hooks.values():
    for g in groups:
        for h in g["hooks"]:
            h["command"] = h["command"].replace("ewe-companion ", sys.argv[2] + " ", 1)
market = json.load(open(root + "/.claude-plugin/marketplace.json"))["name"]
print(json.dumps({"enabledPlugins": {"ewe-companion@" + market: False}, "hooks": hooks}, indent=1))
PY2
    echo "$dir/.claude/settings.local.json"
    ;;
  claude)
    dir="${1:?claude DIR PROMPT}"; shift
    [ -f "$dir/.claude/settings.local.json" ] || die "no hooks in $dir: tests/island.sh claude-project $dir"
    # a Claude Code session's own variables would make the child think it
    # is nested; the socket is the scratch daemon's
    unset CLAUDECODE $(env | sed -n 's/^\(CLAUDE_CODE_[A-Z_]*\)=.*/\1/p')
    cd "$dir" && EWE_COMPANION_SOCK="$SOCK" claude -p --permission-mode default --model haiku "$*"
    ;;
  down)
    drv down >/dev/null 2>&1
    daemon_stop
    rm -f "$SOCK"
    [ "$(cat "$LAST" 2>/dev/null)" = "$WORK" ] && rm -f "$LAST"
    echo "down ($WORK)"
    ;;
  *) awk 'NR == 1 { next } /^#/ { sub(/^# ?/, ""); print; next } { exit }' "$0"; exit 1 ;;
esac
