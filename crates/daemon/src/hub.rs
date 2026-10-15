//! The hub: everything the daemon knows, and the islands it tells.
//!
//! One `std::sync::Mutex` around plain data, never held across an await.
//! Every change that the island draws ends in [`Hub::push_state`], a full
//! snapshot (small: a handful of cards and sessions), so an island that
//! missed a message is right again at the next one.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use companion_core::commands::Call;
use companion_core::id::{new_id, now_ms, now_s};
use companion_core::proto::{self, Answer, Card, ChatSettings, Env, Peek, Session};
use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot};

use crate::store::Store;

/// A card a hook is waiting on: the island's ack, then the answer.
pub struct Wait {
    pub ack: Option<oneshot::Sender<()>>,
    pub answer: Option<oneshot::Sender<Answer>>,
}

#[derive(Default)]
struct Inner {
    islands: HashMap<u64, mpsc::UnboundedSender<String>>,
    cards: Vec<Card>,
    waits: HashMap<String, Wait>,
    sessions: HashMap<String, Session>,
    edited: HashMap<String, HashSet<String>>,
    env: Env,
    // ── the chat (P2.5) ──
    chat: ChatSettings,
    /// providers::status of `chat`, refreshed when the settings or a key change
    chat_status: Value,
    /// the running conversation, (role, text), oldest first
    history: Vec<(String, String)>,
    /// Undo tokens → what puts things back (the last few only)
    undo: Vec<(String, Undo)>,
    /// proposed shell commands waiting for their Run click: token → it
    proposals: Vec<(String, Proposal)>,
    /// `do` requests waiting for the island's `do.result`
    dos: HashMap<String, oneshot::Sender<Result<Value, String>>>,
    /// game mode: Do not disturb as it was before, and the held CPU boost
    game: Option<(bool, Option<tokio::process::Child>)>,
    glance: Value,
    /// Claude plan usage from the status-line relay
    usage: Value,
}

/// Turns of chat kept as context for the model.
const HISTORY: usize = 8;

/// How long a proposed command waits for its Run click.
const PROPOSAL_TTL: std::time::Duration = std::time::Duration::from_secs(15 * 60);

/// What an Undo puts back: a catalog command, or the argv lists ewe
/// browser answered with (its `undo`), run again through the island.
#[derive(Debug, Clone)]
pub enum Undo {
    Call(Call),
    Browser(Vec<Vec<String>>),
}

/// A shell command offered in the chat, waiting for the user's click.
#[derive(Debug, Clone)]
pub struct Proposal {
    pub command: String,
    pub about: String,
    /// it must run in a terminal (a password, a full screen, no end)
    pub terminal: bool,
    pub made: std::time::Instant,
}

pub struct Hub {
    inner: Mutex<Inner>,
    store: Mutex<Store>,
    next: AtomicU64,
    /// `EWE_COMPANION_DEBUG=1`: the `debug.*` verbs (fake cards, answers)
    pub debug: bool,
    /// how long a hook may wait for a human: 108 s, shorter only in debug
    /// runs (`EWE_COMPANION_WAIT_MS`, for the timeout test)
    pub wait: std::time::Duration,
}

/// How long a session stays listed after its last event.
const SESSION_HOURS: i64 = 12;

impl Hub {
    pub fn new(store: Store, debug: bool) -> Hub {
        let mut inner = Inner::default();
        for (s, files) in store.recent_sessions(SESSION_HOURS) {
            inner
                .edited
                .insert(s.id.clone(), files.into_iter().collect());
            inner.sessions.insert(s.id.clone(), s);
        }
        Hub {
            inner: Mutex::new(inner),
            store: Mutex::new(store),
            next: AtomicU64::new(1),
            debug,
            wait: crate::server::WAIT,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }

    pub fn store(&self) -> std::sync::MutexGuard<'_, Store> {
        self.store.lock().unwrap_or_else(|p| p.into_inner())
    }

    // ── islands ──────────────────────────────────────────────────────────

    pub fn add_island(&self) -> (u64, mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.lock().islands.insert(id, tx);
        (id, rx)
    }

    /// The last island gone means nobody can answer: every waiting hook
    /// gets "no opinion" now, so Claude asks in its terminal (C2).
    pub fn remove_island(&self, id: u64) {
        let empty = {
            let mut g = self.lock();
            g.islands.remove(&id);
            g.islands.is_empty()
        };
        if empty {
            self.release_waits("island-gone");
        }
    }

    pub fn island_count(&self) -> usize {
        self.lock().islands.len()
    }

    pub fn broadcast(&self, line: String) {
        let mut g = self.lock();
        g.islands.retain(|_, tx| tx.send(line.clone()).is_ok());
    }

    pub fn snapshot(&self) -> Value {
        let g = self.lock();
        let cut = now_s() - SESSION_HOURS * 3600;
        let mut sessions: Vec<&Session> = g.sessions.values().filter(|s| s.updated > cut).collect();
        sessions.sort_by_key(|s| std::cmp::Reverse(s.updated));
        sessions.truncate(8);
        json!({
            "presence": presence(&g.cards, &sessions),
            "pending": g.cards,
            "sessions": sessions,
            "quiet": {"locked": g.env.locked, "dnd": g.env.dnd},
            "chat": g.chat_status,
            "glance": g.glance,
            "usage": live_usage(&g.usage),
            "game_mode": g.game.is_some(),
        })
    }

    pub fn push_state(&self) {
        let snap = self.snapshot();
        self.broadcast(proto::line("state", snap));
    }

    // ── env ──────────────────────────────────────────────────────────────

    pub fn set_env(&self, env: Env) {
        let now_locked = {
            let mut g = self.lock();
            let was = g.env.locked;
            g.env = env;
            !was && g.env.locked
        };
        // the lock screen must not show commands, and nobody can click:
        // let the terminals ask
        if now_locked {
            self.release_waits("locked");
        }
        self.push_state();
    }

    pub fn env(&self) -> Env {
        self.lock().env.clone()
    }

    // ── cards ────────────────────────────────────────────────────────────

    pub fn add_card(&self, card: Card, wait: Option<Wait>) {
        self.store().card_opened(&card);
        let line = proto::line("card", serde_json::to_value(&card).unwrap_or_default());
        {
            let mut g = self.lock();
            if let Some(w) = wait {
                g.waits.insert(card.id.clone(), w);
            }
            g.cards.push(card);
        }
        self.broadcast(line);
        self.push_state();
    }

    /// Remove a card wherever it is; `false` if it was already gone.
    pub fn close_card(&self, id: &str, reason: &str, action: Option<&str>) -> bool {
        let found = {
            let mut g = self.lock();
            g.waits.remove(id);
            let before = g.cards.len();
            g.cards.retain(|c| c.id != id);
            before != g.cards.len()
        };
        if found {
            self.store().card_closed(id, action, reason);
            self.broadcast(proto::line(
                "card.close",
                json!({"id": id, "reason": reason}),
            ));
            self.push_state();
        }
        found
    }

    pub fn card_shown(&self, id: &str) {
        if let Some(tx) = self.lock().waits.get_mut(id).and_then(|w| w.ack.take()) {
            let _ = tx.send(());
        }
    }

    /// A human's answer. A hook waiting on the card gets it; a card nobody
    /// waits on (a debug card, a note) just closes.
    pub fn card_answer(&self, ans: Answer) {
        let tx = self
            .lock()
            .waits
            .get_mut(&ans.id)
            .and_then(|w| w.answer.take());
        match tx {
            Some(tx) => {
                let _ = tx.send(ans);
            }
            None => {
                let action = ans.action.clone();
                if self.close_card(&ans.id, "answered", Some(&action)) && self.debug {
                    self.peek(Peek {
                        key: "debug".into(),
                        line: format!("Debug card answered: {action}"),
                        level: "info".into(),
                        ..Default::default()
                    });
                }
            }
        }
    }

    /// Give every waiting hook "no opinion" (its card closes with `reason`).
    pub fn release_waits(&self, reason: &str) {
        let waiting: Vec<(String, Wait)> = self.lock().waits.drain().collect();
        for (id, mut w) in waiting {
            if let Some(tx) = w.answer.take() {
                let _ = tx.send(Answer {
                    id: id.clone(),
                    action: reason.into(),
                    answers: None,
                });
            }
            self.close_card(&id, reason, None);
        }
    }

    fn close_session_cards(&self, session: &str, reason: &str) {
        let ids: Vec<String> = self
            .lock()
            .cards
            .iter()
            .filter(|c| c.session.as_deref() == Some(session))
            .map(|c| c.id.clone())
            .collect();
        for id in ids {
            if let Some(tx) = self.lock().waits.get_mut(&id).and_then(|w| w.answer.take()) {
                let _ = tx.send(Answer {
                    id: id.clone(),
                    action: reason.into(),
                    answers: None,
                });
            }
            self.close_card(&id, reason, None);
        }
    }

    // ── peeks and history ────────────────────────────────────────────────

    pub fn peek(&self, p: Peek) {
        self.store()
            .event("companion", "peek", &p.line, "", &p.level, &p.key);
        self.broadcast(proto::line(
            "peek",
            serde_json::to_value(&p).unwrap_or_default(),
        ));
    }

    pub fn say(&self, source: &str, line: &str, level: &str, key: &str, ttl_ms: u64) -> bool {
        let id = self.store().event(source, "say", line, "", level, key);
        let key = if key.is_empty() {
            if id.is_empty() { new_id("e") } else { id }
        } else {
            key.to_string()
        };
        let p = Peek {
            key,
            line: line.into(),
            level: level.into(),
            actions: vec![],
            ttl_ms,
        };
        self.broadcast(proto::line(
            "peek",
            serde_json::to_value(&p).unwrap_or_default(),
        ));
        self.island_count() > 0
    }

    // ── agent sessions ───────────────────────────────────────────────────

    /// Fold one hook event into its session. Returns the session as it is
    /// now and the outcome to announce (`finished`/`failed`), if any.
    pub fn agent_event(
        &self,
        agent: &str,
        event: &str,
        payload: &Value,
        term: &Value,
    ) -> Option<(Session, Option<String>)> {
        let sid = payload
            .get("session_id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())?
            .to_string();
        let u = companion_core::claude::update_for(event, payload);
        let cwd = payload
            .get("cwd")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let now = now_s();
        let (sess, ended, edited_new) = {
            let mut g = self.lock();
            let mut edited_new = None;
            if let Some(f) = &u.edited
                && g.edited.entry(sid.clone()).or_default().insert(f.clone())
            {
                edited_new = Some(f.clone());
            }
            let files = g.edited.get(&sid).map(|s| s.len() as u32).unwrap_or(0);
            let s = g.sessions.entry(sid.clone()).or_insert_with(|| Session {
                id: sid.clone(),
                agent: agent.into(),
                started: now,
                state: "idle".into(),
                ..Default::default()
            });
            if !cwd.is_empty() {
                s.project = companion_core::paths::project_name(&cwd);
                s.cwd = cwd;
            }
            if term.is_object() {
                s.term = term.clone();
            }
            if event == "SessionStart" {
                s.started = now;
            }
            if let Some(st) = u.state {
                s.state = st.into();
            }
            if let Some(l) = &u.last {
                s.last = l.clone();
                if !l.is_empty() && s.steps.last() != Some(l) {
                    s.steps.push(l.clone());
                    if s.steps.len() > 4 {
                        s.steps.remove(0);
                    }
                }
                if event == "SessionStart" {
                    s.steps.clear();
                }
            }
            s.files_changed = files;
            s.updated = now;
            let snap = s.clone();
            if u.ended {
                g.sessions.remove(&sid);
                g.edited.remove(&sid);
            }
            (snap, u.ended, edited_new)
        };
        {
            let st = self.store();
            st.session_put(&sess, ended);
            if let Some(f) = edited_new {
                st.session_edit(&sid, &f, companion_core::claude::tool_name(payload));
            }
        }
        if ended || u.outcome.is_some() {
            // the turn is over: any card still up for it was answered in the terminal
            self.close_session_cards(
                &sid,
                if ended {
                    "agent-gone"
                } else {
                    "answered-elsewhere"
                },
            );
        }
        let line = match u.outcome {
            Some("finished") => Some(finished_line(&sess)),
            Some(_) => Some(match &u.error {
                Some(e) => format!("Claude stopped in {}/ · {e}", sess.project),
                None => format!("Claude stopped in {}/", sess.project),
            }),
            None => None,
        };
        self.push_state();
        Some((sess, line))
    }

    // ── the chat ─────────────────────────────────────────────────────────

    pub fn chat_settings(&self) -> ChatSettings {
        self.lock().chat.clone()
    }

    /// New settings from the island: keep them and say what the island
    /// should show (ready, needs a key, not installed).
    pub async fn set_chat_settings(&self, s: ChatSettings) {
        let status = crate::providers::status(&s).await;
        {
            let mut g = self.lock();
            if g.chat.provider != s.provider {
                g.history.clear(); // a new provider starts a new conversation
            }
            g.chat = s;
            g.chat_status = status;
        }
        self.push_state();
    }

    pub async fn refresh_chat_status(&self) {
        let s = self.chat_settings();
        let status = crate::providers::status(&s).await;
        self.lock().chat_status = status;
        self.push_state();
    }

    pub fn chat_history(&self) -> Vec<(String, String)> {
        self.lock().history.clone()
    }

    pub fn chat_push(&self, user: &str, reply: &str) {
        let mut g = self.lock();
        g.history.push(("user".into(), user.into()));
        g.history.push(("assistant".into(), reply.into()));
        let over = g.history.len().saturating_sub(HISTORY * 2);
        g.history.drain(..over);
    }

    pub fn keep_undo(&self, undo: Undo) -> String {
        let token = new_id("u");
        let mut g = self.lock();
        g.undo.push((token.clone(), undo));
        let over = g.undo.len().saturating_sub(16);
        g.undo.drain(..over);
        token
    }

    pub fn take_undo(&self, token: &str) -> Option<Undo> {
        let mut g = self.lock();
        let i = g.undo.iter().position(|(t, _)| t == token)?;
        Some(g.undo.remove(i).1)
    }

    /// Keep a proposed command until its Run click (the last few, for 15
    /// minutes); the token goes out with the `chat.reply`.
    pub fn keep_proposal(&self, p: Proposal) -> String {
        let token = new_id("r");
        let mut g = self.lock();
        g.proposals.retain(|(_, p)| p.made.elapsed() < PROPOSAL_TTL);
        g.proposals.push((token.clone(), p));
        let over = g.proposals.len().saturating_sub(8);
        g.proposals.drain(..over);
        token
    }

    /// A proposal, once: a second click on the same Run does nothing.
    pub fn take_proposal(&self, token: &str) -> Option<Proposal> {
        let mut g = self.lock();
        let i = g.proposals.iter().position(|(t, _)| t == token)?;
        let (_, p) = g.proposals.remove(i);
        (p.made.elapsed() < PROPOSAL_TTL).then_some(p)
    }

    /// Ask the island to do what only the shell can (Do not disturb, open an
    /// app, a screenshot, the browser), and wait for its `do.result`.
    pub async fn shell_do(&self, action: &str, args: Value) -> Result<Value, String> {
        self.shell_do_within(action, args, std::time::Duration::from_secs(3))
            .await
    }

    /// [`Hub::shell_do`] with a longer wait, for what may start an app
    /// first (ewe browser's command line starts the browser, then acts).
    pub async fn shell_do_within(
        &self,
        action: &str,
        args: Value,
        wait: std::time::Duration,
    ) -> Result<Value, String> {
        if self.island_count() == 0 {
            return Err("The island is not running.".into());
        }
        let id = new_id("q");
        let (tx, rx) = oneshot::channel();
        self.lock().dos.insert(id.clone(), tx);
        self.broadcast(proto::line(
            "do",
            json!({"id": id, "action": action, "args": args}),
        ));
        let r = tokio::time::timeout(wait, rx).await;
        self.lock().dos.remove(&id);
        match r {
            Ok(Ok(res)) => res,
            _ => Err("The island did not answer.".into()),
        }
    }

    pub fn do_result(&self, id: &str, res: Result<Value, String>) {
        if let Some(tx) = self.lock().dos.remove(id) {
            let _ = tx.send(res);
        }
    }

    pub fn game_mode_on(&self) -> bool {
        self.lock().game.is_some()
    }

    /// Set (Some) or end (None) game mode; ending returns Do not disturb as
    /// it was before, and drops (kills) the held CPU boost.
    pub fn set_game_mode(&self, g: Option<(bool, Option<tokio::process::Child>)>) -> bool {
        let prev = {
            let mut inner = self.lock();
            let prev = inner.game.take().map(|(dnd, _boost)| dnd).unwrap_or(false);
            inner.game = g;
            prev
        };
        self.push_state();
        prev
    }

    pub fn set_glance(&self, v: Value) {
        let changed = {
            let mut g = self.lock();
            let changed = g.glance != v;
            g.glance = v;
            changed
        };
        if changed {
            self.push_state();
        }
    }

    pub fn set_usage(&self, agent: &str, limits: &Value) {
        let pick = |k: &str| {
            limits.get(k).and_then(|w| {
                Some(json!({"pct": w.get("used_percentage")?.as_f64()?.round(), "resets_at": w.get("resets_at")?.as_i64()?}))
            })
        };
        let v = json!({"agent": agent, "five_hour": pick("five_hour"), "seven_day": pick("seven_day"), "ts": now_s()});
        let changed = {
            let mut g = self.lock();
            let changed = g.usage.get("five_hour") != v.get("five_hour")
                || g.usage.get("seven_day") != v.get("seven_day");
            g.usage = v;
            changed
        };
        if changed {
            self.push_state();
        }
    }

    pub fn session(&self, id: &str) -> Option<Session> {
        self.lock().sessions.get(id).cloned()
    }

    pub fn sessions(&self) -> Vec<Session> {
        let mut v: Vec<Session> = self.lock().sessions.values().cloned().collect();
        v.sort_by_key(|s| std::cmp::Reverse(s.updated));
        v
    }

    pub fn pending_count(&self) -> usize {
        self.lock().cards.len()
    }

    pub fn status(&self) -> Value {
        let snap = self.snapshot();
        json!({
            "ok": true,
            "version": companion_core::VERSION,
            "island": self.island_count() > 0,
            "islands": self.island_count(),
            "pending": self.pending_count(),
            "sessions": self.lock().sessions.len(),
            "presence": snap["presence"],
            "events": self.store().count("events"),
            "debug": self.debug,
            "ts": now_ms(),
        })
    }
}

/// Plan usage, minus windows that have already reset (Claude Code drops
/// them the same way) and anything older than a day.
fn live_usage(u: &Value) -> Value {
    let now = now_s();
    if u.get("ts")
        .and_then(Value::as_i64)
        .is_none_or(|t| now - t > 86_400)
    {
        return Value::Null;
    }
    let keep = |k: &str| {
        u.get(k)
            .filter(|w| {
                w.get("resets_at")
                    .and_then(Value::as_i64)
                    .is_some_and(|r| r > now)
            })
            .cloned()
    };
    match (keep("five_hour"), keep("seven_day")) {
        (None, None) => Value::Null,
        (f, s) => json!({"agent": u["agent"], "five_hour": f, "seven_day": s}),
    }
}

fn finished_line(s: &Session) -> String {
    match s.files_changed {
        0 => format!("Claude finished in {}/", s.project),
        1 => format!("Claude finished in {}/ · 1 file", s.project),
        n => format!("Claude finished in {}/ · {n} files", s.project),
    }
}

/// What the eyes show: waiting > working > failed > idle.
pub fn presence(cards: &[Card], sessions: &[&Session]) -> &'static str {
    if cards
        .iter()
        .any(|c| c.kind == "permission" || c.kind == "question")
        || sessions.iter().any(|s| s.state == "waiting")
    {
        "waiting"
    } else if sessions.iter().any(|s| s.state == "working") {
        "working"
    } else if sessions.iter().any(|s| s.state == "failed") {
        "error"
    } else {
        "idle"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hub() -> Hub {
        Hub::new(Store::memory(), false)
    }

    #[test]
    fn a_session_counts_each_file_once_and_finishes() {
        let h = hub();
        let p = |ev: &str, extra: Value| {
            let mut v = json!({"session_id": "s1", "cwd": "/w/ewe", "hook_event_name": ev});
            v.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            v
        };
        h.agent_event(
            "claude",
            "UserPromptSubmit",
            &p("UserPromptSubmit", json!({})),
            &Value::Null,
        );
        assert_eq!(
            presence(&[], &h.sessions().iter().collect::<Vec<_>>()),
            "working"
        );
        for f in ["/w/ewe/a", "/w/ewe/b", "/w/ewe/a"] {
            h.agent_event(
                "claude",
                "PostToolUse",
                &p(
                    "PostToolUse",
                    json!({"tool_name": "Edit", "tool_input": {"file_path": f}}),
                ),
                &Value::Null,
            );
        }
        let (s, line) = h
            .agent_event("claude", "Stop", &p("Stop", json!({})), &Value::Null)
            .unwrap();
        assert_eq!(s.files_changed, 2);
        assert_eq!(line.as_deref(), Some("Claude finished in ewe/ · 2 files"));
        h.agent_event(
            "claude",
            "SessionEnd",
            &p("SessionEnd", json!({})),
            &Value::Null,
        );
        assert!(h.sessions().is_empty());
    }

    #[tokio::test]
    async fn the_last_island_leaving_releases_waits() {
        let h = hub();
        let (id, _rx) = h.add_island();
        let (atx, arx) = oneshot::channel();
        let card = Card {
            id: "c1".into(),
            kind: "permission".into(),
            ..Default::default()
        };
        h.add_card(
            card,
            Some(Wait {
                ack: None,
                answer: Some(atx),
            }),
        );
        h.remove_island(id);
        let a = arx.await.unwrap();
        assert_eq!(a.action, "island-gone");
        assert_eq!(h.pending_count(), 0);
    }

    #[tokio::test]
    async fn locking_releases_waits() {
        let h = hub();
        let (_id, _rx) = h.add_island();
        let (atx, arx) = oneshot::channel();
        h.add_card(
            Card {
                id: "c1".into(),
                kind: "permission".into(),
                ..Default::default()
            },
            Some(Wait {
                ack: None,
                answer: Some(atx),
            }),
        );
        h.set_env(Env {
            locked: true,
            ..Default::default()
        });
        assert_eq!(arx.await.unwrap().action, "locked");
    }

    #[test]
    fn islands_get_cards_and_closes() {
        let h = hub();
        let (_id, mut rx) = h.add_island();
        h.add_card(
            Card {
                id: "c9".into(),
                kind: "note".into(),
                ..Default::default()
            },
            None,
        );
        h.card_answer(Answer {
            id: "c9".into(),
            action: "later".into(),
            answers: None,
        });
        let mut types = vec![];
        while let Ok(l) = rx.try_recv() {
            let v: Value = serde_json::from_str(l.trim()).unwrap();
            types.push(v["type"].as_str().unwrap().to_string());
        }
        assert_eq!(types, ["card", "state", "card.close", "state"]);
    }
}
