//! Who answers a question (PLAN §11, decision 4), chosen in the plugin's
//! settings (Komble → Plugins → Companion → Options).
//!
//! Two lanes:
//!
//! - **Your own agent, your own login.** `claude-code`, `codex`, `opencode`:
//!   the companion runs the agent you are already logged in to, in its
//!   print mode, with **no tools** (it answers; the companion acts through
//!   its own commands). The companion never sees or stores that login. For
//!   Claude, Anthropic's rules for apps that drive your Claude Code are not
//!   settled, which is why it is a choice you make, never a default.
//! - **An API key or a local model.** `anthropic`, and every
//!   OpenAI-compatible endpoint: `openai`, `gemini` (AI Studio key),
//!   `openrouter`, `ollama`, `lmstudio`, `custom`. The key lives in the
//!   keyring ([`crate::keyring`]); local models need none.
//!
//! Answers come back whole (streaming is a later step). Nothing here logs a
//! prompt, an answer or a key.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use companion_core::proto::ChatSettings;
use serde_json::{Value, json};
use tokio::process::Command;

use crate::keyring;

const API_BUDGET: Duration = Duration::from_secs(60);
const CLI_BUDGET: Duration = Duration::from_secs(120);

pub struct Info {
    pub id: &'static str,
    pub label: &'static str,
    /// a CLI agent the user logs in to themselves
    pub cli: Option<&'static str>,
    pub base_url: &'static str,
    pub needs_key: bool,
    pub default_model: &'static str,
}

pub const PROVIDERS: &[Info] = &[
    Info {
        id: "claude-code",
        label: "Claude Code (your login)",
        cli: Some("claude"),
        base_url: "",
        needs_key: false,
        default_model: "haiku",
    },
    Info {
        id: "codex",
        label: "Codex (your ChatGPT login)",
        cli: Some("codex"),
        base_url: "",
        needs_key: false,
        default_model: "",
    },
    Info {
        id: "opencode",
        label: "OpenCode (its providers)",
        cli: Some("opencode"),
        base_url: "",
        needs_key: false,
        default_model: "",
    },
    Info {
        id: "anthropic",
        label: "Anthropic API",
        cli: None,
        base_url: "https://api.anthropic.com/v1",
        needs_key: true,
        default_model: "claude-haiku-5-5",
    },
    Info {
        id: "openai",
        label: "OpenAI API",
        cli: None,
        base_url: "https://api.openai.com/v1",
        needs_key: true,
        default_model: "gpt-5-mini",
    },
    Info {
        id: "gemini",
        label: "Gemini API (AI Studio key)",
        cli: None,
        base_url: "https://generativelanguage.googleapis.com/v1beta/openai",
        needs_key: true,
        default_model: "gemini-2.5-flash",
    },
    Info {
        id: "openrouter",
        label: "OpenRouter",
        cli: None,
        base_url: "https://openrouter.ai/api/v1",
        needs_key: true,
        default_model: "openrouter/auto",
    },
    Info {
        id: "ollama",
        label: "Ollama (local)",
        cli: None,
        base_url: "http://localhost:11434/v1",
        needs_key: false,
        default_model: "",
    },
    Info {
        id: "lmstudio",
        label: "LM Studio (local)",
        cli: None,
        base_url: "http://localhost:1234/v1",
        needs_key: false,
        default_model: "",
    },
    Info {
        id: "custom",
        label: "Custom (OpenAI-compatible)",
        cli: None,
        base_url: "",
        needs_key: false,
        default_model: "",
    },
];

pub fn info(id: &str) -> Option<&'static Info> {
    PROVIDERS.iter().find(|p| p.id == id)
}

pub fn on_path(cmd: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(cmd).is_file()))
        .unwrap_or(false)
}

/// What the island shows about the chosen provider.
pub async fn status(s: &ChatSettings) -> Value {
    let Some(p) = info(&s.provider) else {
        return json!({"provider": "none", "label": "No provider", "ready": false, "needs_key": false,
                      "hint": "I run commands and know my way round ewe on my own. For other questions, pick who answers: Komble → Plugins → Companion → Options."});
    };
    let (ready, hint) = match p.cli {
        Some(cmd) if !on_path(cmd) => (false, format!("{} is not installed.", p.label)),
        Some(_) => (true, String::new()),
        None if p.needs_key && keyring::get(p.id).await.is_none() => {
            (false, "Paste your API key to start.".into())
        }
        None if p.id == "custom" && s.base_url.trim().is_empty() => (
            false,
            "Set the server address in the plugin's options.".into(),
        ),
        None => (true, String::new()),
    };
    json!({
        "provider": p.id, "label": p.label, "ready": ready, "hint": hint,
        "needs_key": p.needs_key && !ready && p.cli.is_none(),
        "model": if s.model.trim().is_empty() { p.default_model } else { s.model.trim() },
    })
}

/// One question, answered whole. `history` is (role, text), oldest first,
/// role `user` or `assistant`.
pub async fn ask(
    s: &ChatSettings,
    system: &str,
    history: &[(String, String)],
    text: &str,
) -> Result<String, String> {
    let p = info(&s.provider)
        .ok_or("No provider is set. Pick one in Komble → Plugins → Companion → Options.")?;
    let model = if s.model.trim().is_empty() {
        p.default_model.to_string()
    } else {
        s.model.trim().to_string()
    };
    match p.cli {
        Some(cmd) => ask_cli(p, cmd, &model, system, history, text).await,
        None => {
            let base = if p.id == "custom" || !s.base_url.trim().is_empty() {
                s.base_url.trim().trim_end_matches('/').to_string()
            } else {
                p.base_url.to_string()
            };
            if base.is_empty() {
                return Err("Set the server address in the plugin's options.".into());
            }
            let key = keyring::get(p.id).await;
            if p.needs_key && key.is_none() {
                return Err(format!(
                    "{} needs an API key. Paste it in the island; it goes to your keyring.",
                    p.label
                ));
            }
            if p.id == "anthropic" {
                ask_anthropic(
                    &base,
                    key.as_deref().unwrap_or(""),
                    &model,
                    system,
                    history,
                    text,
                )
                .await
            } else {
                ask_openai_compat(p, &base, key.as_deref(), &model, system, history, text).await
            }
        }
    }
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(API_BUDGET)
        .user_agent(concat!("ewe-companion/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("http: {e}"))
}

/// A provider's own error message, or the HTTP status in words.
fn api_error(status: reqwest::StatusCode, body: &Value) -> String {
    let msg = body
        .pointer("/error/message")
        .and_then(Value::as_str)
        .or_else(|| body.get("error").and_then(Value::as_str));
    match (status.as_u16(), msg) {
        (401 | 403, _) => "The provider refused the key. Paste a new one.".into(),
        (_, Some(m)) => companion_core::trim::cut(m, 200),
        (code, None) => format!("The provider answered {code}."),
    }
}

async fn ask_anthropic(
    base: &str,
    key: &str,
    model: &str,
    system: &str,
    history: &[(String, String)],
    text: &str,
) -> Result<String, String> {
    let mut messages: Vec<Value> = history
        .iter()
        .map(|(r, t)| json!({"role": r, "content": t}))
        .collect();
    messages.push(json!({"role": "user", "content": text}));
    let res = client()?
        .post(format!("{base}/messages"))
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({"model": model, "max_tokens": 800, "system": system, "messages": messages}))
        .send()
        .await
        .map_err(|_| "Could not reach the Anthropic API.".to_string())?;
    let status = res.status();
    let body: Value = res.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        return Err(api_error(status, &body));
    }
    let text: String = body
        .get("content")
        .and_then(Value::as_array)
        .map(|blocks| {
            blocks
                .iter()
                .filter_map(|b| b.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default();
    if text.trim().is_empty() {
        Err("The model sent an empty answer.".into())
    } else {
        Ok(text)
    }
}

/// A local server's first model, when none is set (Ollama, LM Studio).
async fn first_model(base: &str) -> Option<String> {
    let body: Value = client()
        .ok()?
        .get(format!("{base}/models"))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    body.get("data")?
        .as_array()?
        .first()?
        .get("id")?
        .as_str()
        .map(str::to_string)
}

async fn ask_openai_compat(
    p: &Info,
    base: &str,
    key: Option<&str>,
    model: &str,
    system: &str,
    history: &[(String, String)],
    text: &str,
) -> Result<String, String> {
    let model = match model {
        "" => first_model(base).await.ok_or_else(|| match p.id {
            "ollama" => {
                "Ollama is not running, or has no model. Try: ollama pull qwen3:8b".to_string()
            }
            "lmstudio" => "LM Studio's server is not running, or has no model loaded.".to_string(),
            _ => "Set a model in the plugin's options.".to_string(),
        })?,
        m => m.to_string(),
    };
    let mut messages = vec![json!({"role": "system", "content": system})];
    messages.extend(
        history
            .iter()
            .map(|(r, t)| json!({"role": r, "content": t})),
    );
    messages.push(json!({"role": "user", "content": text}));
    let mut req = client()?
        .post(format!("{base}/chat/completions"))
        .json(&json!({"model": model, "messages": messages}));
    if let Some(k) = key {
        req = req.bearer_auth(k);
    }
    let res = req
        .send()
        .await
        .map_err(|_| format!("Could not reach {}.", p.label))?;
    let status = res.status();
    let body: Value = res.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        return Err(api_error(status, &body));
    }
    body.pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| "The model sent an empty answer.".into())
}

/// The conversation as one prompt, for agents driven through print mode.
fn transcript(system: &str, history: &[(String, String)], text: &str, with_system: bool) -> String {
    let mut s = String::new();
    if with_system {
        s.push_str(system);
        s.push_str("\n\n");
    }
    if !history.is_empty() {
        s.push_str("Earlier in this chat:\n");
        for (r, t) in history {
            s.push_str(if r == "user" { "User: " } else { "You: " });
            s.push_str(t);
            s.push('\n');
        }
        s.push('\n');
    }
    s.push_str(text);
    s
}

/// Where agent-lane chats run: an empty directory of our own, so an agent
/// that looks around sees nothing of yours (PLAN §11.1).
fn agent_home() -> PathBuf {
    let d = companion_core::paths::state_dir().join("agent-home");
    let _ = std::fs::create_dir_all(&d);
    d
}

async fn ask_cli(
    p: &Info,
    cmd: &str,
    model: &str,
    system: &str,
    history: &[(String, String)],
    text: &str,
) -> Result<String, String> {
    if !on_path(cmd) {
        return Err(match p.id {
            "codex" => {
                "Codex is not installed (npm install -g @openai/codex, then codex login).".into()
            }
            other => format!("{other} is not installed."),
        });
    }
    let mut c = Command::new(cmd);
    c.current_dir(agent_home())
        // our own hooks stay out of the companion's own sessions
        .env("EWE_COMPANION_INTERNAL", "1")
        .env_remove("CLAUDECODE")
        .stdin(Stdio::null())
        .kill_on_drop(true);
    match p.id {
        "claude-code" => {
            c.args(["-p", &transcript(system, history, text, false)])
                .args(["--model", model, "--system-prompt", system])
                .args([
                    "--tools",
                    "",
                    "--permission-mode",
                    "default",
                    "--no-session-persistence",
                    "--output-format",
                    "json",
                ]);
        }
        // (verify) codex exec's flags move between releases; checked against
        // the docs of 2026-10, not a local install
        "codex" => {
            c.args(["exec", "--skip-git-repo-check", "--sandbox", "read-only"]);
            if !model.is_empty() {
                c.args(["--model", model]);
            }
            c.arg(transcript(system, history, text, true));
        }
        _ => {
            // OpenCode's read-only `plan` agent: it answers, it does not edit
            c.args(["run", "--agent", "plan"]);
            if !model.is_empty() {
                c.args(["--model", model]);
            }
            c.arg(transcript(system, history, text, true));
        }
    }
    let out = tokio::time::timeout(CLI_BUDGET, c.output())
        .await
        .map_err(|_| format!("{} took longer than two minutes.", p.label))?
        .map_err(|_| format!("Could not start {cmd}."))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if p.id == "claude-code" {
        let v: Value = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
        let result = v
            .get("result")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if v.get("is_error").and_then(Value::as_bool) == Some(true) || result.trim().is_empty() {
            return Err(if result.trim().is_empty() {
                "Claude Code did not answer. Is it logged in? Run claude once in a terminal.".into()
            } else {
                companion_core::trim::cut(&result, 200)
            });
        }
        return Ok(result);
    }
    let text = strip_ansi(&stdout);
    if !out.status.success() || text.trim().is_empty() {
        let err = strip_ansi(&String::from_utf8_lossy(&out.stderr));
        return Err(companion_core::trim::cut(
            err.trim().lines().last().unwrap_or("No answer."),
            200,
        ));
    }
    Ok(text.trim().to_string())
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(ch) = it.next() {
        if ch == '\u{1b}' {
            if it.peek() == Some(&'[') {
                it.next();
                for c in it.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_provider_is_known_once() {
        let mut ids: Vec<&str> = PROVIDERS.iter().map(|p| p.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), PROVIDERS.len());
        assert!(info("claude-code").unwrap().cli.is_some());
        assert!(info("anthropic").unwrap().needs_key);
        assert!(!info("ollama").unwrap().needs_key);
    }

    #[test]
    fn transcripts_carry_history() {
        let h = vec![
            ("user".to_string(), "hi".to_string()),
            ("assistant".to_string(), "Hello.".to_string()),
        ];
        let t = transcript("SYS", &h, "and now?", true);
        assert!(t.starts_with("SYS"));
        assert!(t.contains("User: hi\nYou: Hello."));
        assert!(t.ends_with("and now?"));
    }

    #[test]
    fn ansi_is_stripped() {
        assert_eq!(strip_ansi("\u{1b}[1;32mok\u{1b}[0m done"), "ok done");
    }
}
