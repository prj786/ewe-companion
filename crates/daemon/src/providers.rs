//! Who answers a question (PLAN §11, decision 4), chosen in the plugin's
//! settings (Komble → Plugins → Shaun: its sheet, Who answers questions).
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
//! **Fast, and only Shaun's.** Claude Code runs lean: none of the user's
//! own Claude Code setup (plugins, skills, MCP servers, settings, hooks)
//! comes along, which was ~94k tokens and 7.5 s for "2+2" on the owner's
//! machine (2026-11-08); lean, it is ~650 tokens. Starting `claude` takes
//! about 4 s, so [`warm`] starts one ahead of time when the chat field gets
//! focus (`chat.warm`) and after every answer, and drops it after
//! [`WARM_IDLE`] (an idle one holds ~190 MB). A warm one answers in about
//! a second.
//!
//! **Streaming.** Claude Code (stream-json), the Anthropic API and the
//! OpenAI-compatible APIs (SSE) hand each piece of text to a [`Sink`] as
//! it arrives, so the island shows the answer growing. Codex and OpenCode
//! answer whole.
//!
//! Nothing here logs a prompt, an answer or a key.

use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use companion_core::proto::ChatSettings;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, info};

use crate::keyring;

const API_BUDGET: Duration = Duration::from_secs(60);
const CLI_BUDGET: Duration = Duration::from_secs(120);
/// How long a warm `claude` waits for a question before it is dropped.
pub const WARM_IDLE: Duration = Duration::from_secs(180);

/// The answer so far, as it streams in (the whole text each time, not the
/// new piece).
pub type Sink = dyn Fn(&str) + Send + Sync;

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

/// The model these settings use: the one set, else the provider's own.
fn model_of(s: &ChatSettings, p: &Info) -> String {
    if s.model.trim().is_empty() {
        p.default_model.to_string()
    } else {
        s.model.trim().to_string()
    }
}

/// What the island shows about the chosen provider.
pub async fn status(s: &ChatSettings) -> Value {
    let Some(p) = info(&s.provider) else {
        return json!({"provider": "none", "label": "No provider", "ready": false, "needs_key": false,
                      "hint": "I run commands and know my way round ewe on my own. For other questions, set Who answers questions in Komble → Plugins → Shaun."});
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
        "model": model_of(s, p),
    })
}

/// One question, answered whole. `history` is (role, text), oldest first,
/// role `user` or `assistant`. `sink`, when given, sees the answer grow.
pub async fn ask(
    s: &ChatSettings,
    system: &str,
    history: &[(String, String)],
    text: &str,
    sink: Option<&Sink>,
) -> Result<String, String> {
    let p = info(&s.provider)
        .ok_or("No provider is set. Set Who answers questions in Komble → Plugins → Shaun.")?;
    let model = model_of(s, p);
    match p.cli {
        Some(cmd) => ask_cli(p, cmd, &model, system, history, text, sink).await,
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
            let msgs = Messages {
                system,
                history,
                text,
            };
            if p.id == "anthropic" {
                ask_anthropic(&base, key.as_deref().unwrap_or(""), &model, &msgs, sink).await
            } else {
                ask_openai_compat(p, &base, key.as_deref(), &model, &msgs, sink).await
            }
        }
    }
}

struct Messages<'a> {
    system: &'a str,
    history: &'a [(String, String)],
    text: &'a str,
}

impl Messages<'_> {
    /// The turns as the APIs take them: the history, then the question.
    fn turns(&self) -> Vec<Value> {
        let mut m: Vec<Value> = self
            .history
            .iter()
            .map(|(r, t)| json!({"role": r, "content": t}))
            .collect();
        m.push(json!({"role": "user", "content": self.text}));
        m
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

// ── server-sent events (the APIs' streaming) ───────────────────────────────

/// Splits a byte stream into the `data:` payloads of its events. Bytes are
/// kept until an event is whole, so a character split across two chunks
/// survives.
#[derive(Default)]
struct Sse {
    buf: Vec<u8>,
}

impl Sse {
    fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buf.extend_from_slice(chunk);
        let mut out = Vec::new();
        loop {
            // an event ends at a blank line (\n\n or \r\n\r\n)
            let end = self
                .buf
                .windows(2)
                .position(|w| w == b"\n\n")
                .map(|i| (i, 2))
                .into_iter()
                .chain(
                    self.buf
                        .windows(4)
                        .position(|w| w == b"\r\n\r\n")
                        .map(|i| (i, 4)),
                )
                .min_by_key(|(i, _)| *i);
            let Some((at, len)) = end else { break };
            let event: Vec<u8> = self.buf.drain(..at + len).collect();
            let event = String::from_utf8_lossy(&event[..at]).to_string();
            let data: Vec<&str> = event
                .lines()
                .filter_map(|l| l.trim_end_matches('\r').strip_prefix("data:"))
                .map(|d| d.strip_prefix(' ').unwrap_or(d))
                .collect();
            if !data.is_empty() {
                out.push(data.join("\n"));
            }
        }
        out
    }
}

/// Whether a response is an event stream (a server may ignore `stream`).
fn is_sse(res: &reqwest::Response) -> bool {
    res.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("text/event-stream"))
}

/// Read an event stream to its end, giving each `data:` payload to `each`;
/// `each` returns an error to stop.
async fn read_sse(
    mut res: reqwest::Response,
    mut each: impl FnMut(&str) -> Result<(), String>,
) -> Result<(), String> {
    let mut sse = Sse::default();
    while let Some(chunk) = res
        .chunk()
        .await
        .map_err(|_| "The answer stopped halfway.".to_string())?
    {
        for data in sse.push(&chunk) {
            each(&data)?;
        }
    }
    Ok(())
}

// ── Anthropic ──────────────────────────────────────────────────────────────

/// Claude 5-series models think by default (Haiku 5.5 at `medium`); a
/// quick chat answers faster at `low`. Older models take no `effort`.
fn quick_effort(model: &str) -> bool {
    model.starts_with("claude-")
        && model
            .split('-')
            .nth(2)
            .and_then(|v| v.parse::<u32>().ok())
            .is_some_and(|major| major >= 5)
}

async fn ask_anthropic(
    base: &str,
    key: &str,
    model: &str,
    m: &Messages<'_>,
    sink: Option<&Sink>,
) -> Result<String, String> {
    // the system prompt is the same from one question to the next (the
    // turn's notes ride with the question), so it is cached
    let mut body = json!({
        "model": model, "max_tokens": 2000, "stream": true,
        "system": [{"type": "text", "text": m.system, "cache_control": {"type": "ephemeral"}}],
        "messages": m.turns(),
    });
    if quick_effort(model) {
        body["output_config"] = json!({"effort": "low"});
    }
    let res = client()?
        .post(format!("{base}/messages"))
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|_| "Could not reach the Anthropic API.".to_string())?;
    let status = res.status();
    if !status.is_success() || !is_sse(&res) {
        let body: Value = res.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(api_error(status, &body));
        }
        return whole_anthropic(&body);
    }
    let mut text = String::new();
    let mut refused = false;
    read_sse(res, |data| {
        let v: Value = serde_json::from_str(data).unwrap_or(Value::Null);
        match v["type"].as_str() {
            Some("content_block_delta") if v["delta"]["type"] == "text_delta" => {
                text.push_str(v["delta"]["text"].as_str().unwrap_or(""));
                if let Some(s) = sink {
                    s(&text);
                }
            }
            Some("message_delta") if v["delta"]["stop_reason"] == "refusal" => refused = true,
            Some("error") => {
                return Err(companion_core::trim::cut(
                    v.pointer("/error/message")
                        .and_then(Value::as_str)
                        .unwrap_or("The provider stopped with an error."),
                    200,
                ));
            }
            _ => {}
        }
        Ok(())
    })
    .await?;
    if refused && text.trim().is_empty() {
        return Err("The model declined to answer that.".into());
    }
    if text.trim().is_empty() {
        Err("The model sent an empty answer.".into())
    } else {
        Ok(text)
    }
}

/// An answer that came back whole (no stream).
fn whole_anthropic(body: &Value) -> Result<String, String> {
    let text: String = body
        .get("content")
        .and_then(Value::as_array)
        .map(|blocks| {
            blocks
                .iter()
                .filter(|b| b["type"] == "text")
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

// ── OpenAI-compatible ──────────────────────────────────────────────────────

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
    m: &Messages<'_>,
    sink: Option<&Sink>,
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
    let mut messages = vec![json!({"role": "system", "content": m.system})];
    messages.extend(m.turns());
    let mut req = client()?
        .post(format!("{base}/chat/completions"))
        .json(&json!({"model": model, "messages": messages, "stream": true}));
    if let Some(k) = key {
        req = req.bearer_auth(k);
    }
    let res = req
        .send()
        .await
        .map_err(|_| format!("Could not reach {}.", p.label))?;
    let status = res.status();
    if !status.is_success() || !is_sse(&res) {
        let body: Value = res.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(api_error(status, &body));
        }
        return body
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|t| !t.trim().is_empty())
            .ok_or_else(|| "The model sent an empty answer.".into());
    }
    let mut text = String::new();
    read_sse(res, |data| {
        if data.trim() == "[DONE]" {
            return Ok(());
        }
        let v: Value = serde_json::from_str(data).unwrap_or(Value::Null);
        if let Some(e) = v.get("error").filter(|e| !e.is_null()) {
            return Err(companion_core::trim::cut(
                e.get("message")
                    .and_then(Value::as_str)
                    .or(e.as_str())
                    .unwrap_or("The provider stopped with an error."),
                200,
            ));
        }
        if let Some(piece) = v
            .pointer("/choices/0/delta/content")
            .and_then(Value::as_str)
        {
            text.push_str(piece);
            if let Some(s) = sink {
                s(&text);
            }
        }
        Ok(())
    })
    .await?;
    if text.trim().is_empty() {
        Err("The model sent an empty answer.".into())
    } else {
        Ok(text)
    }
}

// ── the agent lane ─────────────────────────────────────────────────────────

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
    sink: Option<&Sink>,
) -> Result<String, String> {
    if !on_path(cmd) {
        return Err(match p.id {
            "codex" => {
                "Codex is not installed (npm install -g @openai/codex, then codex login).".into()
            }
            other => format!("{other} is not installed."),
        });
    }
    if p.id == "claude-code" {
        return ask_claude(
            model,
            system,
            &transcript(system, history, text, false),
            sink,
        )
        .await;
    }
    let mut c = Command::new(cmd);
    c.current_dir(agent_home())
        // our own hooks stay out of the companion's own sessions
        .env("EWE_COMPANION_INTERNAL", "1")
        .env_remove("CLAUDECODE")
        .stdin(Stdio::null())
        .kill_on_drop(true);
    match p.id {
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

// ── Claude Code: lean, warm, streamed ──────────────────────────────────────

/// `claude` as Shaun runs it: print mode reading one question as
/// stream-json on stdin and streaming its answer, no tools, and none of
/// the user's own Claude Code setup (no skills or slash commands, no MCP
/// servers, no user/project/local settings and so no hooks or plugins).
/// OAuth still works (`--bare` would refuse it).
fn claude_command(model: &str, system: &str) -> Command {
    let mut c = Command::new("claude");
    c.current_dir(agent_home())
        // our own hooks stay out of the companion's own sessions
        .env("EWE_COMPANION_INTERNAL", "1")
        .env_remove("CLAUDECODE")
        .args(["-p", "--input-format", "stream-json"])
        .args(["--output-format", "stream-json", "--verbose"])
        .arg("--include-partial-messages")
        .args(["--model", model, "--system-prompt", system])
        .args(["--tools", "", "--permission-mode", "default"])
        .arg("--no-session-persistence")
        .args(["--disable-slash-commands", "--strict-mcp-config"])
        .args(["--setting-sources", ""])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    c
}

/// A started `claude`, waiting for its question.
struct Warm {
    /// the model and system prompt it was started with
    key: u64,
    child: Child,
    since: Instant,
}

fn warm_slot() -> &'static tokio::sync::Mutex<Option<Warm>> {
    static W: OnceLock<tokio::sync::Mutex<Option<Warm>>> = OnceLock::new();
    W.get_or_init(|| tokio::sync::Mutex::new(None))
}

fn warm_key(model: &str, system: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    model.hash(&mut h);
    system.hash(&mut h);
    h.finish()
}

/// Start a `claude` for the next question, if these settings use Claude
/// Code and none is waiting already. The one waiting is dropped after
/// [`WARM_IDLE`]; it is ~190 MB while it waits.
pub async fn warm(s: &ChatSettings, system: &str) {
    let Some(p) = info(&s.provider) else { return };
    if p.id != "claude-code" || !on_path("claude") {
        return;
    }
    let model = model_of(s, p);
    let key = warm_key(&model, system);
    let mut slot = warm_slot().lock().await;
    if let Some(w) = slot.as_mut()
        && w.key == key
        && matches!(w.child.try_wait(), Ok(None))
    {
        w.since = Instant::now();
        return;
    }
    match claude_command(&model, system).spawn() {
        Ok(child) => {
            debug!("claude warmed");
            *slot = Some(Warm {
                key,
                child,
                since: Instant::now(),
            });
            reaper();
        }
        Err(_) => *slot = None,
    }
}

/// Once: every 30 s, drop a warm `claude` that waited too long.
fn reaper() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        tokio::spawn(async {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let mut slot = warm_slot().lock().await;
                if slot.as_ref().is_some_and(|w| w.since.elapsed() > WARM_IDLE) {
                    debug!("warm claude dropped (idle)");
                    // kill_on_drop ends it
                    *slot = None;
                }
            }
        });
    });
}

/// The warm `claude` for this model and prompt, if one is alive.
async fn take_warm(key: u64) -> Option<Child> {
    let mut slot = warm_slot().lock().await;
    let mut w = slot.take()?;
    (w.key == key && matches!(w.child.try_wait(), Ok(None))).then_some(w.child)
}

async fn ask_claude(
    model: &str,
    system: &str,
    prompt: &str,
    sink: Option<&Sink>,
) -> Result<String, String> {
    let key = warm_key(model, system);
    let (mut child, warmed) = match take_warm(key).await {
        Some(c) => (c, true),
        None => (
            claude_command(model, system)
                .spawn()
                .map_err(|_| "Could not start claude.".to_string())?,
            false,
        ),
    };
    let started = Instant::now();
    let (Some(mut stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
        return Err("Could not start claude.".into());
    };
    // what it says on stderr, for an error line (bounded; read so the pipe
    // never fills)
    let stderr = child.stderr.take().map(|mut e| {
        tokio::spawn(async move {
            let mut buf = Vec::new();
            let _ = (&mut e).take(8 * 1024).read_to_end(&mut buf).await;
            String::from_utf8_lossy(&buf).to_string()
        })
    });
    let msg = json!({"type": "user", "message": {"role": "user", "content": prompt}});
    if stdin
        .write_all(format!("{msg}\n").as_bytes())
        .await
        .is_err()
        || stdin.flush().await.is_err()
    {
        return Err("Claude Code did not take the question.".into());
    }
    let mut lines = BufReader::new(stdout).lines();
    let mut text = String::new();
    let read = async {
        while let Ok(Some(line)) = lines.next_line().await {
            let Ok(v) = serde_json::from_str::<Value>(&line) else {
                continue;
            };
            match v["type"].as_str() {
                Some("stream_event") => {
                    let e = &v["event"];
                    if e["type"] == "content_block_delta" && e["delta"]["type"] == "text_delta" {
                        text.push_str(e["delta"]["text"].as_str().unwrap_or(""));
                        if let Some(s) = sink {
                            s(&text);
                        }
                    }
                }
                Some("result") => return Some(v),
                _ => {}
            }
        }
        None
    };
    let result = tokio::time::timeout(CLI_BUDGET, read).await;
    // the question is answered: let it end (or end it)
    drop(stdin);
    tokio::spawn(async move {
        let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
    });
    info!(
        warmed,
        ms = started.elapsed().as_millis() as u64,
        "claude answered"
    );
    let v = match result {
        Err(_) => return Err("Claude Code took longer than two minutes.".into()),
        Ok(Some(v)) => v,
        Ok(None) => {
            let err = match stderr {
                Some(h) => h.await.unwrap_or_default(),
                None => String::new(),
            };
            let last = strip_ansi(&err);
            let last = last.trim().lines().last().unwrap_or("").trim();
            return Err(if last.is_empty() {
                "Claude Code did not answer. Is it logged in? Run claude once in a terminal.".into()
            } else {
                companion_core::trim::cut(last, 200)
            });
        }
    };
    let result = v["result"].as_str().unwrap_or("").to_string();
    if v["is_error"].as_bool() == Some(true) || result.trim().is_empty() {
        return Err(if result.trim().is_empty() {
            "Claude Code did not answer. Is it logged in? Run claude once in a terminal.".into()
        } else {
            companion_core::trim::cut(&result, 200)
        });
    }
    Ok(result)
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

    #[test]
    fn events_survive_any_chunking() {
        let stream =
            "event: content_block_delta\ndata: {\"t\":\"Bä\"}\n\n: ping\n\ndata: [DONE]\r\n\r\n";
        let bytes = stream.as_bytes();
        // every split point, including inside the two-byte "ä"
        for cut in 0..bytes.len() {
            let mut sse = Sse::default();
            let mut got = sse.push(&bytes[..cut]);
            got.extend(sse.push(&bytes[cut..]));
            assert_eq!(got, vec!["{\"t\":\"Bä\"}", "[DONE]"], "cut at {cut}");
        }
    }

    #[test]
    fn only_5_series_claude_gets_effort() {
        assert!(quick_effort("claude-haiku-5-5"));
        assert!(quick_effort("claude-sonnet-5"));
        assert!(!quick_effort("claude-haiku-4-5"));
        assert!(!quick_effort("gpt-5-mini"));
        assert!(!quick_effort("haiku"));
    }

    #[test]
    fn claude_runs_lean() {
        let c = claude_command("haiku", "SYS");
        let args: Vec<String> = c
            .as_std()
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        for flag in [
            "--disable-slash-commands",
            "--strict-mcp-config",
            "--setting-sources",
            "--no-session-persistence",
            "--include-partial-messages",
        ] {
            assert!(args.iter().any(|a| a == flag), "{flag}");
        }
        // --setting-sources "" (none of the user's settings)
        let i = args.iter().position(|a| a == "--setting-sources").unwrap();
        assert_eq!(args[i + 1], "");
        assert!(!args.iter().any(|a| a == "--bare"), "--bare refuses OAuth");
    }
}
