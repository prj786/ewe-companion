# Extending the companion

One recipe per kind of change: what to touch, in which order, what to
test and which docs to update. Read `AGENTS.md` first (the rules, the map,
how the pieces talk). Paths are relative to this repo; `plugin/` means the
sibling repo `../ewe-plugin-companion`.

The examples use a made-up `night_light` command to show the shape of each
edit; they are not code that exists or a feature that was decided.

Every recipe ends the same way: run the loop (`cargo build`, `cargo test`,
clippy, fmt, `tests/chat.sh`, `tests/relay-failsafe.sh`; the two scripts run
`target/debug` as it is, so build first), look at the island in the
harness when anything visible changed (`docs/TESTING.md`), and update what
describes the change (the table at the end of `AGENTS.md`).

Contents:

1. [A command it runs on its own (T1)](#1-a-command-it-runs-on-its-own-t1)
2. [Something only the shell can do (a `do` action)](#2-something-only-the-shell-can-do-a-do-action)
3. [New words for a command](#3-new-words-for-a-command)
4. [Knowledge: a topic about ewe](#4-knowledge-a-topic-about-ewe)
5. [A command it proposes, and the guard](#5-a-command-it-proposes-and-the-guard)
6. [A provider](#6-a-provider)
7. [A fact the model sees](#7-a-fact-the-model-sees)
8. [A glance row](#8-a-glance-row)
9. [A CLI verb](#9-a-cli-verb)
10. [A protocol message](#10-a-protocol-message)
11. [A card kind](#11-a-card-kind)
12. [A home section or quick action](#12-a-home-section-or-quick-action)
13. [A plugin setting](#13-a-plugin-setting)
14. [Something stored](#14-something-stored)
15. [Another agent (P6)](#15-another-agent-p6)
16. [What comes next: watchers, notifications, tools](#16-what-comes-next-watchers-notifications-tools)
17. [A power through another ewe app (Safe, ewe browser)](#17-a-power-through-another-ewe-app-safe-ewe-browser)

---

## 1. A command it runs on its own (T1)

A T1 command is a desktop action that is safe to run straight from the
user's words: it changes nothing that matters for long, says in one line
what it did, and offers Undo where there is a way back (volume, Do not
disturb, opening things). If it changes settings, files or the system, it is
**not** T1: it belongs behind a confirm (today: a `run_shell` proposal,
recipe 5, or a catalog command that only proposes; later: P4's ConfirmCard).

**A command that only proposes** (T2/T3: `theme_mode`, `power`,
`empty_trash`): a catalog row and words like any other, its name in
`commands::PROPOSALS`, and a run arm that returns
`Outcome { propose: Some((exact command, about)), .. }` and never runs
anything (`daemon/desktop.rs` `proposal`). `chat::reply_of` turns it into a
`confirm` through `shell::assess`, so only the Run click runs it. Read the
state first when that saves a click ("Ewe Light is already on", "the trash
is already empty"), and teach the guard why it is risky (recipe 5).

The catalog has three users: the offline matcher, the persona (which shows
the list to models) and the daemon. One command is four edits.

**a. The catalog row** (`crates/core/src/commands.rs`, `CATALOG`). The
`args` and `summary` are what a model reads, so write them for a model:

```rust
CommandInfo {
    name: "night_light",
    args: r#"{"on": true|false}"#,
    summary: "turn the warm night light on or off",
},
```

**b. Validation** (same file, `validate`). Every call is checked before it
runs, whoever asked for it (the matcher, a model, a button). Use the helpers
(`int`, `one_of`, `text`, `flag`) and refuse anything else:

```rust
"night_light" => flag("on", true),
```

The test `every_catalog_entry_validates_a_good_call` fails until the new
row has a good call in its table: add one.

**c. The words** (`crates/core/src/intent.rs`): recipe 3.

**d. The run arm** (`crates/daemon/src/commands.rs`, `run`). Argv only,
never a shell, within `BUDGET`. Return `done(line, undo)` or
`failed(line)`. The Undo is the call that puts things back, read *before*
the change:

```rust
"night_light" => {
    let on = a["on"].as_bool().unwrap_or(true);
    let was = night_light_on().await;          // read first, for the Undo
    match run_argv("hyprctl", &["hyprsunset", if on { "temperature 4500" } else { "identity" }], &hypr).await {
        Ok(_) => done(
            if on { "Night light on" } else { "Night light off" },
            was.filter(|w| *w != on).map(|w| Call::new("night_light", json!({"on": w}))),
        ),
        Err(_) => failed("Could not change the night light."),
    }
}
```

`hypr` (from `hub.env().hypr`) is the shell's Hyprland instance: pass it to
anything that talks to the compositor. A user unit may not have its own.

When the words can name something that does not exist here (an app, a
folder), return `missed(line)` instead of `failed`: the chat then treats the
words as a question ("open the pod bay doors").

**Nothing checks step d.** Without a run arm everything compiles and every
test passes; the user just gets "No command called …". Try the command in
the harness (`tests/island.sh ctl ask "…"`) before you call it done.

**The name is API once a button uses it.** A quick action in the plugin
(`Actions.qml`) sends the command by name (`command.run`), so after that the
name and its args must never be renamed, only added to.

**Tests.** A row in `intent.rs` tests for the words; `validate` tests for
the bad calls (`bad_calls_are_refused`). Add a `tests/chat.sh` check only for
a command that is safe to run on the dev machine (see TESTING.md: the test
daemon acts for real), or one whose tool is a PATH shim: a fake in
`tests/fakes/` named like the tool (`nmcli`, `ewe-bt`, `qs`, `notify-send`
…) that keeps its state and a log in `$EWE_FAKE_DIR`. Add the tool to the
guard loops at the top of `tests/chat.sh` and `tests/canary.sh` (they refuse
to run when a shim is not first on the daemon's PATH) before the first
check. `ewe-companion debug undo <token>` clicks the reply's Undo.

**Docs.** README (the list of things to say), PLAN §20, and the quick
actions in the plugin if it deserves a button (recipe 12).

## 2. Something only the shell can do (a `do` action)

The daemon is a user unit outside the shell. Anything that must belong to
the desktop session (an app it opens must not die with the daemon's unit),
or that only Quickshell can reach (Do not disturb, panels, the launcher's
app index, screenshots), is asked of the island.

**Daemon side**, in the command's run arm:

```rust
match hub.shell_do("night_light", json!({"on": on})).await {
    Ok(_data) => done("Night light on", None),
    Err(e) => failed(e),          // "The island is not running." / "did not answer"
}
```

`shell_do` broadcasts `do {id, action, args}` and waits 3 s for the
answer. `Ok` carries the island's `data` (e.g. `launch` answers `{name}`).

**Island side**, `plugin/Service.qml` `perform()`: one `case`, and **always**
answer with `root.done(id, ok, error, data)`, even on failure, or the
daemon waits the full 3 s:

```qml
case "night_light":
    Quickshell.execDetached(["hyprctl", "hyprsunset", a.on ? "temperature 4500" : "identity"])
    root.done(id, true)
    break
```

Close the notch first (`CompanionState.dropOpen = false`) when the action
puts something on screen. Use the shell's public API (`Shell.launch`,
`Shell.openSettings`, `Globals.*`; ewe's `docs/PLUGINS.md`) rather than
reaching into the shell's internals.

**Docs.** `docs/PROTOCOL.md`: the `do` row lists every action and its args.
That row is public API between the two repos: a new action is fine, a
renamed one breaks an older island.

**Order matters for users.** A daemon with a new action and an island
without it gets "The island cannot do …". Ship both together (one wave).

## 3. New words for a command

`crates/core/src/intent.rs` turns whole sentences into a `Call` with no
model (C5). The text is cleaned first (lower case, trailing `.!?` and
"please" dropped, spaces collapsed), then the **first** rule whose regex
matches the **whole** text wins.

```rust
r(r"(?:turn |switch )?(?:the )?night ?light (on|off)|(warm|cool) (?:the )?screen",
  |c| {
      let on = match c.get(1).map(|m| m.as_str()) {
          Some(w) => on_word(w)?,
          None => &c[2] == "warm",
      };
      Some(Call::new("night_light", json!({"on": on})))
  }),
```

- **Order.** Specific rules first. Rules in the middle catch a lot:
  `find_files` takes anything starting with find or locate (unless it ends
  in "online"), `web_search` anything starting with search, google or look
  up, and `play_music` anything starting with play. A site's own search
  ("search X on Y", "search Y for X") sits right before the finder's
  `find …` and declines (`site_of`) when Y is a place here, the web as a
  whole, a day, or longer than three words. The calculator rule is first
  and matches everything, but declines unless `calc::eval` reads a sum.
  "go back" stays the music's; "go back a page" is the browser's. The
  desktop's workspaces are numbers, so a browser workspace by number needs
  the word "browser". Safe's rules sit before
  `lock`, so "lock my safe" is Safe's. The last four are
  deliberately generic: an app in a folder, two folder rules, and
  `open|launch|start|run X`. A new rule starting with one of those words must
  go above them, or it never wins.
- **Questions are not commands.** "What is the volume?" must fall through to
  the pack or the model. The test `questions_are_not_commands` lists such
  sentences: add yours.
- **Literal on purpose.** Matching is predictable; anything fuzzy belongs
  to the model, which gets the catalog anyway.
- **Return `None` from `make`** to decline a match the regex let through
  (e.g. a number out of range): the next rule gets a try.

Tests: `is("night light on", "night_light", json!({"on": true}))` in the
matching test function, plus a sentence that must *not* match.

## 4. Knowledge: a topic about ewe

The pack (`knowledge/*.md`) is what makes the companion know ewe. It is
compiled into the binary (`include_str!` in `core/knowledge.rs`), so every
install has it. `primer.md` goes with every question to a model; the other
files hold topics, found by word match.

**Search first:** `grep -n '^## ' knowledge/*.md`. Edit an existing topic
rather than adding a twin: two topics with the same keys tie, and which one
answers then depends on file order. (The pack already has `## Night light`,
which says ewe has none yet; when the made-up `night_light` command lands,
that topic is the one to change.) A topic looks like this:

```markdown
## Night light
keys: night light, warm screen, blue light, eye strain, hyprsunset
open: quicksettings home
run: hyprctl hyprsunset temperature 4500
Quick settings → Night light. It warms the screen after sunset; press Run to turn it on now.

A second paragraph answers a different question about the same topic.
<!-- Sources: ewe/dotfiles/quickshell/…, checked against 0.26.1. Comments never reach a model. -->
```

The `keys:`, `open:` and `run:` lines come right after the title, before the
first line of text (after it, they are just text). `keys:` is required, and
a one-word key may not be a stop word ("help", "show", "set" …: it would
match nearly any question); `the_built_in_pack_parses` checks both.

- **`keys:`** the words people actually use, phrases first. A question that
  contains a whole key phrase is a *confident* match (score ≥
  `CONFIDENT`, 3.0): with no provider, the topic answers by itself.
- **`open:`** (optional) an `open` command, `<what> [page]`, offered as a
  button under the answer ("Open Settings"). The test
  `every_open_line_is_a_valid_command` checks it against `validate`.
- **`run:`** (optional) a shell command offered with Run (recipe 5). The
  test `every_run_line_is_one_the_guard_lets_through` checks it against
  `shell::assess`. Mind C8: package and system work goes through Komble;
  see *Open questions* in `AGENTS.md`.
- **The body.** `Topic::answer` picks the paragraph that fits the question
  (the first wins a tie), so put the common answer first and keep each
  paragraph self-contained. Write for the user, in ewe's words: the exact
  labels from the apps, "Settings → Wallpaper", no jargon.
- **Settings pages.** Settings 0.18 opens on Appearance whatever the page,
  so a Settings answer names the page to pick in the sidebar.
- **Follow ewe.** When a label, a page or a shortcut changes in ewe, change
  the pack in the same wave. Say in a comment what you checked and against
  which version.

**A new file** in `knowledge/` must also be added to `BUILT_IN` in
`crates/core/src/knowledge.rs`. `EWE_COMPANION_KNOWLEDGE` loads every `.md`
in its folder, so a file missing from `BUILT_IN` works in the harness and
silently disappears from the build.

Iterate with `EWE_COMPANION_KNOWLEDGE=<dir of .md files>` on the daemon
(also through `tests/island.sh up`); the daemon reads the pack once, so
restart it after each edit (`tests/island.sh restart`). Then `cargo test`
(the pack parses, the lines validate, `questions_find_their_topic`: add a
question for your topic there) and `tests/chat.sh`, which checks some pack
answers word for word (light mode, the wallpaper, Bluetooth). Deploying to
the owner needs a release build (the pack is in the binary).

## 5. A command it proposes, and the guard

The companion never runs a shell command on its own. Three sources propose
one: a model's `run_shell` ACTION, a topic's `run:` line, the user's
`$ cmd`. All go through `chat::propose` → `shell::assess` →
`hub.keep_proposal` (a one-time token, 15 minutes) → `confirm` in the reply
→ the Run or In terminal click → `chat::confirm`.

To change what the guard thinks (`crates/core/src/shell.rs`):

- **Refused** is for machine killers and house-rule breakers only (rule 1:
  writing `ewe.conf`; rule 9: `generated/`; rule 10: a partial upgrade),
  and for what would print a password manager's values or names
  (`secret_rules`: `ewe-safe read`, `op read`, `pass show` …): a run's
  output goes back into the conversation, and so to a model (C3).
  Loosening a refusal is the owner's call, never yours.
- **Risky** shows the reason in warning ink and a danger-styled Run. Say the
  reason in plain words ("deletes files", "runs as root").
- **Terminal** is for what needs a person: a password (sudo), an editor, a
  pager, a prompt, something that never ends (the `TALKERS` list).

Each rule gets a test sentence in the matching test
(`machine_killers_are_refused`, `risky_commands_warn`, …). The guard is a
guard rail, not a sandbox: the exact command in plain sight, and the click,
are what keep the user in charge.

The runner (`daemon/runner.rs`) runs `bash -c` in home, output and errors
together, 120 s at most (the whole process group is killed after), and keeps
the last 12 lines. A long job belongs in a terminal: if your feature
proposes one, set `terminal: true`.

## 6. A provider

`crates/daemon/src/providers.rs`, `PROVIDERS`:

- **An OpenAI-compatible API** (most of them): one `Info` row with its
  `base_url`, `needs_key` and a cheap, fast `default_model`. Nothing else in
  Rust. `ask()` sends it through the OpenAI-compatible path.
- **A different API**: a row plus an `ask_<name>` function beside
  `ask_anthropic`, and a branch in `ask()`. Map errors through `api_error`
  (a 401/403 says "paste a new key").
- **A CLI agent the user logs in to** (the agent lane): a row with
  `cli: Some("<binary>")` and an arm in `ask_cli` with its print-mode
  arguments, **no tools** (read-only or plan mode), the model if set. The
  child gets `EWE_COMPANION_INTERNAL=1` so our own hooks ignore it. Never
  read or copy its login (C7).

Then:

- `plugin/manifest.json`: add the id to the `provider` setting's `choices`
  and its description; the plugin README's settings table.
- Keys: nothing to do. `key.set` puts any provider's key in the keyring
  under `provider <id>`, and the island asks for it when `needs_key`.
- Test: `tests/mock-llm.py` speaks the OpenAI-compatible protocol; point a
  `custom` provider at it the way `tests/chat.sh` does. Never put a real key
  in a test or a log.
- Docs: README (who can answer), PLAN §11 if it is a new lane.

## 7. A fact the model sees

`crates/daemon/src/context.rs` builds "this computer now" for each question
a model gets. Two parts:

- **installed** (`installed()`, kept 60 s): versions, which ewe apps and
  plugins are here, shortcuts, the look. Slow-changing reads.
- **live** (`live()`): disks, updates, battery, volume, brightness, Do not
  disturb, game mode, agents at work. Read every time, so keep each read
  quick; the whole snapshot runs within 2 s.

Add one line per fact, in the same `- fact` style. Read with ewe's own tools
(they print JSON; `context::tool()` finds them on PATH or in
`~/.local/bin`) or a file. **Never** add anything that names the user's
accounts, servers, mail or contacts, never `ewe.conf` as a whole, never a
secret (C3). Tests: `tests/chat.sh` checks what the mock provider was sent;
extend that check if your fact matters.

## 8. A glance row

The home panel shows a fact only when it needs the user (a disk past 85 %,
waiting updates, a low battery). The daemon reports numbers; the island
decides what is worth a line.

- Daemon: `crates/daemon/src/glance.rs` `read()` returns the `glance` object
  of the `state` snapshot. Add a key; keep the read cheap: it runs when an
  island connects, every ten minutes, and whenever the home panel opens
  (`glance.refresh`).
- Island: `plugin/sections/Glance.qml` builds `rows` from
  `CompanionState.glance`; add a row only past a threshold.
- PROTOCOL: the `glance` line under *Daemon → island*.
- If a model should know it too, add it to `context.rs` live (recipe 7).

## 9. A CLI verb

Two ends:

- `crates/cli/src/verbs.rs` `run()`: parse the words and flags, then
  `ask("verb.name", json!({…}))` (or `ask_within` with a longer wait), and
  add the verb to `VERBS` (what `help` and an unknown verb list). The CLI is
  std-only: no tokio, no new heavy crates.
- `crates/daemon/src/server.rs` `cli()`: one arm returning one JSON object
  with `ok`. Debug-only verbs (stand-ins for clicks) are guarded
  `"debug.x" if hub.debug =>`; a daemon without `EWE_COMPANION_DEBUG=1`
  answers them "debug verbs need a daemon started with
  EWE_COMPANION_DEBUG=1".

Rule 5: print one JSON object, exit 0, always, including when the daemon is
not running (`not_running()`). Docs: the CLI table in PROTOCOL.md and the
README's *Use it* block.

## 10. A protocol message

The protocol is public API between our repos: add, never rename or remove.
An unknown `type` is ignored on both ends, so a newer client still talks to
an older daemon.

- **Island → daemon:** a variant in `crates/core/src/proto.rs` `Inbound`
  with `#[serde(rename = "x.y")]`, an arm in `server.rs` (the island loop),
  and the sender in `plugin/CompanionState.qml` (a function that calls
  `st.send({type: "x.y", …})`).
- **Daemon → island:** `hub.broadcast(proto::line("x.y", json!({…})))`, and
  a `case "x.y":` in `plugin/Service.qml` `onLine` that writes into
  `CompanionState`. If the island draws it, prefer adding a field to the
  `state` snapshot (`hub.snapshot()`): every island is then right after any
  reconnect.
- **Docs:** the tables in `docs/PROTOCOL.md`, in the same change.
- **Tests:** a decode test in `proto.rs`; `tests/fake-island.py` prints
  every message it gets, handy for an end-to-end check.

## 11. A card kind

A card is something that waits for an answer (`proto::Card`, `kind`:
`permission`, `question`, `note`; `confirm` and `digest` are reserved for
P4 and P3).

**Often a card is not what you need.** A list the user picks from (Safe's
logins, found files, the browser's tabs) is `rows` on the chat reply
(`Outcome::rows`; PROTOCOL's *Rows*): no ack, no deadline, nothing stored,
and each row's buttons are catalog commands. `hub.add_card` stores the
card's title in `companion.db`, so never put a Safe name in a card.

- Daemon: build the `Card` (id, kind, title, body, actions, `deadline_ms`,
  payload) and `hub.add_card(card, wait)`. `wait` is set when someone (a
  hook) blocks on the answer; `server.rs` `wait_for_human` then enforces the
  800 ms ack and the deadline. An answer arrives in `hub.card_answer`.
- Island: `plugin/cards/<Kind>Card.qml` with `property var card` and the
  answer through `CompanionState.answer(card.id, action, answers)`; a
  `Component` for it and its branch in `Drop.qml`'s card `Loader`
  (`sourceComponent`). The Loader sends the ack (`CompanionState.shown`) on
  load: if your card can fail to load, the agent falls back to its terminal
  (C2), which is the point.
- Never answer for the user: no default action, no timeout that allows.
- Docs: PROTOCOL's *Card* section; the plugin README.

## 12. A home section or quick action

Island only, in the plugin repo; see its `AGENTS.md` for the QML rules.

- **A quick action:** one entry in `plugin/sections/Actions.qml` `buttons`
  (`ic` a `Theme.ic*` icon, `label`, `on`, `cmd`, `args`). It runs a catalog
  command through the daemon, so it needs recipe 1 first.
- **A section:** `plugin/sections/<Name>.qml`, placed in `Drop.qml`'s body
  `Column` with `width: body.width` and a `visible:` that is true only when
  there is something live to show (the owner: "if memory is okay I don't
  care looking at it").

## 13. A plugin setting

`plugin/manifest.json` `settings` (types: `choice`, `string`, `bool`,
`int`). The shell hands them to the plugin as `settings`; they live in
`ewe.conf` under `[plugins.settings]` and sync to the user's account, so
**never a secret** (rule 2). If the daemon needs one, `Service.qml`
`sendSettings()` sends it in the `settings` message (today: provider, model,
base_url; `proto::ChatSettings`). Document it in the plugin README's
settings table.

## 14. Something stored

`~/.local/state/ewe/companion/companion.db`, SQLite in WAL mode, per
machine, never synced.

- A new table or column is a new file
  `crates/daemon/src/store/migrations/00N_<what>.sql`, appended to
  `MIGRATIONS` in `store.rs` (the example: `002_timers.sql`, the timers
  that must survive a daemon restart; `Store::timer_add` and friends, a
  test in `store.rs`). `PRAGMA user_version` records the last one
  applied. Never edit a migration that has shipped.
- `001_init.sql` already has the tables later phases fill (notifications,
  chats, messages, memory, nudges, usage): use them before adding one.
- Logs carry ids and counts, never what is stored. Respect the retention
  (`KEEP_DAYS`).
- Docs: PLAN §6 and the vault's `12-Reference/Storage Map.md`.

## 15. Another agent (P6)

Claude Code is the only agent today. A new one (Codex, OpenCode, Gemini) is
the same three layers:

- **The dialect** (pure, in core, like `core/claude.rs`): which events wait,
  the card each becomes, and the exact output its hook expects for allow,
  deny, an answer, and "no opinion". Verify each against the agent's live
  docs and a recorded payload; never trust the documented example alone.
- **The relay**: `ewe-companion hook <agent>` (`cli/src/hook.rs` returns
  early for anything but `claude` today, and so does the daemon:
  `server.rs` `agent()` drops any other `msg.agent`). Same fail-safes:
  print nothing and exit 0 on any failure.
- **The integration**: `integrations/<agent>/` and `ewe-companion agents
  install <agent>` (`cli/src/verbs.rs`), changing nothing the user owns
  outside a plugin or a clearly named file.
- **Tests**: recorded payloads in `tests/payloads/<agent>/` (record them in a
  scratch project with `EWE_COMPANION_HOOK_LOG`), and the fail-safe checks
  of `tests/relay-failsafe.sh` for the new dialect.

PLAN §10.2 has the research per agent; re-verify it first.

## 16. What comes next: watchers, notifications, tools

These have plans, not code. Read the PLAN section before starting, and keep
the phase's gate in mind:

- **P3, notifications** (PLAN §9): crosses into `ewe` (Plugin API 4, the
  core watchdog, IPC `notifications`) and merges as one wave. The watchdog
  is C2 for banners: if the plugin does not present a notification in time,
  the stock banner shows.
- **P4, the brain** (§11, §12, §16): streaming, native tool calls, tiers and
  taint (C4), redaction and the canary test (C3). T2/T3 tools get a
  ConfirmCard.
- **P5, watchers and MCP** (§13, §12.4): deterministic watchers (storage,
  updates, sync …) under the attention budget (C6); `ewe-companion mcp` for
  terminal agents; package-cache cleaning through a Komble verb
  (decision 9).
- **P7, ship**: the PKGBUILD, ewe-repo, Komble's first-party list, ewe's
  `plugins/bundle.json` (`default: false`), via the `ewe-release` skill.

## 17. A power through another ewe app (Safe, ewe browser)

Safe (`daemon/safe.rs`) and ewe browser (`daemon/browser.rs`) are driven
through their own command lines (ewe-safe `docs/PROTOCOL.md`, ewe-browser
`docs/CLI.md`): argv only, one JSON object back (the last line), rule 5 on
both sides. A new command on top is recipe 1, with these rules:

- **One function per command** in the app's module, returning an
  `Outcome`. Use `rows` for a list to pick from, `action` for one button
  ("Unlock Safe"), `undo_browser` for the browser's `undo` (only its own
  undoing verbs pass `browser::undo_steps`).
- **Set `private`** whenever the line names something the app holds: a
  login's title or username, a page's title, a file's name, a track's tags.
  `private` is what the conversation keeps, and so what a model hears:
  "done" or a count. A Safe outcome always sets it (`safe::failed_with`
  does for failures, since Safe's own sentences may name an item).
  `tests/canary.sh` fails when a name leaks.
- **Values never cross.** Safe's `show` and `reveal` are never called;
  `copy` makes Safe put the value on the clipboard itself. Never add a
  verb that brings a password, a code, a key or a note into the daemon.
- **Acts that may start an app run in the island** (`do browser`, `do
  safe_unlock`): an app started from the daemon's unit dies with it.
  Reads (`summary`, `match`, `tabs`, `history`, `page`) run in the daemon.
- **Respect the app's tiers and its untrusted marks.** ewe browser's
  `commands` lists each tool's tier (2 → a proposal, never a run) and
  whether its reply can hold a website's words (`untrusted`): such words
  reach a model only wrapped as `<untrusted>` in a tainted turn
  (`chat::about_page`), never as plain context.
- **Detect, then fall back.** `browser::cli()` asks `commands` once per
  binary (and only a `#!` script: 0.1.0-beta's binary would open a page);
  without it, `web_search` and friends open a URL the old way. A tool the
  browser does not list answers "update it in Komble"; an argument a tool
  is new to (`search`'s `site`, `open`'s `workspace`) is checked with
  `Cli::takes`, which reads the tool's schema, and an older browser gets
  the old behaviour, said in the line.
- **New verbs by their argv.** A verb whose JSON argument names the
  browser's docs do not spell out (`workspace activate <name>`, `back`,
  `download show <id>`) runs in its command-line form through
  `browser::act_verb` (tier checked by the tool's name), not through `call`
  with guessed names. Its `undo` steps must be in `undo_steps`' list.
- **Tests:** teach the fake (`tests/fakes/ewe-safe`, `tests/fakes/ewe-browser`)
  the new verb, then a check in `tests/chat.sh` (the scripted island
  `tests/fake-island.py --do` runs the island's half). A verb that shows
  names: put `$FAKE_BROWSER_MARK` into what the fake answers and a chat
  in `tests/canary.sh` that asks for it. Never the real
  `ewe-safe` or `ewe-browser`: they reach the owner's safe and browser
  profile. `EWE_COMPANION_SAFE` and `EWE_COMPANION_BROWSER` point the
  daemon at the fakes.
