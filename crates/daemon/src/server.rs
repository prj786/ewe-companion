//! The socket: `$XDG_RUNTIME_DIR/ewe-companion.sock`, mode 0600, and every
//! peer checked with `SO_PEERCRED` (same uid or nothing).
//!
//! The first line of a connection says what it is:
//!
//! - `hello` from the island (or, later, the window): it stays, gets a
//!   `welcome` and the full `state`, and sends acks, answers and `env`;
//! - `agent` from a hook relay: folded into its session, and when it waits,
//!   held open until a human answers or a budget runs out;
//! - `cli` from a CLI verb: one reply, then closed.

use std::sync::Arc;
use std::time::Duration;

use companion_core::id::{new_id, now_ms};
use companion_core::proto::{self, AgentMsg, Answer, Card, Inbound, Peek, Reply};
use companion_core::{VERSION, claude, paths};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::oneshot;
use tracing::{debug, info, warn};

use crate::hub::{Hub, Wait};
use crate::term;

/// No `card.shown` within this → the island is not there: no opinion.
pub const ACK: Duration = Duration::from_millis(800);
/// How long a hook may wait for a human (the relay gives up at 110 s, the
/// hook's own timeout is 120 s).
pub const WAIT: Duration = Duration::from_secs(108);

type Reader = BufReader<OwnedReadHalf>;

/// One line, at most `MAX_LINE` bytes; `None` at EOF, on an error, or when
/// the line is too long (the connection is then dropped).
async fn read_line(rd: &mut Reader) -> Option<String> {
    let mut buf = Vec::new();
    let n = (&mut *rd)
        .take(proto::MAX_LINE as u64 + 1)
        .read_until(b'\n', &mut buf)
        .await
        .ok()?;
    if n == 0 || buf.len() > proto::MAX_LINE {
        return None;
    }
    String::from_utf8(buf)
        .ok()
        .map(|s| s.trim_end_matches(['\n', '\r']).to_string())
}

async fn write(wr: &mut OwnedWriteHalf, line: &str) -> bool {
    wr.write_all(line.as_bytes()).await.is_ok() && wr.flush().await.is_ok()
}

pub async fn handle(hub: Arc<Hub>, stream: UnixStream) {
    match stream.peer_cred() {
        Ok(c) if c.uid() == paths::uid() => {}
        Ok(c) => {
            warn!(uid = c.uid(), "refused a peer with another uid");
            return;
        }
        Err(_) => return,
    }
    let (rd, mut wr) = stream.into_split();
    let mut rd = BufReader::new(rd);
    let Some(first) = read_line(&mut rd).await else {
        return;
    };
    match proto::decode(&first) {
        Ok(Inbound::Hello { client, pid }) => {
            debug!(client, pid, "hello");
            island(hub, rd, wr).await
        }
        Ok(Inbound::Agent(msg)) => agent(hub, msg, rd, wr).await,
        Ok(Inbound::Cli { verb, args }) => {
            let reply = cli(&hub, &verb, &args).await;
            let mut line = reply.to_string();
            line.push('\n');
            write(&mut wr, &line).await;
        }
        Ok(other) => debug!(?other, "first line was not hello/agent/cli"),
        Err(proto::DecodeError::NewerProtocol(v)) => {
            let refuse = proto::line(
                "error",
                json!({"error": "newer-protocol", "theirs": v, "ours": proto::PROTOCOL}),
            );
            write(&mut wr, &refuse).await;
        }
        Err(e) => debug!(?e, "bad first line"),
    }
}

// ── the island ──────────────────────────────────────────────────────────

async fn island(hub: Arc<Hub>, mut rd: Reader, mut wr: OwnedWriteHalf) {
    let (id, mut rx) = hub.add_island();
    info!(island = id, "island connected");
    hub.set_glance(crate::glance::read());
    let hello = proto::line(
        "welcome",
        json!({"daemon": VERSION, "caps": ["cards", "peeks", "sessions", "jump"]}),
    );
    let state = proto::line("state", hub.snapshot());
    if !write(&mut wr, &hello).await || !write(&mut wr, &state).await {
        hub.remove_island(id);
        return;
    }
    let writer = tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if !write(&mut wr, &line).await {
                break;
            }
        }
    });
    while let Some(line) = read_line(&mut rd).await {
        match proto::decode(&line) {
            Ok(Inbound::CardShown { id }) => hub.card_shown(&id),
            Ok(Inbound::CardAnswer(a)) => hub.card_answer(a),
            Ok(Inbound::Env(env)) => hub.set_env(env),
            Ok(Inbound::Settings(chat)) => {
                let hub = hub.clone();
                tokio::spawn(async move { hub.set_chat_settings(chat).await });
            }
            Ok(Inbound::ChatSend { text }) => {
                tokio::spawn(crate::chat::send(
                    hub.clone(),
                    companion_core::trim::cut(&text, 4000),
                ));
            }
            Ok(Inbound::KeySet { provider, key }) => {
                let hub = hub.clone();
                tokio::spawn(async move {
                    let res = match crate::providers::info(&provider) {
                        Some(p) if p.cli.is_none() => crate::keyring::set(p.id, &key).await,
                        _ => Err("That provider takes no key.".into()),
                    };
                    let (kind, text) = match res {
                        Ok(()) => ("done", "Key saved in your keyring.".to_string()),
                        Err(e) => ("error", e),
                    };
                    info!(provider = %provider, ok = kind == "done", "api key stored");
                    hub.broadcast(proto::line(
                        "chat.reply",
                        json!({"id": new_id("m"), "kind": kind, "text": text}),
                    ));
                    hub.refresh_chat_status().await;
                });
            }
            Ok(Inbound::Undo { token }) => {
                let hub = hub.clone();
                tokio::spawn(async move {
                    let id = new_id("m");
                    let r = crate::chat::undo(&hub, &token).await;
                    hub.broadcast(proto::line("chat.reply", crate::chat::reply_json(&id, &r)));
                });
            }
            // a proposed command's Run / In terminal click
            Ok(Inbound::ConfirmRun { token, mode }) => {
                let hub = hub.clone();
                tokio::spawn(async move {
                    let id = new_id("m");
                    hub.broadcast(proto::line(
                        "chat.reply",
                        json!({"id": id, "kind": "thinking"}),
                    ));
                    let r = crate::chat::confirm(&hub, &token, &mode).await;
                    hub.broadcast(proto::line("chat.reply", crate::chat::reply_json(&id, &r)));
                });
            }
            Ok(Inbound::GlanceRefresh) => hub.set_glance(crate::glance::read()),
            Ok(Inbound::Run(call)) => {
                let hub = hub.clone();
                tokio::spawn(async move {
                    let id = new_id("m");
                    let out = crate::commands::run(&hub, &call).await;
                    info!(command = %call.command, ok = out.ok, "button");
                    let r = crate::chat::reply_of(&hub, &call, out);
                    hub.broadcast(proto::line("chat.reply", crate::chat::reply_json(&id, &r)));
                });
            }
            Ok(Inbound::DoResult {
                id,
                ok,
                error,
                data,
            }) => {
                let res = if ok {
                    Ok(data)
                } else {
                    Err(error.unwrap_or_else(|| "The island could not do that.".into()))
                };
                hub.do_result(&id, res);
            }
            Ok(Inbound::SessionJump { id }) => {
                let hub = hub.clone();
                tokio::spawn(async move {
                    if let Some(s) = hub.session(&id) {
                        term::jump(&s.term, &hub.env().hypr).await;
                    }
                });
            }
            Ok(_) => {}
            Err(e) => debug!(?e, "island sent a bad line"),
        }
    }
    writer.abort();
    hub.remove_island(id);
    info!(island = id, "island gone");
}

// ── a hook relay ────────────────────────────────────────────────────────

/// Resolves when the relay hangs up: Claude cancelled the hook because
/// the question was answered in the terminal.
async fn hung_up(rd: &mut Reader) {
    let mut buf = [0u8; 256];
    loop {
        match rd.read(&mut buf).await {
            Ok(0) | Err(_) => return,
            Ok(_) => continue,
        }
    }
}

async fn agent(hub: Arc<Hub>, msg: AgentMsg, mut rd: Reader, mut wr: OwnedWriteHalf) {
    if msg.agent != "claude" {
        return;
    }
    debug!(event = %msg.event, wait = msg.wait, "agent event");
    if let Some((sess, Some(line))) =
        hub.agent_event(&msg.agent, &msg.event, &msg.payload, &msg.term)
    {
        let failed = msg.event == "StopFailure";
        let hub2 = hub.clone();
        tokio::spawn(async move {
            // you are looking at it already: history only (PLAN §7)
            if !failed && term::is_focused(&sess.term, &hub2.env().hypr).await {
                return;
            }
            hub2.peek(Peek {
                key: format!("claude:{}:stop", sess.id),
                line,
                level: if failed { "danger" } else { "success" }.into(),
                actions: vec![proto::Action::new(
                    &format!("jump:{}", sess.id),
                    "Open terminal",
                    "ghost",
                )],
                ttl_ms: 0,
            });
        });
    }
    if !msg.wait || !claude::waits(&msg.event, &msg.payload) {
        return;
    }
    let reply = wait_for_human(&hub, &msg, &mut rd).await;
    let mut line =
        serde_json::to_string(&reply).unwrap_or_else(|_| r#"{"decision":"none"}"#.into());
    line.push('\n');
    write(&mut wr, &line).await;
}

async fn wait_for_human(hub: &Arc<Hub>, msg: &AgentMsg, rd: &mut Reader) -> Reply {
    let id = new_id("c");
    let started = tokio::time::Instant::now();
    let Some(card) = claude::card_for(
        &msg.event,
        &msg.payload,
        &id,
        now_ms(),
        hub.wait.as_millis() as u64,
    ) else {
        return Reply::none();
    };
    if hub.env().locked || hub.island_count() == 0 {
        return Reply::none();
    }
    let (ack_tx, ack_rx) = oneshot::channel();
    let (ans_tx, ans_rx) = oneshot::channel::<Answer>();
    hub.add_card(
        card,
        Some(Wait {
            ack: Some(ack_tx),
            answer: Some(ans_tx),
        }),
    );

    let gone = hung_up(rd);
    tokio::pin!(gone);
    tokio::select! {
        r = tokio::time::timeout(ACK, ack_rx) => {
            if !matches!(r, Ok(Ok(()))) {
                info!(card = %id, "no ack from the island");
                hub.close_card(&id, "no-ack", None);
                return Reply::none();
            }
        }
        _ = &mut gone => {
            hub.close_card(&id, "answered-elsewhere", None);
            return Reply::none();
        }
    }
    tokio::select! {
        r = ans_rx => match r {
            Ok(ans) => {
                hub.close_card(&id, "answered", Some(&ans.action));
                reply_for(hub, &ans, &msg.term).await
            }
            Err(_) => Reply::none(),
        },
        _ = tokio::time::sleep_until(started + hub.wait) => {
            hub.close_card(&id, "timeout", None);
            Reply::none()
        }
        _ = &mut gone => {
            hub.close_card(&id, "answered-elsewhere", None);
            Reply::none()
        }
    }
}

/// A click, in the relay's terms. Only `allow`, `always:<n>` and a
/// question's `answer` become an allow; everything else is deny or none.
async fn reply_for(hub: &Arc<Hub>, ans: &Answer, term_id: &Value) -> Reply {
    let mut r = Reply::none();
    match ans.action.as_str() {
        "allow" => r.decision = "allow".into(),
        "deny" => r.decision = "deny".into(),
        "answer" if ans.answers.is_some() => {
            r.decision = "allow".into();
            r.answers = ans.answers.clone();
        }
        "terminal" => {
            let (t, hypr) = (term_id.clone(), hub.env().hypr);
            tokio::spawn(async move { term::jump(&t, &hypr).await });
        }
        a => {
            if let Some(n) = a
                .strip_prefix("always:")
                .and_then(|n| n.parse::<usize>().ok())
            {
                r.decision = "allow".into();
                r.always = Some(n);
            }
        }
    }
    r
}

// ── CLI verbs ───────────────────────────────────────────────────────────

fn arg<'a>(args: &'a Value, k: &str) -> &'a str {
    args.get(k).and_then(Value::as_str).unwrap_or("")
}

async fn cli(hub: &Arc<Hub>, verb: &str, args: &Value) -> Value {
    match verb {
        "status" => hub.status(),
        "say" => {
            let text = arg(args, "text").trim();
            if text.is_empty() {
                return json!({"ok": false, "error": "say needs some text"});
            }
            let level = match arg(args, "level") {
                l @ ("info" | "success" | "warning" | "danger") => l,
                _ => "info",
            };
            let ttl = args
                .get("ttl_s")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                .min(60)
                * 1000;
            let source = Some(arg(args, "source"))
                .filter(|s| !s.is_empty())
                .unwrap_or("cli");
            let shown = hub.say(
                source,
                &companion_core::trim::cut(text, 300),
                level,
                arg(args, "key"),
                ttl,
            );
            json!({"ok": true, "shown": shown})
        }
        "history" => {
            let limit = args.get("limit").and_then(Value::as_u64).unwrap_or(20);
            json!({"ok": true, "events": hub.store().history(limit)})
        }
        "sessions" => json!({"ok": true, "sessions": hub.sessions()}),
        "ask" => {
            let r = crate::chat::answer(hub, arg(args, "text")).await;
            let mut v = crate::chat::reply_json("", &r);
            v["ok"] = Value::Bool(r.kind != "error");
            v
        }
        "key.set" => {
            let provider = arg(args, "provider");
            let res = match crate::providers::info(provider) {
                Some(p) if p.cli.is_none() => crate::keyring::set(p.id, arg(args, "key")).await,
                Some(_) => Err("That provider uses its own login, not a key.".into()),
                None => Err(format!("No provider called {provider}.")),
            };
            hub.refresh_chat_status().await;
            match res {
                Ok(()) => json!({"ok": true, "provider": provider}),
                Err(e) => json!({"ok": false, "error": e}),
            }
        }
        "key.clear" => {
            let ok = crate::keyring::clear(arg(args, "provider")).await;
            hub.refresh_chat_status().await;
            json!({"ok": ok})
        }
        // the status-line relay: Claude's plan limits, nothing else
        "usage" => {
            hub.set_usage(
                arg(args, "agent"),
                args.get("rate_limits").unwrap_or(&Value::Null),
            );
            json!({"ok": true})
        }
        "settings" => {
            if let Ok(s) =
                serde_json::from_value::<companion_core::proto::ChatSettings>(args.clone())
            {
                hub.set_chat_settings(s).await;
            }
            json!({"ok": true, "chat": hub.snapshot()["chat"]})
        }
        // the finder from a terminal: what it finds, as paths
        "find" => {
            let mut q = companion_core::find::parse(arg(args, "query"));
            if arg(args, "kind") == "folder" {
                q.kind = companion_core::find::Kind::Folder;
            }
            if q.is_empty() {
                return json!({"ok": false, "error": "find needs words to look for"});
            }
            let limit = args
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(20)
                .min(200);
            let (hits, partial) = crate::finder::find(q, limit as usize).await;
            let results: Vec<Value> = hits
                .iter()
                .map(|h| {
                    json!({"path": h.path, "kind": if h.dir { "folder" } else { "file" },
                           "modified": h.modified_s()})
                })
                .collect();
            json!({"ok": true, "results": results, "partial": partial})
        }
        "jump" => match hub.session(arg(args, "session")) {
            Some(s) => term::jump(&s.term, &hub.env().hypr).await,
            None => json!({"ok": false, "error": "no such session"}),
        },
        "debug.card" if hub.debug => {
            let card = debug_card(arg(args, "kind"));
            let id = card.id.clone();
            hub.add_card(card, None);
            json!({"ok": true, "id": id})
        }
        "debug.answer" if hub.debug => {
            let answers = args.get("answers").filter(|a| !a.is_null()).cloned();
            hub.card_answer(Answer {
                id: arg(args, "id").into(),
                action: arg(args, "action").into(),
                answers,
            });
            json!({"ok": true})
        }
        // a stand-in for the Run click, so tests can run a proposal
        "debug.confirm" if hub.debug => {
            let r = crate::chat::confirm(hub, arg(args, "token"), arg(args, "mode")).await;
            let mut v = crate::chat::reply_json("", &r);
            v["ok"] = Value::Bool(r.kind != "error");
            v
        }
        // a stand-in for the Undo click
        "debug.undo" if hub.debug => {
            let r = crate::chat::undo(hub, arg(args, "token")).await;
            let mut v = crate::chat::reply_json("", &r);
            v["ok"] = Value::Bool(r.kind != "error");
            v
        }
        // a stand-in for a button's click (`command.run`): a row's Copy,
        // Open, Unlock Safe …
        "debug.run" if hub.debug => {
            let call = companion_core::commands::Call::new(
                arg(args, "command"),
                args.get("args").cloned().unwrap_or(json!({})),
            );
            let out = crate::commands::run(hub, &call).await;
            let r = crate::chat::reply_of(hub, &call, out);
            let mut v = crate::chat::reply_json("", &r);
            v["ok"] = Value::Bool(r.kind != "error");
            v
        }
        v if v.starts_with("debug.") => {
            json!({"ok": false, "error": "debug verbs need a daemon started with EWE_COMPANION_DEBUG=1"})
        }
        other => json!({"ok": false, "error": format!("unknown verb: {other}")}),
    }
}

/// The fake cards the island smoke draws (PLAN §19).
fn debug_card(kind: &str) -> Card {
    let id = new_id("d");
    let now = now_ms();
    let payload = match kind {
        "question" => json!({
            "session_id": "debug", "cwd": "/home/you/Projects/ewe-browser", "tool_name": "AskUserQuestion",
            "tool_input": {"questions": [{"question": "Which test runner?", "header": "Tests", "multiSelect": false,
                "options": [{"label": "Vitest", "description": "Fast, Vite-native"},
                            {"label": "Jest", "description": "The classic"},
                            {"label": "Node test", "description": "Built in, no deps"}]}]}
        }),
        "note" => {
            return Card {
                id,
                kind: "note".into(),
                source: "companion".into(),
                title: "Download done".into(),
                body: "archlinux-2026.10.01-x86_64.iso · 1.2 GB".into(),
                actions: vec![
                    proto::Action::new("open", "Open", "primary"),
                    proto::Action::new("show", "Show in folder", "secondary"),
                ],
                created_ms: now,
                ..Default::default()
            };
        }
        _ => json!({
            "session_id": "debug", "cwd": "/home/you/Projects/ewe-browser", "tool_name": "Bash",
            "tool_input": {"command": "npm install", "description": "Install dependencies"},
            "permission_suggestions": [{"type": "addRules", "rules": [{"toolName": "Bash", "ruleContent": "npm install:*"}],
                                        "behavior": "allow", "destination": "localSettings"}]
        }),
    };
    let event = if kind == "question" {
        "PreToolUse"
    } else {
        "PermissionRequest"
    };
    let mut c = claude::card_for(event, &payload, &id, now, 0).unwrap_or_default();
    c.deadline_ms = 0;
    c.session = None;
    c
}
