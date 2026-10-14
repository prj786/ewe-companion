# ewe-companion

The desktop companion for ewe: a quiet island in the bar's centre. When
Claude Code wants to run something or asks you a question, a card drops out
of the bar. Click **Allow**, **Deny** or **Always for …**, pick an answer, or
go to that terminal tab. You never leave what you were doing. It also tells
you when a session finishes.

It also has a small chat: ask a question, or say "volume 60", "brightness
up", "game mode on", "open wifi" or "search for …". It opens any installed
app ("open nemo", "open the browser") and opens apps in a folder ("open
kitty in documents", "open documents in nemo", "open zed in ewe-companion").
Plain commands work with no AI.

It runs shell commands for you, after a click. Ask in your own words ("make
everything light", "how much space do my downloads take?"): when a command
does it, the chat shows that exact command with **Run** and **In terminal**,
and nothing runs until you press one. Type `$ <command>` to have it run one
of your own the same way. Commands that need your password (sudo) open in a
terminal, risky ones say why first, and a few are refused outright: wiping
home or the system, erasing a disk, a partial `pacman -Sy`, writing ewe.conf
or the generated files by hand.

It knows ewe. A knowledge pack built into the daemon covers the apps and
their pages, the shortcuts, how to do everyday things and what to do when
something is wrong. "How do I change my wallpaper?" is answered with no AI
at all, with a button that opens the place. Other questions go to whoever
you pick in the plugin's options: your own Claude Code, Codex or OpenCode
login, an API key (Anthropic, OpenAI, Gemini, OpenRouter), or a local model
(Ollama, LM Studio). They are told what the pack knows and what this
computer is like now.

**Status: 0.1.0-beta, P1 and P2 working, plus P2.5 (the notch, the home
panel, the chat groundwork) and P2.6 (the notch fixes, the ewe knowledge
pack), 2026-10-14. Not released.**

- **Why and what:** [`ewe/docs/RFC-007-companion.md`](../ewe/docs/RFC-007-companion.md)
- **How, phase by phase:** [`docs/PLAN.md`](docs/PLAN.md)
- **The socket contract:** [`docs/PROTOCOL.md`](docs/PROTOCOL.md)

## What lives here

| part | what |
|---|---|
| `crates/core` | the protocol, Claude Code's hook dialect (card wording, the exact answer JSON), payload trimming, paths. Pure and unit-tested |
| `crates/daemon` → `ewe-companiond` | the socket, cards and their deadlines, agent sessions, jump-to-terminal, the SQLite store (`~/.local/state/ewe/companion/companion.db`) |
| `crates/cli` → `ewe-companion` | the CLI (JSON out, exit 0) and `hook claude`, the relay Claude Code's hooks run. Std-only, so it starts in milliseconds |
| `knowledge` | what the companion knows about ewe: `primer.md` (with every question) and the topics (`apps`, `howto`, `keys`, `fix`). Built into the binary; see *The knowledge pack* |
| `integrations/claude-code` | a Claude Code plugin with the hooks, plus a local marketplace for it |
| `packaging` | the user unit (no `[Install]`: the plugin starts it on demand) and the PKGBUILD |

The island is the shell plugin `ewe.companion`, in
[`prj786/ewe-plugin-companion`](../ewe-plugin-companion).

## Use it

```sh
ewe-companion agents install claude   # Claude Code plugin from the local marketplace
ewe-companion status                  # {"ok":true,"running":true,"island":true,…}
ewe-companion say "Backup done" --level success
ewe-companion ask "volume 60"            # a command, no AI needed
ewe-companion ask "open kitty in documents"    # an app in a folder
ewe-companion ask '$ df -h'                    # a command, offered with a token
ewe-companion debug confirm <token>            # (debug daemon) the Run click
ewe-companion ask "how do I pair headphones?"  # answered from the knowledge pack
ewe-companion ask "what is a swap file?" # a question, to your provider
ewe-companion key set anthropic < keyfile  # the key goes to the keyring
ewe-companion agents usage claude on     # plan usage in the notch
sudo pacman -Syu && ewe-companion say "Updates done"
ewe-companion sessions                # the agent sessions the island lists
ewe-companion agents uninstall claude
```

Restart running Claude Code sessions after `agents install` so they load the
hooks. Nothing in `~/.claude/settings.json` changes: the hooks live in the
plugin.

## The knowledge pack

`knowledge/*.md` is user-facing text about ewe, researched from the repos
(Settings 0.18, Komble 0.20, ewe-sync 0.14, the shell 0.25.2). A topic is a
`## Title`, a `keys:` line (the words people use), an optional `open:` line
(`settings wallpaper`, `komble updates`, `quicksettings wifi`, `overview`,
`sync`) and the text. `<!-- … -->` comments are for whoever keeps the pack
and never reach a model.

- **It ships inside the binary** (`include_str!`), so every install has it.
- **To try an edited pack** without a rebuild, start the daemon with
  `EWE_COMPANION_KNOWLEDGE=<dir of .md files>`.
- **It must follow ewe.** When a page, a label or a shortcut changes in ewe,
  change the pack in the same change. `cargo test` checks that every topic
  parses and that every `open:` line is a valid command.
- **Settings ignores page links today** (0.18 opens on Appearance), so
  answers name the page to pick. Komble and Quick settings pages do open.

What a provider is told with each question: the primer, the topics that fit,
and *this computer now*: the ewe and kernel versions, which ewe apps and
plugins are here, the plugins' and your own shortcuts, the scheme, accent
and Glass, the disks, waiting updates, battery, volume, brightness, Do not
disturb, game mode and your agents. Never ewe.conf as a whole (its account
and mail sections name your servers), never a key or a password. With no
provider, nothing leaves the machine.

## Honest limits

- **It never approves anything without your click.**
  - With the daemon or the island down, the screen locked, or no answer
    within 108 s, the relay prints nothing and Claude asks in its terminal
    as usual.
  - "Always" only ever hands back the rule Claude itself suggested. The
    companion keeps no permission rules of its own.
  - The one exception is `debug answer`, which exists for tests. It works
    only on a daemon started with `EWE_COMPANION_DEBUG=1`, which neither the
    unit nor the plugin sets.
- **Claude Code only, for now.** OpenCode, Codex and Gemini come in P6.
- **Some subscriptions cannot be used by an app like this.** Claude.ai and
  Google consumer subscriptions are off-limits by their own terms. The chat
  (P4) uses your API key or a local model, and P6 adds your own logged-in
  agents. It never reads or stores a subscription login.
- **It runs as you, unsandboxed,** like every ewe plugin. Any process
  running as you can talk to its socket, as it can already drive your
  session.
- **Jump to the exact tab needs kitty remote control.** ewe's kitty config
  turns it on (socket-only, in your private runtime dir). In other
  terminals, *Answer in terminal* focuses the window only.

## Develop

```sh
cargo test && cargo clippy --all-targets -- -D warnings
cargo build && tests/relay-failsafe.sh     # 26 fail-safe checks over a real socket
```

To try the island without installing anything, use ewe's nested harness (the
`ewe-smoke` skill). Copy the plugin into a scratch payload, run a debug
daemon on a short socket path, and drive it:

```sh
export EWE_COMPANION_SOCK=$XDG_RUNTIME_DIR/ewe-companion-smoke.sock
EWE_COMPANION_DEBUG=1 XDG_STATE_HOME=/tmp/cs target/debug/ewe-companiond &
# HS_PAYLOAD=<dir with ewe.companion/ + bundle.json> HS_PLUGINS=1 driver.sh up
target/debug/ewe-companion debug card permission     # or question / note
target/debug/ewe-companion say "Download done · archlinux.iso · 1.2 GB" --level success
```

The headless harness has no mouse, so hovers and clicks never happen there:
`tests/vptr.py` adds a virtual one (see its header for the step script).

Record real hook payloads with `EWE_COMPANION_HOOK_LOG=<file>` in a scratch
project (`tests/payloads/claude/README.md`). Never test hooks against your
real `~/.claude`.

MIT licence.
