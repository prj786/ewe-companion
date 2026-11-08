# ewe-companion

**Shaun**, the desktop companion for ewe: a quiet island in the bar's centre. When
Claude Code wants to run something or asks you a question, a card drops out
of the bar. Click **Allow**, **Deny** or **Always for …**, pick an answer, or
go to that terminal tab. You never leave what you were doing. It also tells
you when a session finishes.

It also has a small chat: ask a question, or say "volume 60", "brightness
up", "game mode on", "open wifi" or "search for …". It opens any installed
app ("open nemo", "open the browser") and opens apps in a folder ("open
kitty in documents", "open documents in nemo", "open zed in ewe-companion").
Plain commands work with no AI.

It works with ewe Safe and ewe Browser, and finds your files:

- **ewe Safe** (ewe's password manager): "copy my GitHub password", "copy my
  GitHub username", "copy my GitHub code", "log me in to GitHub", "lock my
  safe", "unlock my safe", "is my safe synced?", "how many passwords do I
  have?". ewe Safe copies the password itself and clears the clipboard later;
  the companion never sees it. When several logins fit, the island lists
  them (title, username) and a click copies the one you pick. A locked safe
  offers **Unlock ewe Safe**: you type the master password in ewe Safe's own
  window, never in the chat, and the copy happens right after. Logins'
  names are shown on the island only: they never go to an AI and are never
  stored.
- **ewe Browser**: "search for …", "look up …", "open github.com", "play
  …", "find tax in my history", "that site about sheep", "show my tabs",
  "go to the YouTube tab", "what is this page about?". With a version of ewe
  Browser that has its command line (newer than 0.1.0-beta), searches use its own
  engine and come with Undo; with an older one, the page opens as before.
  A page's text goes to your AI marked as untrusted: if the answer asks to
  do something, it is a button for you to press, never done by itself.
- **A site's own search**: "search men's shoes on zara", "find a kettle on
  amazon", "search youtube for sheepdogs", "shop for linen shirts on h&m",
  "go to zara and search for shoes". ewe Browser (0.4) opens that site's
  own results; a site it does not know, or an older browser, gets a plain
  web search of the whole phrase, and the reply says so. "On the web",
  "online" and "on my desktop" keep their old meaning.
- **More of ewe Browser** (0.4 for workspaces, back, forward and
  downloads' buttons): "bookmark this page" (Undo removes it), "find
  recipes in my bookmarks", "go back a page", "go forward", "reload the
  page", "reopen the closed tab", "mute this tab", "pin the github tab",
  "switch to my work workspace in the browser", "next browser workspace", "new workspace
  called travel", "show my browser workspaces", "open github.com in my
  work workspace" (switching needs the word "browser": "the firefox
  workspace" is the desktop's), "what did I download" (rows with Open and Show in
  folder), "open the browser's settings" (or history, music, files, home).
  "Close this tab" is only offered, with Run: ewe Browser asks first for
  that one. Titles, bookmarks, downloads and workspaces show on the island
  only.
- **Files and folders**: "find my resume", "where is the ewe-safe folder",
  "find pdfs about tax", "search my files for invoice". It looks through
  your home folder by name (hidden folders, build folders and caches are
  skipped, at most 1.5 s) and lists what it finds with **Open**, **Show in
  folder**, **Copy path** and, for a folder, **Open terminal here**. "Open
  the ewe-safe folder" finds it and opens it. "find" means files; "search",
  "google" and "look up" mean the web.

- **The desktop**, each with Undo: "wifi off", "bluetooth on", "connect
  my airpods" / "disconnect my headphones" (a paired device by the words
  in its name; several that fit become rows), "airplane mode on" (Wi-Fi and
  Bluetooth off; off puts back what was on), "keep the screen awake" (the
  Insomnia plugin), "install obs studio" (opens Komble's search; installing
  stays your click).
- **Timers, reminders and sums**: "set a timer for 5 minutes", "remind me
  in 10 minutes to call mum", "remind me at 5pm to call the bank", "show my
  timers", "cancel the timer"; a notification when it is due, kept in
  `companion.db` so it survives a restart. "What is 15% of 80", "80 + 15%",
  "12 x 7" are worked out offline.
- **The clock, dates and units** (0.3): "what time is it", "what's the
  date", "what time is it in tokyo", "how many days until christmas",
  "what day is 25 december", "the date in 30 days", "convert 10 km to
  miles", "20 c in f", "5 gb in mb", "sqrt 144". No currencies: those need
  the network.
- **This computer** (0.3): "how much battery do I have", "how much ram is
  free", "what's using my cpu" (the programs as rows, each with **Quit**,
  which only offers the command with Run), "how long has my computer been
  on", "am I online", "what's my ip" (the local addresses; it never asks a
  website for your public one), "how much disk space is left", "what's
  eating my disk" (the package cache, caches, the trash, Downloads, the
  journal), "why is my laptop slow" (all of it at once). Programs are named
  by their process name; the desktop's own (Hyprland, the shell, PipeWire)
  never get a Quit.
- **Windows and workspaces** (0.3): "switch to workspace 3", "next
  workspace", "move this window to workspace 2", "make this window
  fullscreen", "maximise", "float this window", "minimise this window",
  each with Undo. "Close this window" and "quit firefox" show a **Close**
  button first; "firefox is frozen" offers `pkill` with Run.
- **What you tell it to remember** (0.3): "remember that my dentist is on
  friday", "what do you remember", "forget about my dentist" (Undo puts it
  back), "forget everything" (asks first). Kept on this computer, 100 at
  most, and given to whoever answers your questions so answers can use
  them. Anything that looks like a password, a PIN, a code, a key or a card
  number is refused, and its words stay out of the conversation too.
- **Two things at once** (0.3): "dnd on and set a timer for 25 minutes",
  "mute, then lock the screen": up to three, with one Undo for all of
  them. And small typos are forgiven: "opne firefox", "volme 50".
- **Asked first** (shown with Run, never run by the matcher): "dark mode" /
  "switch to light mode" (`ewe-theme scheme apply …`), "suspend",
  "restart", "shut down", "log out" (ewe's own `power.sh`, what the Power
  button runs), "empty the trash" (`gio trash --empty`, after saying how
  many items).

All of these work with no AI, and an AI you picked can ask for them too:
it then hears back only "done" or a count.

It runs shell commands for you, after a click. Ask in your own words ("make
everything light", "how much space do my downloads take?"): when a command
does it, the chat shows that exact command with **Run** and **In terminal**,
and nothing runs until you press one. Type `$ <command>` to have it run one
of your own the same way. Commands that need your password (sudo) open in a
terminal, risky ones say why first, and a few are refused outright: wiping
home or the system, erasing a disk, a partial `pacman -Sy`, writing ewe.conf
or the generated files by hand.

It knows ewe. A knowledge pack built into the daemon covers the apps and
their pages, the shortcuts, how to do everyday things, what to do when
something is wrong, ewe Safe, ewe Browser, and every ewe command-line tool
(`ewe-conf`, `ewe-theme`, `ewe-plugin`, `ewe-bt`, `ewe-safe`,
`ewe-browser`, `komble`, `qs ipc` …). "How do I change my wallpaper?" is answered with no AI
at all, with a button that opens the place. Other questions go to whoever
you pick in the plugin's options: your own Claude Code, Codex or OpenCode
login, an API key (Anthropic, OpenAI, Gemini, OpenRouter), or a local model
(Ollama, LM Studio). They are told what the pack knows and what this
computer is like now.

Answers are quick and appear as they are written (0.3). Your Claude Code
runs lean, without your own plugins, skills, MCP servers or settings
(those once made "2+2" take 7.5 s), and it is started when you type the
first letter of a question, so the answer starts in about a second. When
the answer depends on this computer, the model looks it up first (what
uses the CPU, what fills the disk) and answers from what it found; one
reply can do up to three things, with one Undo.

**Status: 0.2.0-beta (wave 2, 2026-10-18: a site's own search, more of the
browser, the desktop's switches, timers, sums, every ewe command-line tool),
P1 and P2 working, plus P2.5 (the notch, the home
panel, the chat groundwork), P2.6 (the notch fixes, the ewe knowledge pack),
P2.7 (apps and folders), P2.8 (it runs what it suggests, on a click),
2026-10-14, and P2.9 (ewe Safe, ewe Browser, the finder), 2026-10-15. First
release 2026-10-17: the `ewe-companion` package in the [ewe] repository,
and the island (plugin `ewe.companion` 0.4.0) in ewe 0.29.0-beta's
plugins. Wave 2 (2026-10-18, P2.10: a site's search, more of the browser,
the desktop's switches, timers, sums, the CLI knowledge) shipped as
0.2.0-beta the same day. 0.2.2-beta (2026-11-01) names the other apps
the way the desktop now does: ewe Settings, ewe Sync, ewe Browser, ewe
Safe and ewe Cast. 0.2.3-beta (2026-11-01) knows ewe 0.39's UI revamp:
Quick settings without the rail, the System monitor in the bar, Komble's
plugin sheet, ewe Settings' four groups, ewe Sync's words, ewe Browser's
save prompt and ewe Safe as the SSH gate. 0.2.4-beta (2026-11-08) hands
"game mode" to ewe's own (`ewe-game`, ewe 0.42): one mode for the
tile, ewe Settings and Shaun, with Undo. 0.3 (2026-11-08, not yet
released) is faster and wiser: Claude Code lean and started ahead of the
question, answers that stream in, the time and this computer's state for
the model, look-ups, up to three things at once; and new things to ask:
the clock, dates and units, this computer's battery, memory, CPU, disk and
network, windows and workspaces, and what you tell it to remember.**

**Install:** `sudo pacman -S ewe-companion`, then Komble → Plugins → From
ewe → Shaun (or `ewe-plugin install ewe.companion`). For the agent cards:
`ewe-companion agents install claude`.

- **Why and what:** [`ewe/docs/RFC-007-companion.md`](../ewe/docs/RFC-007-companion.md)
- **How, phase by phase:** [`docs/PLAN.md`](docs/PLAN.md)
- **The socket contract:** [`docs/PROTOCOL.md`](docs/PROTOCOL.md)
- **Working on it** (people and agents): [`AGENTS.md`](AGENTS.md), then
  [`docs/EXTENDING.md`](docs/EXTENDING.md) and [`docs/TESTING.md`](docs/TESTING.md)

## What lives here

| part | what |
|---|---|
| `crates/core` | the protocol, Claude Code's hook dialect (card wording, the exact answer JSON), payload trimming, paths. Pure and unit-tested |
| `crates/daemon` → `ewe-companiond` | the socket, cards and their deadlines, agent sessions, jump-to-terminal, the SQLite store (`~/.local/state/ewe/companion/companion.db`) |
| `crates/cli` → `ewe-companion` | the CLI (JSON out, exit 0) and `hook claude`, the relay Claude Code's hooks run. Std-only, so it starts in milliseconds |
| `knowledge` | what the companion knows about ewe: `primer.md` (with every question) and the topics (`apps`, `howto`, `keys`, `fix`, `safe`, `browser`, `cli`). Built into the binary; see *The knowledge pack* |
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
ewe-companion ask "copy my GitHub password"    # ewe Safe copies it; the reply never holds it
ewe-companion ask "find pdfs about tax"        # the finder: rows on the island
ewe-companion find resume                      # the finder's paths, as JSON
ewe-companion ask "search men's shoes on zara"  # the site's own search, in ewe Browser
ewe-companion ask "remind me in 10 minutes to call mum"   # a notification when due
ewe-companion ask "what is 15% of 80"          # worked out offline
ewe-companion ask "restart"                    # offered with Run, never run by itself
ewe-companion debug undo <token>               # (debug daemon) the Undo click
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
(ewe 0.29, ewe Settings 0.18, Komble 0.20, ewe Sync 0.14, ewe Browser 0.3 and
the wave-2 contracts for 0.4, ewe Safe 0.2); `cli.md` covers every ewe
command-line tool. A topic is a
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
- **ewe Settings ignores page links today** (0.18 to 0.22 open on
  Appearance), so answers name the page to pick. Komble and Quick settings
  pages do open.

What a provider is told: the primer (the same every time, so it can be
cached), and with each question the topics that fit, what you asked it to
remember, and *this computer now*: the date and time, the ewe and kernel
versions, which ewe apps and plugins are here, the plugins' and your own
shortcuts, the scheme, accent and Glass, uptime, memory in use, CPU load,
online or not, the disks, waiting updates, battery, volume, brightness, Do
not disturb, game mode and your agents. Never ewe.conf as a whole (its
account and mail sections name your servers), never a key or a password,
never a network's name or an address. With no provider, nothing leaves the
machine.

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
- **It never sees a password.** It only asks ewe Safe to copy one (`ewe-safe
  copy`), and never fills a login itself: the fill stays your click in the
  browser, or `Super+P`.
- **The finder is by name only.** It does not read what is inside your
  files, and it stops after 1.5 s on a very large home folder (the reply
  says so).
- **Browser history, tabs, the page and "play" from your library need ewe
  Browser's command line,** which comes after 0.1.0-beta. Until then those
  say so, and searches and pages open the old way. A site's own search,
  workspaces, back and forward and the downloads' buttons need ewe Browser
  0.4: an older one gets a plain web search, or "update it in Komble".
- **Airplane mode is the companion's, not ewe's.** ewe has no airplane
  switch; the companion turns Wi-Fi and Bluetooth off and remembers what
  was on (in memory: after a daemon restart, "airplane mode off" turns both
  on). Bluetooth comes back on at the next boot anyway.
- **Keeping the screen awake needs the Insomnia plugin,** and a shell
  restart turns it off (the plugin keeps no state).
- **A quick answer from Claude Code costs memory for a while.** The
  `claude` started ahead of your question holds about 190 MB until it
  answers, or for 3 minutes if you never ask.
- **Window commands speak Hyprland's Lua** (0.56 and later, as ewe ships).
  "This window" is the last window you used before opening the notch.
- **Timers are not alarms.** A due timer is a normal notification: Do not
  disturb hides its banner (it still lands in Quick settings →
  Notifications), and nothing rings.
- **Jump to the exact tab needs kitty remote control.** ewe's kitty config
  turns it on (socket-only, in your private runtime dir). In other
  terminals, *Answer in terminal* focuses the window only.

## Develop

Start with [`AGENTS.md`](AGENTS.md): the rules, the map of the code, how the
pieces talk, and where each kind of change goes.
[`docs/EXTENDING.md`](docs/EXTENDING.md) has a recipe per change (a command,
a knowledge topic, a provider, a card…), [`docs/TESTING.md`](docs/TESTING.md)
every way to check one.

```sh
cargo test && cargo clippy --all-targets -- -D warnings
tests/chat.sh && tests/relay-failsafe.sh   # the chat path; the relay's fail-safes
tests/canary.sh                            # names stay local: ewe Safe's names reach no model, log or store
tests/island.sh up --scale 1.8             # the island in ewe's nested harness
tests/island.sh ctl debug card permission  # a card to look at
tests/island.sh shot card                  # → /tmp/ewe-companion-island/card.png
tests/island.sh down
```

`tests/island.sh` runs a debug daemon on its own socket and the plugin in a
sandboxed nested shell, with a virtual mouse for hovers and clicks and a
scratch Claude Code project for real hook runs. The debug daemon still runs
commands on this machine (volume, brightness, a Run click), and hooks are
never tested against your real `~/.claude`. ewe Safe and ewe Browser are the
fakes in `tests/fakes/` there and in every test: never your real safe or
browser.

MIT licence.
