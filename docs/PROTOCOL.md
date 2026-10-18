# The companion socket protocol, v1

This is the contract between `ewe-companiond` and its clients: the island
(plugin `ewe.companion`), the CLI, and the agent hook relay. Message types are
public API between our own repos. Fields may be added; nothing is renamed or
removed (the rule-4 spirit). The types live in `crates/core/src/proto.rs`.

## Transport

- **Socket.** `$XDG_RUNTIME_DIR/ewe-companion.sock`, mode `0600`. Tests may set
  `EWE_COMPANION_SOCK` instead.
  - The directory must be a real directory owned by the user and writable by
    nobody else, or no client connects and the daemon refuses to bind.
  - A Unix socket path is limited to 108 bytes (`SUN_LEN`), so keep test
    sockets short.
- **Peers.** Both ends check `SO_PEERCRED`: the same uid, or the connection
  is dropped.
- **Framing.** UTF-8 JSON, one object per line, `\n`-terminated, at most
  1 MiB per line. A longer line drops the connection.
- **Envelope.** `{"v":1,"type":"…", …}`.
  - A missing `v` reads as 1.
  - A newer major is refused with `{"type":"error","error":"newer-protocol"}`.
  - An unknown `type` is ignored, so a newer client still talks to an older
    daemon.

The first line of a connection says what it is: `hello`, `agent` or `cli`.

## The island (`hello`)

```json
{"v":1,"type":"hello","client":"island","pid":0}
```

The daemon replies `welcome`, then a full `state`, and keeps the connection.

### Daemon → island

| type | fields | notes |
|---|---|---|
| `welcome` | `daemon` (version), `caps` | once, first |
| `state` | `presence`, `pending: [Card]`, `sessions: [Session]`, `quiet: {locked, dnd}` | after every change; the island redraws from it |
| `peek` | `key`, `line`, `level` (`info` · `success` · `warning` · `danger`), `actions: [Action]`, `ttl_ms` (0 = the island's own `peek_seconds`) | one line; the same `key` replaces the line |
| `card` | a `Card` | followed at once by a `state` that includes it |
| `card.close` | `id`, `reason` | `answered` · `answered-elsewhere` · `no-ack` · `timeout` · `locked` · `island-gone` · `agent-gone` · `daemon-stopping` |
| `chat.reply` | `id`, `kind` (`thinking` · `answer` · `done` · `error` · `info`), `text`, `undo?` (a token), `command?`, `action?` (`{label, command, args}`), `confirm?` (`{token, command, about, risk, reason, terminal}`), `output?`, `rows?` | `thinking` first, then the answer under the same `id` (a reply that comes later by itself, such as the copy that waited for Safe's unlock, has its own `id` and no `thinking`). `action` is a button under the answer ("Open Settings", "Unlock Safe", a tainted turn's "Do it"); a click sends it back as `command.run`. `confirm` is a proposed shell command: the island shows it exactly, with Run and In terminal (only In terminal when `terminal`), and `reason` in warning ink when `risk` is `danger`; a click sends `confirm.run`. `output` is the last lines a command printed. `rows` is what a command found (see *Rows*) |
| `do` | `id`, `action`, `args` | what only the shell does: `dnd {on}` · `open {what, page}` · `launch {app, path?, dir?}` · `term {command}` (kitty runs it in the user's shell and stays open) · `screenshot {mode}` · `browse {url}` (ewe-browser when installed, else the default browser) · `browser {bin, args}` (ewe browser's command line: the island runs `bin` + `args` as argv, `bin`'s name must be `ewe-browser`, and answers `data: {reply}`, the last line it printed, parsed; the daemon waits 20 s, since it may start the browser) · `safe_unlock {bin}` (runs `bin unlock --ask`, Safe's own prompt, detached; `bin`'s name must be `ewe-safe`). The island answers `do.result` within 3 s (`browser`: 20 s). `launch` finds `app` (a desktop id, a name, a program) in the shell's desktop-entry index and answers `data: {name}`, or `ok:false` with `error:"not-found"` |

`presence` is one of `idle`, `working`, `waiting` or `error`. The `state`
snapshot also carries:

- `chat`: `{provider, label, ready, needs_key, hint, model}`;
- `glance`: `{disks: [{path, used_pct, free_gb}], updates: {repo, aur, total}}`;
- `usage`: `{five_hour: {pct, resets_at}, seven_day: {…}}`, or null;
- `game_mode`: true while game mode is on.

Each session has `steps`, its last four actions.

`open` (`companion_core::commands`) takes `what` (`settings` · `komble` ·
`quicksettings` · `overview` · `sync` · `terminal` · `files`) and an
optional `page`: a Settings sidebar id (`displays`, `wallpaper`, `input`, …),
a Komble page (`updates` · `plugins` · `settings` · `search=<words>` ·
`options=<plugin id>`), or a Quick settings tab (`home` · `wifi` · `bt` ·
`audio` · `cal` · `notifs`, plus plugin tabs).

### Rows

What a command found, under its line: Safe's logins, files and folders,
ewe browser's history and tabs.

```json
"rows": [{"title": "ewe-safe", "detail": "~/Projects/ewe", "icon": "folder",
          "actions": [{"label": "Open", "command": "open_folder", "args": {"path": "/home/you/Projects/ewe/ewe-safe"}},
                      {"label": "Copy path", "copy": "/home/you/Projects/ewe/ewe-safe"}]}]
```

- `icon`: `folder` · `file` · `web` · `key`; since wave 2 also `download`
  (ewe browser's downloads) · `workspace` (its workspaces) · `timer` ·
  `bluetooth`. An island that does not know an icon draws a chevron.
- A click on the row does its first action; the others are small buttons.
- An action with `command` is sent back as `command.run` (a catalog
  command, validated like any other). An action with `copy` never leaves
  the island: it puts that text on the clipboard (`wl-copy`).
- A Safe row's action is `safe_copy {id, field, title, username}`: an item
  id and the words to show, never a value.

### The catalog's newer commands

`find_files {query, kind?}` · `open_file {path}` · `browser_history
{query}` · `browser_tabs {}` · `browser_tab {words | id}` · `safe_copy
{query | id, field: password|username|otp}` · `safe_login {query}` ·
`safe_lock {}` · `safe_unlock {then?}` (`then`: the Safe command that
waited for the unlock) · `safe_status {}`. `web_search`, `play_music` and
`open_url` use ewe browser's command line (`search`, `play`, `open`) when it
has one. What these find is shown on the island; the conversation, and so a
model, keeps a line without names ("done", a count).

Wave 2 (added, nothing renamed; a quick action may use any of them):

| command | args | tier | what |
|---|---|---|---|
| `web_search` | `query`, `site?` | 1 | with `site` (a name or a domain, one short argument): ewe browser's `search --site`; `unknown-site` or a browser without it → a plain search of "`query` on `site`" |
| `open_url` | `url`, `workspace?` | 1 | `open … --workspace` (JSON `workspace`); an unknown name says so |
| `browser_bookmark` | — | 1 | `bookmark_add` of the page in front (`page`); Undo is the browser's `bookmark remove <url>` |
| `browser_bookmarks` | `query?` | 0 | rows with Open |
| `browser_nav` | `action`: `back` · `forward` · `reload` · `reopen` | 1 | `back` / `forward` (0.4), `tab_reload` of the tab in front, `tab_reopen` |
| `browser_tab_do` | `action`: `mute` · `unmute` · `pin` · `unpin` · `close`, `words?` \| `id?` | 1 (close: the browser's 2, a proposal) | the tab these words fit, else the one in front; several → rows |
| `browser_workspace` | `action`: `list` · `switch` · `next` · `prev` · `new`, `name?` | 0 / 1 | `workspaces`, `workspace activate|next|prev|create` (0.4); the words for switching need "browser" |
| `browser_downloads` | — | 0 | rows: Open and Show in folder (finished), Show in folder (others) |
| `browser_download` | `id`, `action`: `open` · `show`, `path?` | 1 | `download open|show <id>` (0.4) while the browser runs, else the file ewe browser's own list names for the id (`path` is never opened) |
| `browser_page` | `page`: `home` · `history` · `settings` · `files` · `music` | 1 | `home`, or `open app://<page>/` |
| `wifi` | `on` | 1 | `nmcli radio wifi on|off` |
| `bluetooth` | `on` | 1 | `ewe-bt power on|off` |
| `bluetooth_device` | `name` \| `mac`, `connect` | 1 | `ewe-bt status` finds the paired device, then `connect|disconnect <MAC>`; several → rows |
| `airplane_mode` | `on`, `wifi?`, `bluetooth?` | 1 | Wi-Fi and Bluetooth off; off puts back what was on (the Undo carries it in `wifi`/`bluetooth`) |
| `keep_awake` | `on` | 1 | `qs ipc call ewe.insomnia on|off` (read first with `status`) |
| `calc` | `expr` | 0 | arithmetic and percentages, offline (`companion_core::calc`) |
| `timer` | `seconds` \| `at` ("17:30"), `text?`, `reminder?` | 1 | kept in `companion.db` (`timers`, 50 at most); `notify-send` as Shaun when due (several due at once: one notification); Undo cancels |
| `timers` | — | 0 | rows with Cancel |
| `timer_cancel` | `id?` | 1 | that one; no id: the only one, or rows |
| `theme_mode` | `mode`: `dark` · `light` | 2 | a proposal: `ewe-theme scheme apply ewe-dark|ewe-light` (or "already on") |
| `power` | `action`: `suspend` · `restart` · `shutdown` · `logout` | 3 | a proposal: `~/.config/hypr/scripts/power.sh suspend|reboot|poweroff|logout` |
| `empty_trash` | — | 3 | a proposal: `gio trash --empty` (or "already empty") |

`theme_mode`, `power` and `empty_trash` never run from a call
(`commands::PROPOSALS`): their reply carries `confirm`, like `run_shell`,
and only the Run click runs them. Names stay local here too: a tab's,
bookmark's, download's, workspace's or device's name is on the island only.

`launch_app` takes `app` (a name: "nemo", "zed", "the browser", "terminal")
and an optional `path` (words or a path: "documents", "~/Projects/ewe",
"ewe-companion"); `open_folder` takes `path`. The daemon finds the folder
(`daemon::places`) and the default app behind "the browser" (`xdg-mime`);
the island finds the app. Words that name neither come back as a *miss*,
and the chat treats them as a question.

### Island → daemon

| type | fields | notes |
|---|---|---|
| `card.shown` | `id` | **the ack**. Without it within 800 ms, the daemon answers the agent "no opinion" |
| `card.answer` | `id`, `action`, `answers?` | a click; see *Actions* |
| `env` | `locked`, `dnd`, `fullscreen`, `casting`, `idle`, `hypr`, `wayland` | sent on connect and on every change. `hypr` is the shell's `HYPRLAND_INSTANCE_SIGNATURE`, for jump-to-terminal; `wayland` its `WAYLAND_DISPLAY`, for what Safe copies (a user unit may have none) |
| `session.jump` | `id` | bring that session's terminal forward |
| `settings` | `provider`, `model`, `base_url` | who answers questions (the plugin's options). **Never a key** |
| `chat.send` | `text` | a question or a command in plain words |
| `key.set` | `provider`, `key` | an API key typed into the island; the daemon puts it in the keyring (`secret-tool`, on stdin) |
| `undo` | `token` | run the Undo a reply offered |
| `command.run` | `command`, `args` | a quick-action button: one catalog command (`companion_core::commands`), validated like any other |
| `glance.refresh` | — | the home panel opened: read disk and updates again |
| `confirm.run` | `token`, `mode` (`run` · `terminal`) | the Run / In terminal click on a proposed command; a token works once, for 15 minutes |
| `do.result` | `id`, `ok`, `error?`, `data?` | the answer to a `do` |

### Card

```json
{"id":"c19a…","kind":"permission","source":"agent:claude","title":"Claude wants to run",
 "body":"npm install","project":"ewe-browser","session":"<session id>",
 "actions":[{"id":"allow","label":"Allow","style":"primary"},
            {"id":"deny","label":"Deny","style":"secondary"},
            {"id":"always:0","label":"Always for npm install","style":"ghost",
             "hint":"Saved for ewe-browser/, just for you"},
            {"id":"terminal","label":"Answer in terminal","style":"ghost"}],
 "deadline_ms":1791950871234,"created_ms":1791950763234,
 "payload":{"tool":"Bash","detail":{"kind":"command","text":"npm install","description":"…"}}}
```

`kind` is one of `permission`, `question`, `note`, `confirm` or `digest`.
`payload.detail.kind` says how the card previews its tool call:

| kind | what it carries |
|---|---|
| `command` | `text` (the command) and `description` |
| `diff` | `file` and `edits: [{old, new}]` |
| `text` | `text` |
| `json` | `text`, pretty-printed |

A question card's `payload` is
`{"questions":[{question, header, multiSelect, options:[{label, description}]}]}`.

### Actions

The `action` field of a `card.answer` takes one of these values:

| action | the agent gets |
|---|---|
| `allow` | allow |
| `always:<n>` | allow, plus Claude's own `permission_suggestions[n]` echoed back. An index Claude did not offer is "no opinion" |
| `deny` | deny, with *Denied from the ewe companion* |
| `answer` + `answers` | for a question: allow, with `{question: label}` (an array of labels for multi-select). Every question must be answered, and every label must be one of its options; otherwise "no opinion" |
| `terminal` | "no opinion", and the daemon brings the terminal forward |
| anything else (`later`, Esc) | "no opinion" |

## A hook relay (`agent`)

The relay opens one connection per hook call:

```json
{"v":1,"type":"agent","agent":"claude","event":"PermissionRequest","wait":true,
 "term":{"kitty_window_id":"1","kitty_listen_on":"unix:/run/user/1000/kitty-4242.sock",
         "kitty_pid":"4242","tmux":null,"tmux_pane":null,"term_program":null,
         "wezterm_pane":null,"ppid_chain":[51234,51200,4242]},
 "payload":{ …the hook's stdin JSON, trimmed… }}
```

- **Trimming.** The relay drops `tool_response` and `transcript_path`, and
  cuts every string to 2 KiB. Diff fields keep up to 256 KiB each, 512 KiB in
  total.
- **`wait:false`.** The daemon folds the event into its session and closes.
- **`wait:true`.** Only `PermissionRequest`, and `PreToolUse` for
  `AskUserQuestion`. The daemon replies exactly one line:

  ```json
  {"decision":"allow"|"deny"|"none","always":0,"answers":{…},"message":"…"}
  ```

  `always`, `answers` and `message` appear only when they apply.

The relay then renders the reply into Claude Code's own JSON
(`crates/core/src/claude.rs`). `none` prints nothing.

### Timing

| step | budget | then |
|---|---|---|
| the relay connects | 300 ms | nothing printed, exit 0 |
| the island acks the card | 800 ms | `none`, and `card.close` with `no-ack` |
| a human answers | 108 s (the relay waits 110 s; the hook times out at 120 s) | `none`, and `card.close` with `timeout` |
| the relay hangs up (answered in the terminal) | — | `card.close` with `answered-elsewhere` |
| the screen locks, or the last island leaves | — | `none` at once |

## The CLI (`cli`)

```json
{"v":1,"type":"cli","verb":"say","args":{"text":"Updates done","level":"success"}}
```

The daemon replies one JSON object and closes. The CLI prints the reply and
exits 0. It prints `{"ok":false,"running":false,"error":"not-running"}` when
there is no daemon.

| verb | args | reply |
|---|---|---|
| `status` | — | `ok`, `version`, `island`, `islands`, `pending`, `sessions`, `presence`, `events`, `debug` |
| `say` | `text`, `level`, `key`, `source`, `ttl_s` | `ok`, `shown` (whether an island got it) |
| `history` | `limit` | `ok`, `events` |
| `sessions` | — | `ok`, `sessions` |
| `jump` | `session` | `ok`, `window`, `tab` |
| `ask` | `text` | a chat reply (`kind`, `text`, `undo`, `command`, `action`, `rows`) and `ok` |
| `find` | `query`, `kind` (`any` · `folder`), `limit` | `ok`, `results: [{path, kind, modified}]` (best first, then newest; `modified` in seconds), `partial` (the 1.5 s budget ran out) |
| `settings` | `provider`, `model`, `base_url` | `ok`, `chat` |
| `key.set` / `key.clear` | `provider` (+ `key`) | `ok` |
| `usage` | `agent`, `rate_limits` | `ok`. Only the status-line relay sends it |
| `debug.card` | `kind` | `ok`, `id`. Only for a daemon started with `EWE_COMPANION_DEBUG=1` |
| `debug.answer` | `id`, `action`, `answers` | `ok`. Same condition: a stand-in for a click, so tests can run without a pointer |
| `debug.confirm` | `token`, `mode` | a chat reply with `output`. Same condition: a stand-in for the Run click |
| `debug.run` | `command`, `args` | a chat reply. Same condition: a stand-in for a button's `command.run` (a row's Copy, Unlock Safe) |
| `debug.undo` | `token` | a chat reply. Same condition: a stand-in for the Undo click (`ewe-companion debug undo <token>`) |
