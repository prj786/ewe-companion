//! The socket protocol, v1 (docs/PROTOCOL.md is the contract).
//!
//! UTF-8 JSON, one object per line, `\n`-terminated, at most [`MAX_LINE`]
//! bytes. Every object carries `"v"` and `"type"`. Message types are public
//! API between our own repos: fields may be added, nothing is renamed.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const PROTOCOL: u64 = 1;
pub const MAX_LINE: usize = 1 << 20;

/// A button on a card or a peek.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Action {
    pub id: String,
    pub label: String,
    /// `primary` · `secondary` · `danger` · `ghost` (the Button variants)
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub style: String,
    /// one more line of meaning, shown as the button's tooltip
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hint: String,
}

impl Action {
    pub fn new(id: &str, label: &str, style: &str) -> Self {
        Action {
            id: id.into(),
            label: label.into(),
            style: style.into(),
            hint: String::new(),
        }
    }
}

/// Something that stays until it is answered, snoozed or moot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Card {
    pub id: String,
    /// `permission` · `question` · `note` · `confirm` · `digest`
    pub kind: String,
    /// `agent:claude`, `companion`, `notify:<app>` …
    pub source: String,
    pub title: String,
    #[serde(default)]
    pub body: String,
    /// the project (`basename(cwd)`) an agent card belongs to
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub project: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(default)]
    pub actions: Vec<Action>,
    /// unix ms when the card stops waiting; 0 = it does not expire
    #[serde(default)]
    pub deadline_ms: u64,
    #[serde(default)]
    pub created_ms: u64,
    /// kind-specific: a command or diff preview, the questions …
    #[serde(default)]
    pub payload: Value,
}

/// One line in the island for a few seconds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Peek {
    /// the same key replaces the line instead of queueing a second one
    pub key: String,
    pub line: String,
    /// `info` · `success` · `warning` · `danger`
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub actions: Vec<Action>,
    /// 0 = the island's own `peek_seconds`
    #[serde(default)]
    pub ttl_ms: u64,
}

/// An agent session the island lists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Session {
    pub id: String,
    pub agent: String,
    pub cwd: String,
    pub project: String,
    /// `working` · `waiting` · `idle` · `done` · `failed`
    pub state: String,
    /// the last thing it did, in words: `Edit Bar.qml`
    #[serde(default)]
    pub last: String,
    #[serde(default)]
    pub files_changed: u32,
    #[serde(default)]
    pub started: i64,
    #[serde(default)]
    pub updated: i64,
    /// terminal identity from the hook (kitty window, tmux pane, pid chain)
    #[serde(default)]
    pub term: Value,
    /// its last few steps, oldest first (in memory only)
    #[serde(default)]
    pub steps: Vec<String>,
}

/// The chat settings the island reads from its plugin settings (Komble →
/// Plugins → Companion → Options). Never a key: keys go to the keyring.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatSettings {
    /// `none` · `claude-code` · `codex` · `opencode` · `anthropic` · `openai`
    /// · `gemini` · `openrouter` · `ollama` · `lmstudio` · `custom`
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub base_url: String,
}

/// What the shell knows best and the daemon needs to stay quiet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Env {
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub dnd: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub casting: bool,
    #[serde(default)]
    pub idle: bool,
    /// the shell's `HYPRLAND_INSTANCE_SIGNATURE`, so the daemon's `hyprctl`
    /// talks to the compositor the island lives in
    #[serde(default)]
    pub hypr: String,
    /// the shell's `WAYLAND_DISPLAY`, for what the daemon asks Safe to copy
    /// (a user unit may not have one)
    #[serde(default)]
    pub wayland: String,
}

/// A hook relay's event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentMsg {
    pub agent: String,
    pub event: String,
    #[serde(default)]
    pub wait: bool,
    #[serde(default)]
    pub term: Value,
    #[serde(default)]
    pub payload: Value,
}

/// The island's answer to a card.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Answer {
    pub id: String,
    /// `allow` · `deny` · `always:<n>` · `answer` · `terminal` · `later`
    /// · any action id the card offered
    pub action: String,
    /// for questions: `{ "<question text>": "<label>" }`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answers: Option<Value>,
}

/// Everything a client may send the daemon.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Inbound {
    #[serde(rename = "hello")]
    Hello {
        #[serde(default)]
        client: String,
        #[serde(default)]
        pid: u32,
    },
    #[serde(rename = "agent")]
    Agent(AgentMsg),
    #[serde(rename = "cli")]
    Cli {
        verb: String,
        #[serde(default)]
        args: Value,
    },
    #[serde(rename = "card.shown")]
    CardShown { id: String },
    #[serde(rename = "card.answer")]
    CardAnswer(Answer),
    #[serde(rename = "env")]
    Env(Env),
    #[serde(rename = "session.jump")]
    SessionJump { id: String },
    #[serde(rename = "do.result")]
    DoResult {
        id: String,
        #[serde(default)]
        ok: bool,
        #[serde(default)]
        error: Option<String>,
        /// what happened, for the reply line (`{"via": "ewe-browser"}`)
        #[serde(default)]
        data: Value,
    },
    #[serde(rename = "settings")]
    Settings(ChatSettings),
    #[serde(rename = "chat.send")]
    ChatSend { text: String },
    /// an API key typed into the island; it goes straight to the keyring
    #[serde(rename = "key.set")]
    KeySet { provider: String, key: String },
    #[serde(rename = "undo")]
    Undo { token: String },
    /// the home panel opened: read disk and updates again
    #[serde(rename = "glance.refresh")]
    GlanceRefresh,
    /// a quick-action button: one catalog command, validated like any other
    #[serde(rename = "command.run")]
    Run(crate::commands::Call),
    /// a click on a proposed command's Run (`mode: "run"`) or In terminal
    /// (`"terminal"`); the token is the one its `chat.reply` carried
    #[serde(rename = "confirm.run")]
    ConfirmRun {
        token: String,
        #[serde(default)]
        mode: String,
    },
    /// a newer client's message this daemon does not know: ignored
    #[serde(other)]
    Unknown,
}

/// Why a line was refused.
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    TooLong,
    NotJson,
    NewerProtocol(u64),
    Shape(String),
}

/// Parse one line from a client. A missing `v` is read as 1.
pub fn decode(line: &str) -> Result<Inbound, DecodeError> {
    if line.len() > MAX_LINE {
        return Err(DecodeError::TooLong);
    }
    let v: Value = serde_json::from_str(line).map_err(|_| DecodeError::NotJson)?;
    let ver = v.get("v").and_then(Value::as_u64).unwrap_or(PROTOCOL);
    if ver > PROTOCOL {
        return Err(DecodeError::NewerProtocol(ver));
    }
    serde_json::from_value(v).map_err(|e| DecodeError::Shape(e.to_string()))
}

/// One outbound line: `{"v":1,"type":<ty>, …fields}` plus the newline.
pub fn line(ty: &str, fields: Value) -> String {
    let mut obj = json!({ "v": PROTOCOL, "type": ty });
    if let (Some(o), Value::Object(f)) = (obj.as_object_mut(), fields) {
        for (k, val) in f {
            o.insert(k, val);
        }
    }
    let mut s = obj.to_string();
    s.push('\n');
    s
}

/// The daemon's single reply to a waiting hook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Reply {
    /// `allow` · `deny` · `none`
    pub decision: String,
    /// index into the hook's own `permission_suggestions`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub always: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answers: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Reply {
    pub fn none() -> Self {
        Reply {
            decision: "none".into(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_round_trip() {
        let m = decode(r#"{"v":1,"type":"hello","client":"island","pid":42}"#).unwrap();
        assert_eq!(
            m,
            Inbound::Hello {
                client: "island".into(),
                pid: 42
            }
        );
    }

    #[test]
    fn unknown_types_are_ignored_not_errors() {
        assert_eq!(
            decode(r#"{"v":1,"type":"notes.seen","ids":[1]}"#).unwrap(),
            Inbound::Unknown
        );
    }

    #[test]
    fn newer_major_is_refused() {
        assert_eq!(
            decode(r#"{"v":2,"type":"hello"}"#),
            Err(DecodeError::NewerProtocol(2))
        );
    }

    #[test]
    fn garbage_is_refused() {
        assert_eq!(decode("not json"), Err(DecodeError::NotJson));
        assert!(matches!(
            decode(r#"{"type":"card.shown"}"#),
            Err(DecodeError::Shape(_))
        ));
    }

    #[test]
    fn too_long_is_refused() {
        let big = format!(r#"{{"type":"hello","client":"{}"}}"#, "x".repeat(MAX_LINE));
        assert_eq!(decode(&big), Err(DecodeError::TooLong));
    }

    #[test]
    fn agent_message_parses() {
        let m = decode(
            r#"{"v":1,"type":"agent","agent":"claude","event":"Stop","wait":false,"term":{},"payload":{"session_id":"s"}}"#,
        )
        .unwrap();
        match m {
            Inbound::Agent(a) => {
                assert_eq!(a.event, "Stop");
                assert!(!a.wait);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn answer_parses() {
        let m = decode(r#"{"type":"card.answer","id":"c1","action":"always:0"}"#).unwrap();
        assert_eq!(
            m,
            Inbound::CardAnswer(Answer {
                id: "c1".into(),
                action: "always:0".into(),
                answers: None
            })
        );
    }

    #[test]
    fn outbound_line_is_one_line_with_type() {
        let l = line("peek", json!({"key":"k","line":"a\nb"}));
        assert!(l.ends_with('\n'));
        assert_eq!(l.matches('\n').count(), 1);
        let v: Value = serde_json::from_str(l.trim_end()).unwrap();
        assert_eq!(v["type"], "peek");
        assert_eq!(v["v"], 1);
        assert_eq!(v["line"], "a\nb");
    }
}
