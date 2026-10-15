//! `ewe-companion <verb>`: JSON out, exit 0, always (rule 5).

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use companion_core::{VERSION, proto};
use serde_json::{Value, json};

use crate::client;

const VERBS: &[&str] = &[
    "status", "say", "ask", "find", "key", "history", "sessions", "jump", "agents", "version",
    "help", "debug",
];

fn not_running() -> Value {
    json!({"ok": false, "running": false, "error": "not-running", "version": VERSION})
}

fn ask(verb: &str, args: Value) -> Value {
    ask_within(verb, args, Duration::from_secs(5))
}

fn ask_within(verb: &str, args: Value, wait: Duration) -> Value {
    let msg = json!({"v": proto::PROTOCOL, "type": "cli", "verb": verb, "args": args});
    client::request(&msg, wait).unwrap_or_else(not_running)
}

/// `--level warning` style flags after the positional words.
struct Args<'a> {
    words: Vec<&'a str>,
    flags: Vec<(&'a str, &'a str)>,
}

fn parse(args: &[String]) -> Args<'_> {
    let mut words = Vec::new();
    let mut flags = Vec::new();
    let mut it = args.iter().map(String::as_str);
    while let Some(a) = it.next() {
        match a.strip_prefix("--") {
            Some(name) => flags.push((name, it.next().unwrap_or(""))),
            None => words.push(a),
        }
    }
    Args { words, flags }
}

impl Args<'_> {
    fn flag(&self, name: &str) -> Option<&str> {
        self.flags.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
    }
}

pub fn run(argv: &[String]) -> Value {
    let verb = argv.first().map(String::as_str).unwrap_or("help");
    let a = parse(argv.get(1..).unwrap_or(&[]));
    match verb {
        "status" => {
            let mut v = ask("status", json!({}));
            if v.get("ok") == Some(&Value::Bool(true)) {
                v["running"] = Value::Bool(true);
            }
            v
        }
        "say" => {
            let text = a.words.join(" ");
            if text.trim().is_empty() {
                return json!({"ok": false, "error": "say needs some text"});
            }
            ask(
                "say",
                json!({
                    "text": text,
                    "level": a.flag("level").unwrap_or("info"),
                    "key": a.flag("key").unwrap_or(""),
                    "source": a.flag("source").unwrap_or("cli"),
                    "ttl_s": a.flag("ttl").and_then(|t| t.parse::<u64>().ok()).unwrap_or(0),
                }),
            )
        }
        "history" => ask(
            "history",
            json!({"limit": a.flag("limit").and_then(|l| l.parse::<u64>().ok()).unwrap_or(20)}),
        ),
        "sessions" => ask("sessions", json!({})),
        "jump" => match a.words.first() {
            Some(id) => ask("jump", json!({"session": id})),
            None => {
                json!({"ok": false, "error": "jump needs a session id (see: ewe-companion sessions)"})
            }
        },
        "agents" => agents(&a),
        "debug" => debug(&a),
        // the chat box from a terminal: a command runs, a question goes to
        // the provider chosen in the plugin's options (CLI agents get 2 min)
        "ask" => {
            let text = a.words.join(" ");
            if text.trim().is_empty() {
                return json!({"ok": false, "error": "ask needs some text"});
            }
            ask_within("ask", json!({"text": text}), Duration::from_secs(130))
        }
        // the finder from a terminal: files and folders under home by name
        "find" => {
            let query = a.words.join(" ");
            if query.trim().is_empty() {
                return json!({"ok": false, "error": "find needs words to look for"});
            }
            ask_within(
                "find",
                json!({
                    "query": query,
                    "kind": a.flag("kind").unwrap_or("any"),
                    "limit": a.flag("limit").and_then(|l| l.parse::<u64>().ok()).unwrap_or(20),
                }),
                Duration::from_secs(10),
            )
        }
        "key" => key(&a),
        "version" | "--version" | "-V" => json!({"ok": true, "version": VERSION}),
        "help" | "--help" | "-h" => json!({
            "ok": true, "version": VERSION, "verbs": VERBS,
            "usage": [
                "status", "say <text> [--level info|success|warning|danger] [--key k] [--ttl s]",
                "history [--limit n]", "sessions", "jump <session>",
                "ask <text>   (a command runs at once; a question goes to your provider)",
                "find <words> [--kind folder] [--limit n]   (files and folders under home, by name)",
                "key set <provider>   (the key on stdin; it goes to the keyring) | key clear <provider>",
                "agents list | install claude | uninstall claude | doctor | usage claude on|off",
                "statusline   (Claude Code's status line relay, for plan usage)",
                "hook claude [<Event>]   (the Claude Code relay; reads the hook payload on stdin)",
                "debug card permission|question|note   (needs a daemon started with EWE_COMPANION_DEBUG=1)",
                "debug answer <card> <action> [answers-json]",
                "debug run <command> [args-json]   (a button's click: a row's Copy, Open …)",
            ],
        }),
        other => json!({"ok": false, "error": format!("unknown verb: {other}"), "verbs": VERBS}),
    }
}

fn debug(a: &Args) -> Value {
    match a.words.first().copied() {
        Some("card") => ask(
            "debug.card",
            json!({"kind": a.words.get(1).copied().unwrap_or("permission")}),
        ),
        Some("answer") => {
            let (Some(id), Some(action)) = (a.words.get(1), a.words.get(2)) else {
                return json!({"ok": false, "error": "debug answer <card> <action> [answers-json]"});
            };
            let answers = a
                .words
                .get(3)
                .and_then(|j| serde_json::from_str::<Value>(j).ok());
            ask(
                "debug.answer",
                json!({"id": id, "action": action, "answers": answers}),
            )
        }
        Some("confirm") => {
            let Some(token) = a.words.get(1) else {
                return json!({"ok": false, "error": "debug confirm <token> [run|terminal]"});
            };
            // a run may take the runner's whole two minutes
            ask_within(
                "debug.confirm",
                json!({"token": token, "mode": a.words.get(2).copied().unwrap_or("run")}),
                Duration::from_secs(130),
            )
        }
        // a button's click (`command.run`): a row's Copy, Open, Unlock Safe …
        Some("run") => {
            let Some(command) = a.words.get(1) else {
                return json!({"ok": false, "error": "debug run <command> [args-json]"});
            };
            let args = a
                .words
                .get(2)
                .and_then(|j| serde_json::from_str::<Value>(j).ok())
                .unwrap_or_else(|| json!({}));
            ask_within(
                "debug.run",
                json!({"command": command, "args": args}),
                Duration::from_secs(30),
            )
        }
        _ => {
            json!({"ok": false, "error": "debug card <kind> | debug answer <card> <action> | debug confirm <token> [run|terminal] | debug run <command> [args-json]"})
        }
    }
}

// ── agents: the Claude Code integration (decision 8: a Claude Code plugin
//    from a local marketplace; nothing in ~/.claude/settings.json changes) ──

const MARKETPLACE: &str = "ewe";
const PLUGIN: &str = "ewe-companion@ewe";

/// Where the marketplace directory is: `$EWE_COMPANION_SHARE/claude-code`,
/// the source tree next to a dev build, or the packaged copy.
fn integration_dir() -> Option<PathBuf> {
    let mut cands = Vec::new();
    if let Some(share) = std::env::var_os("EWE_COMPANION_SHARE") {
        cands.push(PathBuf::from(share).join("claude-code"));
    }
    if let Ok(exe) = std::env::current_exe() {
        // target/{debug,release}/ewe-companion → <repo>/integrations/claude-code
        if let Some(repo) = exe
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
        {
            cands.push(repo.join("integrations/claude-code"));
        }
    }
    cands.push(PathBuf::from("/usr/share/ewe-companion/claude-code"));
    cands
        .into_iter()
        .find(|d| d.join(".claude-plugin/marketplace.json").is_file())
}

/// Run `claude …`, returning one step of the report.
fn claude(args: &[&str]) -> Value {
    match Command::new("claude").args(args).output() {
        Ok(o) => {
            let out = String::from_utf8_lossy(&o.stdout);
            let last = out
                .lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("");
            json!({
                "run": format!("claude {}", args.join(" ")),
                "ok": o.status.success(),
                "result": serde_json::from_str::<Value>(last).unwrap_or_else(|_| Value::String(last.to_string())),
                "stderr": String::from_utf8_lossy(&o.stderr).trim().chars().take(400).collect::<String>(),
            })
        }
        Err(e) => {
            json!({"run": format!("claude {}", args.join(" ")), "ok": false, "error": e.to_string()})
        }
    }
}

fn which(cmd: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(cmd))
        .find(|p| p.is_file())
}

fn agents(a: &Args) -> Value {
    let sub = a.words.first().copied().unwrap_or("list");
    let agent = a.words.get(1).copied().unwrap_or("claude");
    if sub != "list" && sub != "doctor" && agent != "claude" {
        return json!({"ok": false, "error": format!("{agent}: not supported yet (Claude Code only until P6)")});
    }
    match sub {
        "list" | "doctor" => {
            let claude_bin = which("claude");
            let installed = claude_bin
                .as_ref()
                .map(|_| claude(&["plugin", "list", "--json"]));
            let ours = installed
                .as_ref()
                .and_then(|r| r.get("result"))
                .map(|r| r.to_string().contains("ewe-companion"))
                .unwrap_or(false);
            json!({
                "ok": true,
                "agents": [{
                    "agent": "claude",
                    "found": claude_bin.is_some(),
                    "hooks": ours,
                    "relay": which("ewe-companion").map(|p| p.display().to_string()),
                    "integration": integration_dir().map(|p| p.display().to_string()),
                    "note": "The hooks only ever ask you; with the companion off, Claude asks in its terminal.",
                }],
                "daemon": ask("status", json!({})).get("ok").cloned().unwrap_or(Value::Bool(false)),
            })
        }
        "install" => {
            if which("claude").is_none() {
                return json!({"ok": false, "error": "claude is not installed"});
            }
            let Some(dir) = integration_dir() else {
                return json!({"ok": false, "error": "the Claude Code integration is missing (ewe-companion package)"});
            };
            let dir = dir.display().to_string();
            let add = claude(&["plugin", "marketplace", "add", &dir, "--json"]);
            let install = claude(&["plugin", "install", PLUGIN, "--json"]);
            let ok = install["ok"] == Value::Bool(true);
            json!({"ok": ok, "steps": [add, install],
                   "note": "Restart running Claude Code sessions to load the hooks."})
        }
        "usage" => match a.words.get(2).copied() {
            Some("on") => usage_on(),
            Some("off") => usage_off(),
            _ => json!({"ok": false, "error": "agents usage claude on|off"}),
        },
        "uninstall" => {
            let rm = claude(&["plugin", "uninstall", PLUGIN, "--json"]);
            let mk = claude(&["plugin", "marketplace", "remove", MARKETPLACE]);
            json!({"ok": rm["ok"] == Value::Bool(true), "steps": [rm, mk]})
        }
        other => {
            json!({"ok": false, "error": format!("agents {other}: use list, install, uninstall or doctor")})
        }
    }
}

/// `key set <provider>`: the key on stdin, never on the command line (it
/// would sit in your shell history and in `ps`).
fn key(a: &Args) -> Value {
    match (a.words.first().copied(), a.words.get(1).copied()) {
        (Some("set"), Some(provider)) => {
            let mut key = String::new();
            if std::io::Read::read_to_string(
                &mut std::io::Read::take(std::io::stdin(), 8192),
                &mut key,
            )
            .is_err()
                || key.trim().is_empty()
            {
                return json!({"ok": false, "error": "pipe the key on stdin: ewe-companion key set anthropic < keyfile"});
            }
            ask("key.set", json!({"provider": provider, "key": key.trim()}))
        }
        (Some("clear"), Some(provider)) => ask("key.clear", json!({"provider": provider})),
        _ => {
            json!({"ok": false, "error": "key set <provider> (key on stdin) | key clear <provider>"})
        }
    }
}

// ── plan usage: the status-line swap ─────────────────────────────────────

const OUR_STATUSLINE: &str = "ewe-companion statusline";

fn claude_settings() -> PathBuf {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".claude")
        })
        .join("settings.json")
}

fn write_json(path: &std::path::Path, v: &Value) -> std::io::Result<()> {
    let tmp = path.with_extension("json.ewe-companion.tmp");
    std::fs::write(
        &tmp,
        serde_json::to_string_pretty(v).unwrap_or_default() + "\n",
    )?;
    std::fs::rename(tmp, path)
}

/// Put the relay in front of your status line. Your previous one is saved
/// and keeps running behind it; the settings file is backed up first.
fn usage_on() -> Value {
    let path = claude_settings();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| "{}".into());
    let Ok(mut v) = serde_json::from_str::<Value>(&text) else {
        return json!({"ok": false, "error": format!("{} is not valid JSON; not touching it", path.display())});
    };
    let current = v.get("statusLine").cloned().unwrap_or(Value::Null);
    if current.get("command").and_then(Value::as_str) == Some(OUR_STATUSLINE) {
        return json!({"ok": true, "already": true});
    }
    let state = companion_core::paths::state_dir();
    let backups = state.join("backups");
    let stamp = companion_core::id::now_s();
    let backup = backups.join(format!("settings.json.{stamp}"));
    if std::fs::create_dir_all(&backups).is_err() || std::fs::write(&backup, &text).is_err() {
        return json!({"ok": false, "error": "could not back up the settings file"});
    }
    if write_json(
        &state.join(crate::statusline::PREVIOUS),
        &json!({"statusLine": current, "saved": stamp}),
    )
    .is_err()
    {
        return json!({"ok": false, "error": "could not save the previous status line"});
    }
    let mut ours = if current.is_object() {
        current.clone()
    } else {
        json!({})
    };
    ours["type"] = json!("command");
    ours["command"] = json!(OUR_STATUSLINE);
    v["statusLine"] = ours;
    match write_json(&path, &v) {
        Ok(()) => json!({"ok": true, "backup": backup.display().to_string(), "previous": current,
                          "note": "Your status line is unchanged; plan usage shows in the island after the next message."}),
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

fn usage_off() -> Value {
    let path = claude_settings();
    let prev_file = companion_core::paths::state_dir().join(crate::statusline::PREVIOUS);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return json!({"ok": false, "error": "no Claude settings file"});
    };
    let Ok(mut v) = serde_json::from_str::<Value>(&text) else {
        return json!({"ok": false, "error": "the settings file is not valid JSON"});
    };
    if v.get("statusLine")
        .and_then(|s| s.get("command"))
        .and_then(Value::as_str)
        != Some(OUR_STATUSLINE)
    {
        return json!({"ok": true, "already": true});
    }
    let prev = std::fs::read_to_string(&prev_file)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|p| p.get("statusLine").cloned())
        .unwrap_or(Value::Null);
    match v.as_object_mut() {
        Some(o) if prev.is_null() => {
            o.remove("statusLine");
        }
        Some(o) => {
            o.insert("statusLine".into(), prev.clone());
        }
        None => return json!({"ok": false, "error": "unexpected settings shape"}),
    }
    match write_json(&path, &v) {
        Ok(()) => {
            let _ = std::fs::remove_file(prev_file);
            json!({"ok": true, "restored": prev})
        }
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}
