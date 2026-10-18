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

It works with Safe and ewe browser, and finds your files:

- **Safe** (ewe's password manager): "copy my GitHub password", "copy my
  GitHub username", "copy my GitHub code", "log me in to GitHub", "lock my
  safe", "unlock my safe", "is my safe synced?", "how many passwords do I
  have?". Safe copies the password itself and clears the clipboard later;
  the companion never sees it. When several logins fit, the island lists
  them (title, username) and a click copies the one you pick. A locked safe
  offers **Unlock Safe**: you type the master password in Safe's own
  window, never in the chat, and the copy happens right after. Logins'
  names are shown on the island only: they never go to an AI and are never
  stored.
- **ewe browser**: "search for …", "look up …", "open github.com", "play
  …", "find tax in my history", "that site about sheep", "show my tabs",
  "go to the YouTube tab", "what is this page about?". With an ewe browser
  that has its command line (newer than 0.1.0-beta), searches use its own
  engine and come with Undo; with an older one, the page opens as before.
  A page's text goes to your AI marked as untrusted: if the answer asks to
  do something, it is a button for you to press, never done by itself.
- **A site's own search**: "search men's shoes on zara", "find a kettle on
  amazon", "search youtube for sheepdogs", "shop for linen shirts on h&m",
  "go to zara and search for shoes". ewe browser (0.4) opens that site's
  own results; a site it does not know, or an older browser, gets a plain
  web search of the whole phrase, and the reply says so. "On the web",
  "online" and "on my desktop" keep their old meaning.
- **More of ewe browser** (0.4 for workspaces, back, forward and
  downloads' buttons): "bookmark this page" (Undo removes it), "find
  recipes in my bookmarks", "go back a page", "go forward", "reload the
  page", "reopen the closed tab", "mute this tab", "pin the github tab",
  "switch to my work workspace in the browser", "next browser workspace", "new workspace
  called travel", "show my browser workspaces", "open github.com in my
  work workspace" (switching needs the word "browser": "the firefox
  workspace" is the desktop's), "what did I download" (rows with Open and Show in
  folder), "open the browser's settings" (or history, music, files, home).
  "Close this tab" is only offered, with Run: ewe browser asks first for
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
something is wrong, Safe, ewe browser, and every ewe command-line tool
(`ewe-conf`, `ewe-theme`, `ewe-plugin`, `ewe-bt`, `ewe-safe`,
`ewe-browser`, `komble`, `qs ipc` …). "How do I change my wallpaper?" is answered with no AI
at all, with a button that opens the place. Other questions go to whoever
you pick in the plugin's options: your own Claude Code, Codex or OpenCode
login, an API key (Anthropic, OpenAI, Gemini, OpenRouter), or a local model
(Ollama, LM Studio). They are told what the pack knows and what this
computer is like now.

**Status: 0.2.0-beta (wave 2, 2026-10-18: a site's own search, more of the
browser, the desktop's switches, timers, sums, every ewe command-line tool),
P1 and P2 working, plus P2.5 (the notch, the home
panel, the chat groundwork), P2.6 (the notch fixes, the ewe knowledge pack),
P2.7 (apps and folders), P2.8 (it runs what it suggests, on a click),
2026-10-14, and P2.9 (Safe, ewe browser, the finder), 2026-10-15. First
release 2026-10-17: the `ewe-companion` package in the [ewe] repository,
and the island (plugin `ewe.companion` 0.4.0) in ewe 0.29.0-beta's
plugins. Wave 2 (2026-10-18, P2.10: a site's search, more of the browser,
the desktop's switches, timers, sums, the CLI knowledge) is built, for the
next release.**

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
ewe-companion ask "copy my GitHub password"    # Safe copies it; the reply never holds it
ewe-companion ask "find pdfs about tax"        # the finder: rows on the island
ewe-companion find resume                      # the finder's paths, as JSON
ewe-companion ask "search men's shoes on zara"  # the site's own search, in ewe browser
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
(ewe 0.29, Settings 0.18, Komble 0.20, ewe-sync 0.14, ewe browser 0.3 and
the wave-2 contracts for 0.4, Safe 0.2); `cli.md` covers every ewe
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
- **It never sees a password.** It only asks Safe to copy one (`ewe-safe
  copy`), and never fills a login itself: the fill stays your click in the
  browser, or `Super+P`.
- **The finder is by name only.** It does not read what is inside your
  files, and it stops after 1.5 s on a very large home folder (the reply
  says so).
- **Browser history, tabs, the page and "play" from your library need ewe
  browser's command line,** which comes after 0.1.0-beta. Until then those
  say so, and searches and pages open the old way. A site's own search,
  workspaces, back and forward and the downloads' buttons need ewe browser
  0.4: an older one gets a plain web search, or "update it in Komble".
- **Airplane mode is the companion's, not ewe's.** ewe has no airplane
  switch; the companion turns Wi-Fi and Bluetooth off and remembers what
  was on (in memory: after a daemon restart, "airplane mode off" turns both
  on). Bluetooth comes back on at the next boot anyway.
- **Keeping the screen awake needs the Insomnia plugin,** and a shell
  restart turns it off (the plugin keeps no state).
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
tests/canary.sh                            # names stay local: Safe's names reach no model, log or store
tests/island.sh up --scale 1.8             # the island in ewe's nested harness
tests/island.sh ctl debug card permission  # a card to look at
tests/island.sh shot card                  # → /tmp/ewe-companion-island/card.png
tests/island.sh down
```

`tests/island.sh` runs a debug daemon on its own socket and the plugin in a
sandboxed nested shell, with a virtual mouse for hovers and clicks and a
scratch Claude Code project for real hook runs. The debug daemon still runs
commands on this machine (volume, brightness, a Run click), and hooks are
never tested against your real `~/.claude`. Safe and ewe browser are the
fakes in `tests/fakes/` there and in every test: never your real safe or
browser.

MIT licence.
