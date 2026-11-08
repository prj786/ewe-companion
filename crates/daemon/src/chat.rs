//! One chat line in, one answer out (the island's chat box, `ewe-companion ask`).
//!
//! 1. The offline matcher first: a plain command ("volume 60") runs at once,
//!    with or without a model (C5). Two or three plain commands in one line
//!    ("dnd on and a 25 minute timer") run one after the other, with one
//!    Undo for all of them.
//! 2. Otherwise the chosen provider answers, told what the companion knows
//!    about ewe (the knowledge pack's primer, in the system prompt) and,
//!    with the question, the topics that fit it and what this computer is
//!    like now ([`crate::context`]: the time, memory, load, battery …). A
//!    reply with `ACTION` lines runs those commands (up to three). A reply
//!    made only of *looks* (reads safe to show a model: the time, what uses
//!    the CPU, a sum …) is a look-up: they run, and the model is asked
//!    again with their results, so it answers from facts. Anything else is
//!    shown as the answer.
//! 3. With no provider, a question the knowledge pack answers confidently
//!    gets that answer; anything else says how to pick a provider. Either
//!    way commands work.
//!
//! An answer streams: the island gets `chat.reply` `partial` lines with
//! the text so far (never an ACTION line) while the model writes, when
//! every island says it draws them (`hello` `caps`). And the chat field's
//! focus (`chat.warm`) gets Claude Code started before the question is
//! typed ([`warm`]).
//!
//! An answer from a topic with an `open:` line carries it as a button
//! ("Open Settings"), so the user is one click from the place it names.
//!
//! A shell command is never run from here. A model's `run_shell`, a topic's
//! `run:` line or the user's own `$ command` becomes a *proposal*: the reply
//! carries `confirm` (the exact command, its risk), the island shows Run and
//! In terminal, and [`confirm`] runs it on that click (the owner,
//! 2026-10-14: the companion should run what it suggests). The check in
//! front of every proposal is `companion_core::shell::assess`.
//!
//! The chat is one running conversation of the last few turns, in memory
//! (the store keeps history later, P4). The turn is the user's own words,
//! so a command it asks for runs without a confirm card (T1, decision 6).
//!
//! What a command found (Safe's logins, files, the browser's pages) shows
//! on the island as rows; the conversation keeps only the outcome's
//! `private` line ("done", a count), so a later question to a model never
//! carries a name (TRIO.md: values never cross, names stay local). A Safe
//! request's own words are kept out too.
//!
//! "What is this page about?" reads the page open in ewe browser and asks
//! the model with the page's text wrapped in `<untrusted source="web
//! page">`. That turn is tainted (C4): an ACTION in its reply does not run;
//! it comes back as a button, with the reason. A look-up's results come
//! back wrapped the same way (a program's name is another program's
//! words), so an ACTION in the answer that follows is a button too.

use std::sync::Arc;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use companion_core::commands::Call;
use companion_core::id::new_id;
use companion_core::knowledge::{self, Pack};
use companion_core::shell::{self, Verdict};
use companion_core::{intent, persona, proto};
use serde_json::{Value, json};
use tracing::{info, warn};

use crate::hub::{Hub, Proposal, Undo};
use crate::providers::Sink;
use crate::{browser, commands, context, providers, runner};

#[derive(Default)]
pub struct Reply {
    /// `done` (a command ran) · `answer` · `error` · `info`
    pub kind: &'static str,
    pub text: String,
    pub undo: Option<String>,
    pub command: Option<String>,
    /// a button under the answer: its label and the `open` it runs
    pub action: Option<(String, Call)>,
    /// a proposed shell command waiting for its Run click:
    /// `{token, command, about, risk, reason, terminal}`
    pub confirm: Option<Value>,
    /// what a command printed, its last lines
    pub output: Option<String>,
    /// what a command found, as rows the island draws
    pub rows: Vec<Value>,
    /// what the conversation keeps instead of `text` (names stay local)
    pub private: Option<String>,
}

impl Reply {
    pub fn info(text: impl Into<String>) -> Reply {
        Reply {
            kind: "info",
            text: text.into(),
            ..Default::default()
        }
    }

    fn error(text: impl Into<String>) -> Reply {
        Reply {
            kind: "error",
            text: text.into(),
            ..Default::default()
        }
    }
}

/// Offer a shell command: checked, kept under a token, shown with Run. A
/// refused one is said in words and never offered.
fn propose(hub: &Arc<Hub>, command: &str, about: &str, terminal: bool) -> Result<Value, String> {
    let command = command.trim();
    let a = shell::assess(command);
    let (risk, reason) = match a.verdict {
        Verdict::Refused(why) => return Err(format!("I won't run `{command}`. {why}")),
        Verdict::Risky(why) => (Some("danger"), why),
        Verdict::Fine => (None, ""),
    };
    let terminal = terminal || a.terminal;
    let about = about.trim().to_string();
    let token = hub.keep_proposal(Proposal {
        command: command.to_string(),
        about: about.clone(),
        terminal,
        made: std::time::Instant::now(),
    });
    info!(terminal, risky = risk.is_some(), "proposal");
    Ok(json!({"token": token, "command": command, "about": about,
              "risk": risk, "reason": reason, "terminal": terminal}))
}

fn proposal_reply(
    hub: &Arc<Hub>,
    command: &str,
    about: &str,
    terminal: bool,
    text: String,
) -> Reply {
    match propose(hub, command, about, terminal) {
        Ok(c) => Reply {
            kind: "answer",
            text,
            confirm: Some(c),
            ..Default::default()
        },
        Err(why) => Reply::error(why),
    }
}

/// The Run click (`confirm.run`): run the proposal, or open it in a
/// terminal, once. What it printed goes back to the chat and into the
/// conversation, so a follow-up question can use it.
pub async fn confirm(hub: &Arc<Hub>, token: &str, mode: &str) -> Reply {
    let Some(p) = hub.take_proposal(token) else {
        return Reply::info(
            "That command is no longer waiting. Ask again and I'll offer it again.",
        );
    };
    if mode == "terminal" || p.terminal {
        return match hub.shell_do("term", json!({"command": p.command})).await {
            Ok(_) => {
                hub.chat_push(
                    &format!("(opened `{}` in a terminal)", p.command),
                    "It runs in the terminal.",
                );
                Reply {
                    kind: "done",
                    text: "Opened it in a terminal".into(),
                    ..Default::default()
                }
            }
            Err(e) => Reply::error(e),
        };
    }
    let ran = runner::run(hub, &p.command).await;
    info!(ok = ran.ok, code = ?ran.code, timed_out = ran.timed_out, "ran a proposal");
    // the command shows above, in its own box: the line only says how it went
    let what = if p.about.is_empty() {
        String::new()
    } else {
        format!(": {}", p.about)
    };
    let text = if ran.timed_out {
        "Stopped after two minutes. A long job belongs in a terminal: ask again and press In terminal.".to_string()
    } else if ran.ok {
        format!("Done{what}")
    } else {
        match ran.code {
            Some(c) => format!("It did not work (exit {c}){what}"),
            None => format!("It was stopped{what}"),
        }
    };
    hub.chat_push(
        &format!("(ran `{}`)", p.command),
        &format!("{text}\n{}", ran.output),
    );
    Reply {
        kind: if ran.ok { "done" } else { "error" },
        text,
        output: (!ran.output.is_empty()).then_some(ran.output),
        ..Default::default()
    }
}

/// The knowledge pack: the one built in, or `$EWE_COMPANION_KNOWLEDGE`
/// (a directory of `.md` files, for whoever edits the pack).
pub fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    P.get_or_init(|| {
        let Some(dir) = std::env::var_os("EWE_COMPANION_KNOWLEDGE") else {
            return Pack::built_in();
        };
        let mut files: Vec<(String, String)> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                (p.extension()? == "md").then_some(())?;
                Some((
                    p.file_stem()?.to_str()?.to_string(),
                    std::fs::read_to_string(&p).ok()?,
                ))
            })
            .collect();
        files.sort();
        if files.is_empty() {
            warn!("EWE_COMPANION_KNOWLEDGE has no .md files; using the built-in pack");
            return Pack::built_in();
        }
        Pack::from_files(files.iter().map(|(n, t)| (n.as_str(), t.as_str())))
    })
}

fn hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "this computer".into())
}

/// The system prompt: the same for every question (so a provider caches
/// it and a warm `claude` starts with it).
fn system() -> String {
    persona::system_prompt(&persona::Brief {
        host: &hostname(),
        primer: &pack().primer,
    })
}

/// The chat field got focus (`chat.warm`), or an answer just went out: a
/// question is likely, so start Claude Code now (it takes ~4 s) and read
/// the slow part of "this computer now" ahead of it.
pub async fn warm(hub: Arc<Hub>) {
    let settings = hub.chat_settings();
    if providers::info(&settings.provider).is_none() {
        return;
    }
    let system = system();
    tokio::join!(providers::warm(&settings, &system), context::prefetch());
}

async fn act(hub: &Arc<Hub>, call: &companion_core::commands::Call) -> Reply {
    let out = commands::run(hub, call).await;
    info!(command = %call.command, ok = out.ok, "command");
    reply_of(hub, call, out)
}

/// A command's outcome as the island shows it: its line, Undo, the rows
/// it found, a button, or a command to confirm first.
pub fn reply_of(hub: &Arc<Hub>, call: &Call, out: commands::Outcome) -> Reply {
    let undo = match (out.undo, out.undo_browser.is_empty()) {
        (Some(u), _) => Some(hub.keep_undo(Undo::Call(u))),
        (None, false) => Some(hub.keep_undo(Undo::Browser(out.undo_browser))),
        (None, true) => None,
    };
    let confirm = out
        .propose
        .and_then(|(cmd, about)| propose(hub, &cmd, &about, false).ok());
    Reply {
        kind: match (out.ok, out.rows.is_empty() && confirm.is_none()) {
            (false, _) => "error",
            (true, true) => "done",
            (true, false) => "answer",
        },
        text: out.line,
        undo,
        command: Some(call.command.clone()),
        action: out.action,
        confirm,
        rows: out.rows,
        private: out.private,
        ..Default::default()
    }
}

/// Several replies as one: their lines in order, one Undo that puts all
/// of them back, the first button and the first proposal.
fn combine(hub: &Arc<Hub>, replies: Vec<Reply>) -> Reply {
    let mut undos = Vec::new();
    let mut out = Reply {
        kind: if replies.iter().all(|r| r.kind == "error") {
            "error"
        } else if replies.iter().all(|r| r.kind == "done") {
            "done"
        } else {
            "answer"
        },
        ..Default::default()
    };
    let mut lines = Vec::new();
    let mut kept = Vec::new();
    for r in replies {
        if let Some(u) = r.undo.as_deref().and_then(|t| hub.take_undo(t)) {
            undos.push(u);
        }
        kept.push(r.private.clone().unwrap_or_else(|| r.text.clone()));
        lines.push(r.text);
        out.command = out.command.or(r.command);
        out.action = out.action.or(r.action);
        out.confirm = out.confirm.or(r.confirm);
        out.rows.extend(r.rows);
    }
    out.text = lines.join("\n");
    out.private = (kept != lines).then(|| kept.join("\n"));
    out.undo = match undos.len() {
        0 => None,
        1 => Some(hub.keep_undo(undos.remove(0))),
        _ => Some(hub.keep_undo(Undo::Many(undos))),
    };
    out
}

/// The Undo click: the call a reply kept, ewe browser's own steps, or
/// several of those in reverse order.
pub async fn undo(hub: &Arc<Hub>, token: &str) -> Reply {
    match hub.take_undo(token) {
        Some(u) => undo_one(hub, u).await,
        None => Reply::info("Nothing to undo."),
    }
}

async fn undo_one(hub: &Arc<Hub>, u: Undo) -> Reply {
    match u {
        Undo::Call(call) => {
            let out = commands::run(hub, &call).await;
            Reply {
                kind: if out.ok { "done" } else { "error" },
                text: out.line,
                command: Some(call.command),
                ..Default::default()
            }
        }
        // ewe browser's own undo steps
        Undo::Browser(steps) => {
            let out = browser::undo(hub, &steps).await;
            Reply {
                kind: if out.ok { "done" } else { "error" },
                text: out.line,
                ..Default::default()
            }
        }
        Undo::Many(all) => {
            let mut replies = Vec::new();
            for u in all.into_iter().rev() {
                replies.push(Box::pin(undo_one(hub, u)).await);
            }
            combine(hub, replies)
        }
    }
}

/// Keep a turn for the next question. A reply with a `private` line is
/// kept as that line, and a Safe request's own words stay out too.
fn remember(hub: &Arc<Hub>, text: &str, r: &Reply) {
    let safe = r
        .command
        .as_deref()
        .is_some_and(companion_core::commands::is_safe);
    let user = if safe {
        "(a request to ewe Safe)"
    } else {
        text
    };
    hub.chat_push(user, r.private.as_deref().unwrap_or(&r.text));
}

/// A model's ACTION: a proposal for `run_shell`, else the command itself.
async fn act_on(hub: &Arc<Hub>, call: &Call, reply: &str) -> Reply {
    if call.command != "run_shell" {
        return act(hub, call).await;
    }
    let a = &call.args;
    let command = a["command"].as_str().unwrap_or("");
    let about = a["about"].as_str().unwrap_or("");
    // whatever the model said around its ACTION line is the reply's text
    let text = match (persona::words_of(reply).as_str(), about.trim()) {
        ("", "") => "This command does it:".to_string(),
        ("", ab) => format!("{ab}:"),
        (w, _) => w.to_string(),
    };
    proposal_reply(
        hub,
        command,
        about,
        a["terminal"].as_bool().unwrap_or(false),
        text,
    )
}

/// "dnd on and a 25 minute timer": the parts of a line that names two or
/// three things to do, or `None`. Only the matcher decides whether each
/// part is a command; this only cuts.
fn split_compound(text: &str) -> Option<Vec<String>> {
    let mut parts = vec![text.trim().to_lowercase()];
    for sep in [
        ", and then ",
        " and then ",
        ", then ",
        " then ",
        ", and ",
        " and ",
        "; ",
    ] {
        parts = parts
            .iter()
            .flat_map(|p| p.split(sep).map(str::to_string).collect::<Vec<_>>())
            .collect();
    }
    let parts: Vec<String> = parts
        .into_iter()
        .map(|p| p.trim().trim_matches(',').trim().to_string())
        .collect();
    (parts.len() >= 2 && parts.len() <= 3 && parts.iter().all(|p| !p.is_empty())).then_some(parts)
}

/// Two or three plain commands in one line, when every part is one.
async fn compound(hub: &Arc<Hub>, text: &str) -> Option<Reply> {
    let parts = split_compound(text)?;
    let calls: Vec<Call> = parts
        .iter()
        .map(|p| intent::match_text(p))
        .collect::<Option<_>>()?;
    let mut replies = Vec::new();
    let mut missed = 0;
    for call in &calls {
        let out = commands::run(hub, call).await;
        info!(command = %call.command, ok = out.ok, miss = out.miss, "command (one of several)");
        missed += usize::from(out.miss);
        replies.push(reply_of(hub, call, out));
    }
    // every part named nothing here: the words were a question after all
    (missed < calls.len()).then(|| combine(hub, replies))
}

/// The answer so far, sent to the island while it streams in: never an
/// ACTION line, at most every 80 ms, only when every island draws it.
fn partial_sink(hub: &Arc<Hub>, id: &str) -> impl Fn(&str) + Send + Sync + use<> {
    let hub = hub.clone();
    let id = id.to_string();
    let last: Mutex<(Option<Instant>, String)> = Mutex::new((None, String::new()));
    move |so_far: &str| {
        if !hub.streams() {
            return;
        }
        let shown = persona::visible(so_far);
        if shown.is_empty() {
            return;
        }
        let Ok(mut g) = last.lock() else { return };
        if g.1 == shown || g.0.is_some_and(|t| t.elapsed() < Duration::from_millis(80)) {
            return;
        }
        *g = (Some(Instant::now()), shown.clone());
        hub.broadcast(proto::line(
            "chat.reply",
            json!({"id": id, "kind": "partial", "text": shown}),
        ));
    }
}

pub async fn answer(hub: &Arc<Hub>, text: &str) -> Reply {
    answer_with(hub, text, None).await
}

pub async fn answer_with(hub: &Arc<Hub>, text: &str, sink: Option<&Sink>) -> Reply {
    let text = text.trim();
    if text.is_empty() {
        return Reply::info("Ask something, or tell me what to do.");
    }
    // the user's own command: `$ ls ~/Downloads`
    if let Some(cmd) = text.strip_prefix("$ ").or_else(|| text.strip_prefix('$')) {
        let r = proposal_reply(hub, cmd, "", false, "Run it?".into());
        return r;
    }
    // a plain command runs at once. One whose words named nothing here
    // ("open the pod bay doors": no such app) is a question after all; its
    // line is kept in case nothing else has an answer either
    let mut missed: Option<String> = None;
    if intent::about_page(text) {
        return about_page(hub, text, sink).await;
    }
    if let Some(call) = intent::match_text(text) {
        let out = commands::run(hub, &call).await;
        info!(command = %call.command, ok = out.ok, miss = out.miss, "command");
        if !out.miss {
            let r = reply_of(hub, &call, out);
            remember(hub, text, &r);
            return r;
        }
        missed = Some(out.line);
    }
    // several plain commands in one line
    if let Some(r) = compound(hub, text).await {
        remember(hub, text, &r);
        return r;
    }

    let pack = pack();
    let hits = pack.search(text, 3);
    let sure = hits.first().filter(|h| h.score >= knowledge::CONFIDENT);
    let offer = sure.and_then(|h| Some((h.topic.open_label()?, h.topic.open.clone()?)));
    // the topic's own command, offered with Run (never run without it)
    let offer_run = |hub: &Arc<Hub>| {
        let h = sure?;
        propose(hub, h.topic.run.as_deref()?, &h.topic.title, false).ok()
    };
    let from_pack = |note: Option<&str>| {
        let h = sure.expect("only called with a sure match");
        let mut line = h.topic.answer(text).to_string();
        if let Some(n) = note {
            line.push_str(&format!("\n({n})"));
        }
        Reply {
            kind: "answer",
            text: line,
            action: offer.clone(),
            confirm: offer_run(hub),
            ..Default::default()
        }
    };

    let settings = hub.chat_settings();
    if providers::info(&settings.provider).is_none() {
        // a command that named nothing here says so: it was plainly a
        // command, and a how-to would not answer it
        if let Some(line) = missed {
            return Reply {
                kind: "error",
                ..Reply::info(line)
            };
        }
        if sure.is_some() {
            let r = from_pack(None);
            hub.chat_push(text, &r.text);
            return r;
        }
        return Reply::info(
            "I can do things like \"volume 60\", \"open nemo\" or \"game mode on\", and I know my way round ewe. For other questions, set Who answers questions in Komble → Plugins → Shaun.",
        );
    }

    let machine = context::snapshot(hub).await;
    // the topics worth a model's time: a weak match is noise
    let worth: Vec<_> = hits.iter().filter(|h| h.score >= 1.5).collect();
    let topics = knowledge::prompt_block(&worth);
    let ask = persona::with_notes(
        &persona::Turn {
            topics: &topics,
            machine: &machine,
            memory: "",
        },
        text,
    );
    let system = system();
    let history = hub.chat_history();
    let mut answered = true;
    let r = match providers::ask(&settings, &system, &history, &ask, sink).await {
        Ok(reply) => {
            let calls = persona::parse_actions(&reply);
            if calls.is_empty() {
                Reply {
                    kind: "answer",
                    text: reply.trim().to_string(),
                    action: offer,
                    confirm: offer_run(hub),
                    ..Default::default()
                }
            } else if calls.iter().all(|c| persona::is_look(&c.command)) {
                look_up(hub, &system, &history, &ask, &calls, sink).await
            } else if calls.len() == 1 {
                act_on(hub, &calls[0], &reply).await
            } else {
                let mut replies = Vec::new();
                for call in &calls {
                    replies.push(act_on(hub, call, &reply).await);
                }
                combine(hub, replies)
            }
        }
        // the provider failed, but the pack knows: answer, and say why the
        // provider did not
        Err(e) if sure.is_some() => {
            answered = false;
            from_pack(Some(&e))
        }
        Err(e) => {
            answered = false;
            Reply::error(e)
        }
    };
    if answered {
        remember(hub, text, &r);
    }
    // the next question finds Claude Code started
    tokio::spawn(warm(hub.clone()));
    r
}

/// What a look tells a model: its model-safe line (what the conversation
/// may keep), never rows.
fn look_line(call: &Call, out: &commands::Outcome) -> String {
    let what = out.private.as_deref().unwrap_or(&out.line);
    format!("{}: {}", call.command, what.trim())
}

/// A reply made only of looks: run them, then ask again with what they
/// found. That second answer is the reply; an ACTION in it is a button,
/// since it rests on what programs reported (C4).
async fn look_up(
    hub: &Arc<Hub>,
    system: &str,
    history: &[(String, String)],
    ask: &str,
    calls: &[Call],
    sink: Option<&Sink>,
) -> Reply {
    let mut found = Vec::new();
    for call in calls {
        let out = commands::run(hub, call).await;
        info!(command = %call.command, ok = out.ok, "look");
        found.push(look_line(call, &out));
    }
    let found = found.join("\n");
    let again = format!(
        "{ask}\n\nWhat you looked up:\n{}\nNow answer the question in words.",
        persona::untrusted("ewe", "what Shaun looked up", &found)
    );
    let settings = hub.chat_settings();
    let reply = match providers::ask(&settings, system, history, &again, sink).await {
        Ok(r) => r,
        // the facts are an answer of their own
        Err(_) => {
            return Reply {
                kind: "answer",
                text: found,
                ..Default::default()
            };
        }
    };
    let words = persona::words_of(&reply);
    let next = persona::parse_actions(&reply)
        .into_iter()
        .find(|c| !persona::is_look(&c.command));
    match next {
        Some(call) if call.command == "run_shell" => act_on(hub, &call, &reply).await,
        Some(call) => Reply {
            kind: "answer",
            text: if words.is_empty() {
                "Shall I?".into()
            } else {
                words
            },
            action: Some(("Do it".into(), call)),
            ..Default::default()
        },
        None => Reply {
            kind: "answer",
            text: if words.is_empty() { found } else { words },
            ..Default::default()
        },
    }
}

/// How much of a page goes to a model.
const PAGE_MAX: usize = 12_000;

/// "What is this page about?": the page open in ewe browser goes to the
/// model as untrusted data, and the turn is tainted: an ACTION in the
/// reply comes back as a button to click, never a run (C4).
async fn about_page(hub: &Arc<Hub>, text: &str, sink: Option<&Sink>) -> Reply {
    let settings = hub.chat_settings();
    if providers::info(&settings.provider).is_none() {
        return Reply::info(
            "Reading a page needs someone to answer: set Who answers questions in Komble → Plugins → Shaun.",
        );
    }
    let page = match browser::page(PAGE_MAX).await {
        Ok(p) => p,
        Err(e) => return Reply::error(e),
    };
    let about = format!("{} · {}", page.title, page.url);
    let ask = format!(
        "{text}\n\nThe page open in ewe Browser:\n{}",
        persona::untrusted("web page", &about, &page.text)
    );
    info!(chars = page.text.len(), "page question");
    let history = hub.chat_history();
    let r = match providers::ask(&settings, &system(), &history, &ask, sink).await {
        Ok(reply) => match persona::parse_action(&reply) {
            // a page asked for it, or seemed to: the user decides
            Some(call) if call.command == "run_shell" => act_on(hub, &call, &reply).await,
            Some(call) => {
                let why = "This request read a web page, so I'm asking first.";
                Reply {
                    kind: "answer",
                    text: match persona::words_of(&reply).as_str() {
                        "" => why.to_string(),
                        w => format!("{w}\n{why}"),
                    },
                    action: Some(("Do it".into(), call)),
                    ..Default::default()
                }
            }
            None => Reply {
                kind: "answer",
                text: reply.trim().to_string(),
                ..Default::default()
            },
        },
        Err(e) => Reply::error(e),
    };
    // the question and the answer; never the page itself
    if r.kind != "error" {
        hub.chat_push(text, &r.text);
    }
    r
}

/// The island's `chat.send`: "thinking", the answer growing (`partial`),
/// then the answer, all as `chat.reply` with the same id.
pub async fn send(hub: Arc<Hub>, text: String) {
    let id = new_id("m");
    hub.broadcast(proto::line(
        "chat.reply",
        json!({"id": id, "kind": "thinking"}),
    ));
    let sink = partial_sink(&hub, &id);
    let r = answer_with(&hub, &text, Some(&sink)).await;
    hub.broadcast(proto::line("chat.reply", reply_json(&id, &r)));
}

pub fn reply_json(id: &str, r: &Reply) -> Value {
    let action = r
        .action
        .as_ref()
        .map(|(label, call)| json!({"label": label, "command": call.command, "args": call.args}));
    let rows = (!r.rows.is_empty()).then_some(&r.rows);
    json!({"id": id, "kind": r.kind, "text": r.text, "undo": r.undo, "command": r.command,
           "action": action, "confirm": r.confirm, "output": r.output, "rows": rows})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compound_lines_split_into_two_or_three() {
        assert_eq!(
            split_compound("Turn on DND and set a timer for 25 minutes").unwrap(),
            ["turn on dnd", "set a timer for 25 minutes"]
        );
        assert_eq!(
            split_compound("mute, then lock the screen; open nemo").unwrap(),
            ["mute", "lock the screen", "open nemo"]
        );
        assert_eq!(
            split_compound("volume 30, and then dim the screen")
                .unwrap()
                .len(),
            2
        );
        // one thing, or too many, is not a compound
        assert_eq!(split_compound("search for salt"), None);
        assert_eq!(split_compound("a and b and c and d"), None);
        assert_eq!(split_compound("and"), None);
    }
}
