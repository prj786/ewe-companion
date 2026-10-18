//! Safe, ewe's password manager, through its command line (ewe-safe
//! `docs/PROTOCOL.md`).
//!
//! - **Values never cross.** The companion asks Safe to *copy*; Safe runs
//!   `wl-copy` itself and clears the clipboard later. Only `summary`,
//!   `match`, `copy`, `lock` and `unlock --ask` are called, never `show`
//!   or `reveal`, so no reply here holds a password, a code or a note.
//! - **Names stay local.** A login's title and username are shown on the
//!   island only. Every outcome here carries `private`: what the
//!   conversation (and so a model) keeps instead, "done" or a count.
//!   Nothing here is stored, and nothing here is logged.
//! - **Safe acts.** A locked safe offers *Unlock Safe*, which opens Safe's
//!   own prompt (`ewe-safe unlock --ask`, run by the island so its window
//!   belongs to the desktop); the master password is never typed into the
//!   companion.
//!
//! `EWE_COMPANION_SAFE` names another binary (the tests' fake).

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use companion_core::commands::{Call, SAFE_FIELDS};
use companion_core::id::new_id;
use companion_core::proto;
use serde_json::{Value, json};
use tokio::process::Command;

use crate::commands::{Outcome, done, failed, missed};
use crate::hub::Hub;
use crate::{browser, chat};

const BUDGET: Duration = Duration::from_secs(8);
/// How long the companion waits for the unlock prompt to be answered.
const UNLOCK_WAIT: Duration = Duration::from_secs(180);
const ROWS: usize = 8;

/// `$EWE_COMPANION_SAFE`, else `ewe-safe` on PATH or in ~/.local/bin.
pub fn bin() -> Option<PathBuf> {
    match std::env::var_os("EWE_COMPANION_SAFE") {
        Some(p) if !p.is_empty() => {
            let p = PathBuf::from(p);
            p.is_file().then_some(p)
        }
        _ => crate::context::tool("ewe-safe"),
    }
}

/// Safe's failure: its code and its sentence.
struct Fail {
    code: String,
    message: String,
}

fn fail(code: &str, message: &str) -> Fail {
    Fail {
        code: code.into(),
        message: message.into(),
    }
}

/// One verb, one JSON object back. Safe's own copy needs the desktop's
/// Wayland display, which a user unit may not have: the island's.
async fn call(hub: &Hub, args: &[&str]) -> Result<Value, Fail> {
    let Some(bin) = bin() else {
        return Err(fail("not-installed", "Safe is not installed."));
    };
    let mut c = Command::new(bin);
    c.args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let wd = hub.env().wayland;
    if !wd.is_empty() {
        c.env("WAYLAND_DISPLAY", wd);
    }
    let out = match tokio::time::timeout(BUDGET, c.output()).await {
        Ok(Ok(o)) => o,
        Ok(Err(_)) => return Err(fail("not-installed", "Safe is not installed.")),
        Err(_) => return Err(fail("timeout", "Safe did not answer.")),
    };
    let Some(v) = browser::last_json(&String::from_utf8_lossy(&out.stdout)) else {
        return Err(fail("internal", "Safe did not answer."));
    };
    if v.get("ok") == Some(&Value::Bool(true)) {
        Ok(v)
    } else {
        Err(fail(
            v["error"].as_str().unwrap_or("internal"),
            v["message"].as_str().unwrap_or("Safe could not do that."),
        ))
    }
}

/// A failure, said on the island; a locked safe offers *Unlock Safe*,
/// with the command that was waiting for it.
fn failed_with(f: Fail, then: Option<Call>) -> Outcome {
    let line = match f.code.as_str() {
        "locked" => "Safe is locked.".to_string(),
        "not-set-up" => "Safe is not set up on this computer yet.".to_string(),
        "daemon-unavailable" => "Safe's service is not running.".to_string(),
        "not-installed" | "timeout" => f.message.clone(),
        _ if f.message.is_empty() => "Safe could not do that.".to_string(),
        _ => f.message.clone(),
    };
    let private = match f.code.as_str() {
        "locked" | "not-set-up" | "daemon-unavailable" | "not-installed" | "timeout" => {
            line.clone()
        }
        // Safe's own sentence may name an item: the model gets less
        _ => "Safe could not do that.".to_string(),
    };
    let action = (f.code == "locked").then(|| {
        (
            "Unlock Safe".to_string(),
            Call::new("safe_unlock", json!({"then": then})),
        )
    });
    Outcome {
        action,
        ..failed(line)
    }
    .private(private)
}

fn field_of(a: &Value) -> &str {
    a.get("field")
        .and_then(Value::as_str)
        .filter(|f| SAFE_FIELDS.contains(f))
        .unwrap_or("password")
}

fn field_word(field: &str) -> &'static str {
    match field {
        "username" => "username",
        "otp" => "one-time code",
        _ => "password",
    }
}

/// Logins that have what is asked for.
fn has(item: &Value, field: &str) -> bool {
    match field {
        "otp" => item["has_otp"].as_bool() == Some(true),
        "username" => !item["username"].as_str().unwrap_or("").is_empty(),
        _ => item["has_password"].as_bool() != Some(false),
    }
}

/// "copy my GitHub password": one login copies at once, several become
/// rows (title, username) whose click copies that one.
pub async fn copy(hub: &Arc<Hub>, a: &Value) -> Outcome {
    let field = field_of(a);
    if let Some(id) = a.get("id").and_then(Value::as_str) {
        let title = a["title"].as_str().unwrap_or("");
        let user = a["username"].as_str().unwrap_or("");
        return copy_one(hub, id, field, title, user).await;
    }
    let q = a["query"].as_str().unwrap_or("").trim();
    let found = match call(hub, &["match", "--query", q, "--limit", "20"]).await {
        Ok(v) => v["items"].as_array().cloned().unwrap_or_default(),
        Err(f) => return failed_with(f, Some(Call::new("safe_copy", a.clone()))),
    };
    let fit: Vec<&Value> = found.iter().filter(|i| has(i, field)).collect();
    match fit.as_slice() {
        [] if found.is_empty() => {
            failed(format!("Safe has no login for {q}.")).private("Safe has no login for that.")
        }
        [] => failed(format!(
            "No login for {q} in Safe has a {}.",
            field_word(field)
        ))
        .private(format!("No login found has a {}.", field_word(field))),
        [one] => {
            let id = one["id"].as_str().unwrap_or("");
            let title = one["title"].as_str().unwrap_or("");
            let user = one["username"].as_str().unwrap_or("");
            copy_one(hub, id, field, title, user).await
        }
        many => {
            let label = format!("Copy {}", field_word(field));
            let rows = many
                .iter()
                .take(ROWS)
                .map(|i| {
                    let title = i["title"].as_str().unwrap_or("");
                    let user = i["username"].as_str().unwrap_or("");
                    let folder = i["folder"].as_str().unwrap_or("");
                    let detail = [user, folder]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" · ");
                    json!({"title": title, "detail": detail, "icon": "key", "actions": [{
                        "label": label, "command": "safe_copy",
                        "args": {"id": i["id"], "field": field, "title": title, "username": user},
                    }]})
                })
                .collect();
            Outcome {
                rows,
                ..done(
                    format!("Safe has {} logins for {q}. Which one?", many.len()),
                    None,
                )
            }
            .private(format!(
                "Safe found {} logins; the user picks one on the island.",
                many.len()
            ))
        }
    }
}

async fn copy_one(hub: &Hub, id: &str, field: &str, title: &str, user: &str) -> Outcome {
    let then = Call::new(
        "safe_copy",
        json!({"id": id, "field": field, "title": title, "username": user}),
    );
    let v = match call(hub, &["copy", id, "--field", field]).await {
        Ok(v) => v,
        Err(f) => return failed_with(f, Some(then)),
    };
    let what = match (title.is_empty(), user.is_empty()) {
        (false, false) => format!("the {title} {} ({user})", field_word(field)),
        (false, true) => format!("the {title} {}", field_word(field)),
        _ => format!("the {}", field_word(field)),
    };
    let clears = match v["cleared_after"].as_u64() {
        Some(0) => "It stays on the clipboard.".to_string(),
        Some(s) => format!("It clears in {s} s."),
        None => String::new(),
    };
    done(
        format!("Copied {what}. {clears}").trim_end().to_string(),
        None,
    )
    .private(format!(
        "Done: Safe copied the {} to the clipboard.",
        field_word(field)
    ))
}

/// A site name that is already an address ("github.com").
fn looks_like_site(q: &str) -> bool {
    q.contains('.') && !q.contains(' ')
}

/// "log me in to GitHub": the login's website opens in ewe browser; the
/// fill stays the user's click there (or Super+P). The companion never
/// types.
pub async fn login(hub: &Arc<Hub>, q: &str) -> Outcome {
    let q = q.trim();
    let found = match call(hub, &["match", "--query", q, "--limit", "20"]).await {
        Ok(v) => v["items"].as_array().cloned().unwrap_or_default(),
        // locked, but the words are an address: open it anyway
        Err(f) if f.code == "locked" && looks_like_site(q) => {
            return open_site(hub, &format!("https://{q}")).await;
        }
        Err(f) => {
            return failed_with(f, Some(Call::new("safe_login", json!({"query": q}))));
        }
    };
    let mut sites: Vec<(&Value, String)> = Vec::new();
    for i in &found {
        let url = i["urls"]
            .as_array()
            .and_then(|u| {
                u.iter()
                    .filter_map(Value::as_str)
                    .find(|u| u.starts_with("https://") || u.starts_with("http://"))
            })
            .map(str::to_string);
        if let Some(url) = url
            && !sites
                .iter()
                .any(|(_, s)| browser::host(s) == browser::host(&url))
        {
            sites.push((i, url));
        }
    }
    match sites.as_slice() {
        // nothing in Safe: the words may mean something else ("sign in to
        // nextcloud" is ewe-sync's), so the chat asks the pack or a model
        [] => missed(format!("Safe has no website for {q}.")),
        [(_, url)] => open_site(hub, url).await,
        many => Outcome {
            rows: many
                .iter()
                .take(ROWS)
                .map(|(i, url)| {
                    let user = i["username"].as_str().unwrap_or("");
                    let detail = [user.to_string(), browser::host(url)]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" · ");
                    json!({"title": i["title"], "detail": detail, "icon": "key", "actions": [{
                        "label": "Open", "command": "open_url", "args": {"url": url},
                    }]})
                })
                .collect(),
            ..done(
                format!(
                    "Safe has logins for {q} on {} sites. Which one?",
                    many.len()
                ),
                None,
            )
        }
        .private(format!(
            "Safe has logins on {} sites; the user picks one on the island.",
            many.len()
        )),
    }
}

async fn open_site(hub: &Arc<Hub>, url: &str) -> Outcome {
    let out = browser::open_url(hub, url, None).await;
    if !out.ok {
        return out.private("The site did not open.");
    }
    Outcome {
        line: format!(
            "Opened {}. Click the login field and press Super+P to fill it.",
            browser::host(url)
        ),
        ..out
    }
    .private("Opened the login's website; the user fills it there.")
}

pub async fn lock(hub: &Arc<Hub>) -> Outcome {
    match call(hub, &["lock"]).await {
        Ok(_) => done("Safe is locked.", None).private("Safe is locked."),
        Err(f) => failed_with(f, None),
    }
}

/// "Unlock Safe": Safe's own prompt opens (the island runs `ewe-safe
/// unlock --ask`), and when the safe opens, the command that was waiting
/// for it (`then`) runs and its reply comes as a new chat line.
pub async fn unlock(hub: &Arc<Hub>, then: Option<&Value>) -> Outcome {
    let then: Option<Call> = then
        .filter(|t| !t.is_null())
        .and_then(|t| serde_json::from_value(t.clone()).ok());
    match call(hub, &["summary"]).await {
        Ok(v) if v["locked"].as_bool() == Some(false) => {
            return match then {
                Some(next) => run_then(hub, &next).await,
                None => done("Safe is already unlocked.", None).private("Safe is unlocked."),
            };
        }
        Ok(_) => {}
        Err(f) if f.code == "locked" => {}
        Err(f) => return failed_with(f, None),
    }
    let Some(bin) = bin() else {
        return failed("Safe is not installed.");
    };
    if let Err(e) = hub.shell_do("safe_unlock", json!({"bin": bin})).await {
        return failed(e);
    }
    tokio::spawn(wait_for_unlock(hub.clone(), then));
    done("Type your master password in Safe's window.", None)
        .private("Safe's unlock window is open.")
}

/// Watch for the unlock, then run what was waiting and say how it went.
/// When the safe does not open (Safe stops answering, or 3 minutes pass
/// with something waiting), say that what waited was dropped: a request
/// never vanishes without a word.
async fn wait_for_unlock(hub: Arc<Hub>, then: Option<Call>) {
    let start = tokio::time::Instant::now();
    let (call, out) = loop {
        if start.elapsed() >= UNLOCK_WAIT {
            if then.is_none() {
                return; // nothing waited: the prompt was just left
            }
            let line = "Safe is still locked, so I dropped what was waiting. Ask again once it is unlocked.";
            break (
                Call::new("safe_unlock", json!({})),
                failed(line).private(line),
            );
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        match call(&hub, &["summary"]).await {
            Ok(v) if v["locked"].as_bool() == Some(false) => match then {
                Some(next) => {
                    let out = run_then(&hub, &next).await;
                    break (next, out);
                }
                None => {
                    break (
                        Call::new("safe_unlock", json!({})),
                        done("Safe is unlocked.", None).private("Safe is unlocked."),
                    );
                }
            },
            Ok(_) => {}
            Err(f) if f.code == "locked" => {}
            // Safe went away before it opened
            Err(_) => {
                let line = if then.is_some() {
                    "The unlock did not finish: Safe stopped answering, so I dropped what was waiting. Ask again."
                } else {
                    "The unlock did not finish: Safe stopped answering."
                };
                break (
                    Call::new("safe_unlock", json!({})),
                    failed(line).private(line),
                );
            }
        }
    };
    let r = chat::reply_of(&hub, &call, out);
    hub.broadcast(proto::line(
        "chat.reply",
        chat::reply_json(&new_id("m"), &r),
    ));
}

/// The Safe command that waited for the unlock (validated as a catalog
/// call already: `safe_unlock`'s `then`).
async fn run_then(hub: &Arc<Hub>, next: &Call) -> Outcome {
    match next.command.as_str() {
        "safe_copy" => copy(hub, &next.args).await,
        "safe_login" => login(hub, next.args["query"].as_str().unwrap_or("")).await,
        _ => status(hub).await,
    }
}

/// "is my safe synced?", "how many passwords do I have?": counts and
/// states only (the `summary` verb exists for the companion).
pub async fn status(hub: &Arc<Hub>) -> Outcome {
    let v = match call(hub, &["summary"]).await {
        Ok(v) => v,
        Err(f) => return failed_with(f, Some(Call::new("safe_status", json!({})))),
    };
    if v["locked"].as_bool() != Some(false) {
        let line = "Safe is locked, so I can't count what is in it.";
        return Outcome {
            action: Some((
                "Unlock Safe".into(),
                Call::new(
                    "safe_unlock",
                    json!({"then": {"command": "safe_status", "args": {}}}),
                ),
            )),
            ..done(line, None)
        }
        .private("Safe is locked.");
    }
    let items = match v["items"].as_u64() {
        Some(1) => "1 item".to_string(),
        Some(n) => format!("{n} items"),
        None => String::new(),
    };
    let sync = match v["sync_state"].as_str().unwrap_or("") {
        "in-sync" => "in sync",
        "pending" => "changes are waiting to sync",
        "offline" => "offline: it syncs when the server can be reached",
        "error" => "sync has a problem (ewe-safe status says more)",
        "local" => "kept on this computer only",
        _ => "",
    };
    let line = ["Safe is unlocked", &items, sync]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    done(format!("{line}."), None).private(format!("{line}."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_locked_safe_offers_its_unlock_and_keeps_the_command() {
        let next = Call::new("safe_copy", json!({"query": "github"}));
        let out = failed_with(fail("locked", "Safe is locked."), Some(next));
        let (label, call) = out.action.expect("an Unlock button");
        assert_eq!(label, "Unlock Safe");
        assert_eq!(call.command, "safe_unlock");
        assert_eq!(call.args["then"]["command"], "safe_copy");
        assert_eq!(companion_core::commands::validate(&call), Ok(()));
        assert_eq!(out.private.as_deref(), Some("Safe is locked."));
    }

    #[test]
    fn safe_s_own_sentences_never_reach_a_model() {
        let out = failed_with(fail("not-found", "GitHub has no one-time code."), None);
        assert_eq!(out.line, "GitHub has no one-time code.");
        assert_eq!(out.private.as_deref(), Some("Safe could not do that."));
    }

    #[test]
    fn what_a_login_has() {
        let i = json!({"username": "sam", "has_password": true, "has_otp": false});
        assert!(has(&i, "password") && has(&i, "username") && !has(&i, "otp"));
        assert_eq!(field_of(&json!({"field": "notes"})), "password");
        assert_eq!(field_word("otp"), "one-time code");
        assert!(looks_like_site("github.com") && !looks_like_site("my bank"));
    }
}
