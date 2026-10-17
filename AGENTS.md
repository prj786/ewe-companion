# ewe-companion — map for agents

This file is the entry point for any agent working on the companion. It
says what exists, how the pieces talk, where each kind of change goes, how
to check it, and which traps have already been hit. Read it whole before
the first edit. Two guides go deeper:

- [`docs/EXTENDING.md`](docs/EXTENDING.md): one recipe per kind of change
  (a command, a topic, a provider, a protocol message, a card, a section…),
  with the files to touch, the tests to add and the docs to update.
- [`docs/TESTING.md`](docs/TESTING.md): every test layer, the island
  harness (`tests/island.sh`), the virtual mouse, real Claude Code runs, and
  deploying to the owner's desktop.

The island (the QML half) lives in the sibling repo
[`ewe-plugin-companion`](../ewe-plugin-companion), with its own
[`AGENTS.md`](../ewe-plugin-companion/AGENTS.md). Most features touch both.

## What this is

**Shaun**, the desktop companion of **ewe**, an Arch-only Hyprland +
Quickshell desktop (`../ewe`). Shaun is the user-facing name (owner,
2026-10-17): the notch's word, the plugin's name, the persona, the
knowledge pack. Every id stays `companion` (the package, the binaries, the
plugin id `ewe.companion`, the IPC target, the socket, the keyring
service): ids are public API (rule 4).

A notch hangs from the top bar's centre (the plugin `ewe.companion`). It
shows what your AI agents are doing, drops a card when
Claude Code wants permission or asks a question, glances at what needs you,
runs quick actions, and has a small chat that runs commands in plain words,
opens apps and folders, finds files, works with Safe (the password manager)
and ewe browser, answers questions about ewe from a built-in knowledge pack,
and proposes shell commands that run only on a click.

This repo is the Rust half: the daemon `ewe-companiond`, the CLI and hook
relay `ewe-companion`, the knowledge pack, the Claude Code plugin, packaging
and tests. Version **0.1.0-beta**, first released 2026-10-17 (the package in
[ewe]; the plugin in ewe 0.29.0-beta). P1 is done, P2 works end
to end with a few tasks left, P2.5 to P2.8 are done (2026-10-14), and P2.9
(Safe, ewe browser, the finder: ewe-safe's `docs/TRIO.md` §C1) is built
(2026-10-15); P3 (notifications) is next. The phase list with open tasks
is `docs/PLAN.md` §20.

| document | holds |
|---|---|
| `../ewe/docs/RFC-007-companion.md` | why, the rules C1–C8, the owner's nine decisions |
| `docs/PLAN.md` | how: processes, schemas, every phase's task list and gate, §25 lessons |
| `docs/PROTOCOL.md` | the socket contract (public API between our repos) |
| `README.md` | the user's view: what it does, honest limits |
| `../project-map/10-Roadmap/Roadmap — ewe-companion.md` | the vault's status note |

## Read first (the house)

ewe has house rules that override anything here. Before touching code:

1. `../AGENTS.md` (the workspace) and `../project-map/00 Onboarding.md`;
2. `../project-map/09-Rules/Rules of the House.md`;
3. `../project-map/08-Decisions/RFC-007 — ewe Companion.md`.

The repos win over the vault; the vault wins over memory. When this file
and a house rule disagree, the house rule wins and this file is wrong: fix
it.

## The rules that bite here

House rules (the numbers are the vault's):

- **1.** `ewe-conf` is the only writer of `ewe.conf`. The companion reads it
  and never writes it; `shell.rs` refuses a command that writes it.
- **2.** Secrets never enter `ewe.conf` (it syncs). API keys go to the
  keyring (`daemon/keyring.rs`, service `ewe-companion`), typed into the
  island, never a plugin setting.
- **3.** Layer-shell surfaces stay in the shell: the notch is QML in the
  plugin, never a window of ours.
- **4.** IPC verbs and protocol message types are public API: add fields,
  never rename or remove.
- **5.** Every CLI verb prints one JSON object and exits 0. The hook relay
  is the one exception: it prints Claude Code's JSON or nothing.
- **8.** No raw colour, size or duration in QML: design tokens only.
- **9.** Never edit `generated/` files or the deployed `~/.config`.
- **10.** No partial upgrades: `pacman -Sy` without `u` is refused.

RFC-007's rules, in code:

- **C1 — it never decides for you.** No allow without a human's click.
  "Always" only echoes back one of Claude's own `permission_suggestions`.
  A shell command runs only from its Run click (`run_shell` is refused by
  `commands::run`, so no path skips the click).
- **C2 — it fails back to the old way.** Daemon down, island down, screen
  locked, no ack within 800 ms, no answer within 108 s: the relay prints
  nothing, exits 0, and Claude asks in its own terminal.
- **C3 — secrets never reach a model.** No key, no password, no `ewe.conf`
  as a whole in a prompt. Logs carry types, ids and timings, never prompt
  text, commands or answers.
- **C4 — untrusted text cannot act.** Notifications, pages, file names and
  agent output are data. A turn that carries them cannot auto-run a
  command. In code since P2.9: a page goes to a model inside
  `<untrusted source="web page">`, and an ACTION in that reply is a "Do it"
  button (`chat::about_page`). What ewe browser's `untrusted` tools return
  reaches a model only as a count.
- **Values never cross, names stay local** (TRIO.md, RFC-008 S8). The
  companion asks Safe to copy; it never calls `show` or `reveal`, and
  `shell::assess` refuses a proposed command that would print a password
  manager's values or names (`ewe-safe read`, `op read`, `pass show` …),
  since a run's output goes back into the conversation. Safe's
  titles and usernames, found files and the browser's pages show on the
  island; the conversation keeps the outcome's `private` line ("done", a
  count), and nothing of them is stored or logged. `tests/canary.sh`
  proves it.
- **C5 — it works with no AI.** Commands, the knowledge pack's answers,
  cards and the glance work with no provider. The offline matcher goes
  first.
- **C6 — attention is a budget.** Levels: log, glance, peek, card. No modal,
  no focus steal (a card that drops by itself takes no keyboard).
- **C7 — it holds no subscription credential.** Your own Claude Code, Codex
  or OpenCode login stays in that app; the companion only runs it.
- **C8 — it uses ewe's pipelines.** Settings through `ewe-conf`, sync
  through ewe-sync, packages through Komble. See *Open questions*: P2.8's
  knowledge `run:` lines stretch this.

The owner's decision 6 sets the tiers: **T1** desktop actions (volume, Do not
disturb, opening things) run at once with an Undo; **T2/T3** (settings,
files, the system) always ask first. Every command in the catalog is T0 (a
read that shows rows: the finder, the browser's history and tabs, Safe's
state) or T1, except `run_shell`, which never runs from the catalog: it only
ever becomes a proposal with Run. ewe browser's own tiers are respected: a
tool it marks tier 2 becomes a proposal too.

## Commands

```sh
cargo build                                   # debug: target/debug/ewe-companion{,d}
cargo test                                    # core 69, daemon 25 unit tests
cargo clippy --all-targets -- -D warnings     # CI runs it with --locked
cargo fmt --all --check
tests/chat.sh                                 # 53 checks: the chat path vs a mock LLM, Safe,
                                              #   ewe browser and the finder vs tests/fakes/
tests/relay-failsafe.sh                       # 26 checks: the hook relay's fail-safes
tests/canary.sh                               # 8 checks: Safe's names reach no model, log or store
BIN=target/release tests/relay-failsafe.sh    # the same against a release build
tests/island.sh up --scale 1.8                # the island in ewe's nested harness
```

The scripts run the binaries in `target/debug` as they are: `cargo test`
does not rebuild them, so **build first**, or they test stale code. Run the
whole loop (build, test, clippy, fmt, the three scripts) after any Rust
change; CI (`.github/workflows/ci.yml`) runs the same on every push to
`main` and every PR. After a QML or knowledge change, look at the island in the
harness and read the screenshot before you report.

## Layout

```
Cargo.toml              workspace: crates/core, crates/daemon, crates/cli; pinned deps
VERSION                 0.1.0-beta (every binary reports it)
crates/core/            companion-core: pure code, no I/O, every rule unit-tested
  src/lib.rs            the module list
  src/proto.rs          the socket protocol: Inbound messages, Card, Action, Peek, Env; decode(), line()
  src/claude.rs         Claude Code's hook dialect: which events wait, card_for(), render() of the answer
  src/trim.rs           shrink a hook payload (2 KiB strings, diffs up to 256 KiB)
  src/paths.rs          the socket path, the state dir, the private-dir check
  src/id.rs             short time-ordered ids
  src/commands.rs       the command CATALOG, validate(), the Settings/Komble/Quick settings pages
  src/intent.rs         the offline matcher: plain words → a Call (regex rules, order matters); about_page()
  src/find.rs           the finder's rules: what "find pdfs about tax" asks for, rank()
  src/persona.rs        the system prompt (Brief) and the one-line ACTION protocol
  src/knowledge.rs      the knowledge pack: parse, search, CONFIDENT, Topic::answer
  src/shell.rs          assess(cmd): Fine, Risky(why) or Refused(why), plus "needs a terminal"
crates/daemon/          ewe-companiond (tokio)
  src/main.rs           start-up, logging (EWE_COMPANION_LOG), the accept loop
  src/server.rs         per-connection: hello (island), agent (hook), cli; the CLI verbs
  src/hub.rs            the state (one Mutex) and the store (its own); push_state(), shell_do(), tokens
  src/store.rs          SQLite (WAL) at ~/.local/state/ewe/companion/companion.db; migrations
  src/store/migrations/ numbered .sql files, compiled in
  src/chat.rs           one chat line in, one Reply out: matcher → pack → provider; proposals; confirm()
  src/commands.rs       runs a catalog Call (argv only, never a shell); Outcome {ok, line, undo, miss,
                        rows, action, private, propose}
  src/browser.rs        ewe browser's command line: detect (commands), read (tier 0), act via the
                        island (tier 1), undo steps; search, open, play, history, tabs, tab, page
  src/safe.rs           Safe's command line: copy, login, lock, unlock (and what waited for it), status
  src/finder.rs         files and folders under home by name (the ignore crate, 1.5 s), rows
  src/places.rs         folders by the words people use; default apps via xdg-mime
  src/providers.rs      PROVIDERS: CLI lane (claude-code, codex, opencode) and API lane
  src/keyring.rs        secret-tool, key on stdin
  src/context.rs        "this computer now" for a model: installed part (kept 60 s) + live part
  src/glance.rs         disks and waiting updates for the home panel
  src/runner.rs         runs a proposed command after the Run click (bash -c, 120 s, last 12 lines)
  src/term.rs           jump to an agent's terminal: Hyprland window, kitty tab, tmux pane
crates/cli/             ewe-companion (std-only: it starts in milliseconds)
  src/main.rs           dispatch: verbs, hook, statusline
  src/verbs.rs          status, say, history, sessions, jump, ask, key, agents, debug …
  src/hook.rs           the hook relay: Claude Code stdin → daemon → Claude's JSON, or nothing
  src/statusline.rs     forwards rate_limits for the plan-usage bars, then runs your old status line
  src/client.rs         connect within a budget, check the peer uid, one line out, one back
knowledge/              the pack (Markdown, compiled in): primer.md + apps, howto, keys, fix, safe, browser
integrations/claude-code/  a Claude Code plugin (hooks.json) + a local marketplace
packaging/              PKGBUILD, ewe-companion.service (user unit, no [Install])
tests/                  chat.sh, relay-failsafe.sh, canary.sh, island.sh, vptr.py, mock-llm.py,
                        fake-island.py (--do answers `do`), fakes/ewe-safe, fakes/ewe-browser,
                        payloads/claude/*.json (recorded hook payloads)
docs/                   PLAN.md, PROTOCOL.md, EXTENDING.md, TESTING.md
```

`docs/PLAN.md` §4 is the layout as planned on 2026-10-13 (watchers/,
providers/, an app/ window…). Those parts do not exist yet; this list is
what does.

## How the pieces talk

```
 Claude Code ──hook──▶ ewe-companion hook claude ─┐  (one connection per hook call)
 a terminal ─────────▶ ewe-companion <verb> ──────┤  (one request, one reply)
                                                  ▼
                      $XDG_RUNTIME_DIR/ewe-companion.sock  (0600, SO_PEERCRED same uid)
                                                  ▼
                                  ewe-companiond (user unit)
                                  hub · store · chat · providers · runner
                                                  ▲
                                   hello, then stays connected
                                                  │
                 Quickshell ── plugin ewe.companion: Service.qml ⇄ CompanionState ⇄ Drop.qml
```

One JSON object per line, `{"v":1,"type":…}`. The first line says who is
calling: `hello` (the island stays), `agent` (a hook relay), `cli`. The
daemon sends the island a full `state` after every change, so an island that
missed a message is right again at the next one. Contract:
`docs/PROTOCOL.md`.

**Why the split.** The daemon outlives shell restarts (an agent waiting on a
card must not lose it), sockets and model calls do not belong in the QML
engine, and the relay runs on every hook so it must start in milliseconds.
What only the shell can do (Do not disturb, opening apps, screenshots, the
browser) the daemon asks of the island with a `do` message, so an app the
companion opens belongs to the desktop, not to the daemon's unit.

### A permission request (scene 1)

1. Claude Code runs `ewe-companion hook claude PermissionRequest` with the
   event on stdin (`integrations/claude-code/…/hooks.json`).
2. The relay trims the payload, adds the terminal it runs in (kitty window,
   socket, parent pids) and sends `agent` with `wait:true`.
3. `server.rs` makes a card (`claude::card_for`), `hub.add_card` stores it
   and sends `card` + `state`.
4. The island draws it and sends `card.shown` (the ack). No ack in 800 ms →
   "none".
5. A click sends `card.answer`; the daemon replies one line
   (`{"decision":"allow",…}`); the relay renders Claude's JSON
   (`claude::render`). Anything else, including a timeout, prints nothing.

### A chat line

1. The island sends `chat.send {text}` (or `ewe-companion ask`).
2. `chat::send` broadcasts `chat.reply {kind:"thinking"}`, then
   `chat::answer`:
   - `$ cmd` → a proposal (below);
   - the matcher (`intent::match_text`) → `commands::run` → a done line with
     Undo. A miss ("open the pod bay doors": no such app) falls through;
   - the knowledge pack (`pack().search`): with no provider, a confident hit
     is the answer, with its `open:` button and `run:` proposal;
   - otherwise the provider, told the persona, the primer, the topics that
     fit and `context::snapshot`. A reply that is an `ACTION {json}` line
     runs that command (or proposes `run_shell`).
3. The answer goes out as `chat.reply` under the same id.

### A proposed command

1. A model's `run_shell`, a topic's `run:` line or the user's `$ cmd` goes
   through `shell::assess`. Refused: an error, nothing offered. Otherwise
   `hub.keep_proposal` stores it under a one-time token (15 minutes, the
   last 8).
2. The reply carries `confirm {token, command, about, risk, reason,
   terminal}`; the island shows the exact command with Run and In terminal.
3. The click sends `confirm.run {token, mode}`. `chat::confirm` takes the
   token (once), then `runner::run` (bash in home, 120 s, the last 12 lines)
   or `shell_do("term")` (kitty, stays open). The result is a `chat.reply`
   with `output`, and it goes into the conversation for the next turn.

### A `do` round trip

`hub.shell_do(action, args)` broadcasts `do {id, action, args}` and waits
3 s for `do.result {id, ok, error, data}`. The island's `Service.qml`
`perform()` holds one `case` per action: `dnd`, `open`, `launch`, `term`,
`screenshot`, `browse`, `browser` (ewe browser's command line; 20 s, through
`hub.shell_do_within`) and `safe_unlock` (Safe's prompt). No island → "The
island is not running."

## Where each change goes

| you want | touch (details in docs/EXTENDING.md) |
|---|---|
| a new thing it can do on its own (T1) | `core/commands.rs` CATALOG + `validate`, `core/intent.rs` words, `daemon/commands.rs` run arm |
| something only the shell can do | the above + a `do` action: `hub.shell_do` + `Service.qml` `perform()` + PROTOCOL |
| it should understand new words | `core/intent.rs` (specific rules before the generic `open X`) |
| it should know something about ewe | `knowledge/*.md` (a topic: `## Title`, `keys:`, `open:`, `run:`) |
| a guard rule for proposed commands | `core/shell.rs` `assess` + its tests |
| a new provider | `daemon/providers.rs` PROVIDERS (+ an `ask_*` or `ask_cli` arm), the plugin's `manifest.json` choices |
| a fact the model should see | `daemon/context.rs` (installed part or live part) |
| a power through another ewe app's command line (Safe, ewe browser) | `daemon/safe.rs` or `daemon/browser.rs` (one fn per command, returning an `Outcome` with rows and a `private` line), + recipe 1; a fake in `tests/fakes/` and checks in `tests/chat.sh` |
| a CLI verb | `cli/verbs.rs` + `daemon/server.rs` `cli()` + PROTOCOL |
| a protocol message | `core/proto.rs` + `daemon/server.rs` + `Service.qml` `onLine` + PROTOCOL |
| a card kind | `core/proto.rs`, `hub.add_card`, plugin `cards/` + `Drop.qml` Loader |
| a home section or quick action | the plugin: `sections/` + `Drop.qml` body |
| something stored | `daemon/store/migrations/00N_*.sql` + `store.rs` |
| another agent (Codex, OpenCode, Gemini) | P6: a dialect module like `core/claude.rs`, `hook <agent>`, `integrations/<agent>/` |

## Deploying to the owner's desktop

The owner runs the companion live. Only when they ask to try a change:

```sh
cargo build --release && BIN=target/release tests/relay-failsafe.sh   # release timing differs
install -m755 target/release/ewe-companiond target/release/ewe-companion ~/.local/bin/
systemctl --user restart ewe-companion.service
systemctl --user restart ewe.service      # only after QML changes: restarts the whole shell
```

The plugin is a symlink, `~/.local/share/ewe/plugins/ewe.companion` →
`../ewe-plugin-companion`, so QML edits are live after the shell restart.
Restarting the shell is visible to the owner: say so first. Knowledge-pack
edits need the release build (the pack is compiled in).

## Traps already hit

- **The scratch daemon acts on the real machine.** Volume, mute,
  brightness, media, game mode and lock run for real even in the harness
  (lock locks the owner's screen). A test once muted the owner's speakers.
  Test with what the nested shell performs, or restore at once.
- **Never test hooks against the real `~/.claude`.** Use a scratch project
  (docs/TESTING.md).
- **The owner's Claude Code runs in `bypassPermissions`.** It fires no
  `PermissionRequest`, so permission cards never appear for them; questions
  and sessions still work. Test cards with `--permission-mode default`.
- **Real `permission_suggestions` are not the documented example**
  (`addDirectories`, `setMode` …). The card words what it can and leaves the
  rest to the terminal.
- **The daemon's own `claude -p` must not relay to itself:** providers.rs
  sets `EWE_COMPANION_INTERNAL=1`, and the relay goes quiet under it.
- **Unix socket paths hold 108 bytes.** Test sockets go in
  `$XDG_RUNTIME_DIR`, never under a long scratch path.
- **Hyprland 0.56 dispatches in Lua:** `hl.dsp.focus({ window =
  "address:0x…" })`, not `focuswindow pid:`.
- **kitty's remote-control socket is a file** in `$XDG_RUNTIME_DIR`, not an
  abstract `@socket` (no permissions there).
- **A user unit may lack `HYPRLAND_INSTANCE_SIGNATURE`.** The island sends
  the shell's in `env.hypr`; use `hub.env().hypr`.
- **tracing's macros import `debug`:** a local `let debug` breaks
  `info!(…, debug, …)`.
- **Settings 0.18 ignores `--page`.** "Open Settings" opens Appearance, so
  replies name the page to pick (`commands::page_label`). Komble and Quick
  settings pages do open.
- **Never `pkill -f` a pattern that appears in your own command** (it kills
  your shell). Find the PID and check `readlink /proc/PID/exe`;
  `tests/island.sh down` does.
- **ewe browser 0.1.0-beta has no command line.** Its `ewe-browser` is
  the Electron binary, so a verb opens as a page (`ewe-browser commands`
  searches "commands"). `browser::cli()` asks only a `#!` script.
- **An app a command line starts joins the caller's cgroup.** ewe browser's
  `search`/`open`/`play` start the browser when it is closed; from the
  daemon's unit it would die with the unit. Acts go through the island
  (`do browser`); reads stay in the daemon.
- **The conversation is model context.** A reply's text used to be kept
  as is, which would have carried Safe's names into the next question.
  Anything that shows names sets `Outcome::private`; `tests/canary.sh`
  fails otherwise.
- **The real `ewe-safe` reaches the owner's real safe.** Tests and the
  harness use `tests/fakes/` (`EWE_COMPANION_SAFE`,
  `EWE_COMPANION_BROWSER`); `tests/island.sh` sets them unless you name
  others. Never point them at the real ones.
- **QML traps** (a Qt Column skips zero-width children, layer windows blink
  when they grow, focus grabs die when an exclusive layer takes the
  keyboard …) are in the plugin's AGENTS.md.

## Environment variables

| variable | read by | what |
|---|---|---|
| `EWE_COMPANION_SOCK` | daemon, CLI, relay, island | the socket path instead of `$XDG_RUNTIME_DIR/ewe-companion.sock` (tests) |
| `EWE_COMPANION_DEBUG=1` | daemon | allows the `debug.*` verbs (stand-ins for clicks); never set by the unit or the plugin |
| `EWE_COMPANION_WAIT_MS` | daemon (debug only) | how long a hook waits for a human, clamped 500–108000 (the timeout test) |
| `EWE_COMPANION_LOG` | daemon | the tracing filter, e.g. `debug` (default `info`) |
| `EWE_COMPANION_KNOWLEDGE` | daemon | a folder of `.md` files to use instead of the built-in pack |
| `EWE_COMPANION_HOOK_LOG` | relay | append every raw hook payload to this file (recording fixtures) |
| `EWE_COMPANION_INTERNAL=1` | relay | stay quiet: set on the daemon's own `claude -p` runs |
| `EWE_COMPANION_SHARE` | CLI | where `agents install` finds `claude-code/` (the marketplace); else the source tree a dev build sits in, else `/usr/share/ewe-companion/claude-code` |
| `EWE_COMPANIOND` | island | the daemon to start in dev when the unit is not installed |
| `EWE_COMPANION_SAFE` | daemon | the `ewe-safe` to run instead of the one on PATH (tests: `tests/fakes/ewe-safe`) |
| `EWE_COMPANION_BROWSER` | daemon | the `ewe-browser` to run instead of the one on PATH (tests: `tests/fakes/ewe-browser`) |
| `EWE_FAKE_DIR`, `FAKE_SAFE_ITEMS`, `FAKE_SAFE_LOCKED` | the fakes | their state and log dir; the fake Safe's items (a JSON file); start it locked |
| `XDG_STATE_HOME` | daemon | the store goes in `$XDG_STATE_HOME/ewe/companion/` |
| `ISLAND_WORK`, `EWE_DIR`, `PLUGIN_DIR` | `tests/island.sh` | the harness's work dir and the sibling checkouts |

## Open questions for the owner

- **C8 and decision 9 vs P2.8's `run:` lines.** `knowledge/howto.md`
  "Update the system" offers `sudo pacman -Syu` with Run (in a terminal),
  and `fix.md` "The disk is full" offers `sudo paccache -rk1`. C8 says
  packages go through Komble, and decision 9 says cache cleaning becomes a
  Komble verb (P5). Both are the user's own click in their own terminal and
  rule 10 holds, but neither went through the owner as an exception. Ask
  before adding more package commands; when P5's Komble verb lands, the
  cache topic should use it. (Also in the vault's `10-Roadmap/Open
  Questions.md` → *Companion*.)

## Keeping things true (update discipline)

A change is not done until what describes it says so, in the same change:

| changed | update |
|---|---|
| what a user can do | `README.md` (and the plugin's README for the island) |
| a protocol message, field, CLI verb or `do` action | `docs/PROTOCOL.md` |
| a phase task done or added | `docs/PLAN.md` §20 (tick it, say what was checked) |
| something building taught you | `docs/PLAN.md` §25, and *Traps* above |
| a way to extend or test | `docs/EXTENDING.md`, `docs/TESTING.md` |
| ewe itself (a page, a label, a shortcut) | `knowledge/*.md`; the pack must follow ewe |
| status, a contract, a gotcha | the vault: `10-Roadmap/Roadmap — ewe-companion.md`, `Troubleshooting Knowledge.md`, `Contracts and Public API.md`, `ewe-facts.json`; then `project-map/.vault/vault-check.sh --diagrams` |

## Git

Commit or push only when the owner asks. `main` is the default branch:
branch off it first (`feat/…`, `fix/…`). Features that touch several repos
(this one, the plugin, `ewe`, the vault, the website) merge as one wave:
one PR per repo, merge commits, branches kept. The plugin repo has no CI;
this one runs `ci.yml`. Releasing (P7) follows the `ewe-release` skill
(`../.opencode/skills/ewe-release/SKILL.md`): a `v*` tag runs
`.github/workflows/release.yml` (makepkg in `packaging/`, the package on the
GitHub release), then ewe-repo's publish carries it; the plugin reaches
users through ewe's payload (`scripts/vendor-plugins.sh`, an ewe release).
