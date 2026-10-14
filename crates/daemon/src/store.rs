//! The store: `~/.local/state/ewe/companion/companion.db` (PLAN §6).
//!
//! SQLite in WAL mode, per machine, never synced. Migrations are numbered
//! files compiled in; `PRAGMA user_version` records the last one applied.
//! Logs never carry what is stored here, only ids and counts.

use std::path::Path;

use companion_core::id::now_s;
use companion_core::proto::{Card, Session};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

const MIGRATIONS: &[&str] = &[include_str!("store/migrations/001_init.sql")];

/// How long events and cards are kept (PLAN §6 retention; a setting later).
const KEEP_DAYS: i64 = 30;

pub struct Store {
    db: Connection,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Store> {
        let db = Connection::open(path)?;
        Self::init(db)
    }

    #[cfg(test)]
    pub fn memory() -> Store {
        Self::init(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn init(db: Connection) -> rusqlite::Result<Store> {
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "NORMAL")?;
        db.busy_timeout(std::time::Duration::from_secs(2))?;
        let have: usize =
            db.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))? as usize;
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(have) {
            db.execute_batch(&format!(
                "BEGIN; {sql}; PRAGMA user_version = {}; COMMIT;",
                i + 1
            ))?;
        }
        let store = Store { db };
        store.prune();
        Ok(store)
    }

    fn prune(&self) {
        let cut = now_s() - KEEP_DAYS * 86_400;
        let _ = self
            .db
            .execute("DELETE FROM events WHERE ts < ?1", params![cut]);
        let _ = self
            .db
            .execute("DELETE FROM cards WHERE created < ?1", params![cut * 1000]);
        let _ = self.db.execute(
            "DELETE FROM session_edits WHERE ts < ?1",
            params![now_s() - 14 * 86_400],
        );
    }

    /// One history line; returns its id. The same `key` within a day
    /// updates the line instead of adding another (and returns "").
    pub fn event(
        &self,
        source: &str,
        kind: &str,
        title: &str,
        body: &str,
        urgency: &str,
        key: &str,
    ) -> String {
        let ts = now_s();
        if !key.is_empty() {
            let n = self
                .db
                .execute(
                    "UPDATE events SET ts=?1, title=?2, body=?3, urgency=?4 WHERE key=?5 AND ts > ?6",
                    params![ts, title, body, urgency, key, ts - 86_400],
                )
                .unwrap_or(0);
            if n > 0 {
                return String::new();
            }
        }
        let id = companion_core::id::new_id("e");
        let _ = self.db.execute(
            "INSERT INTO events(id, ts, source, kind, title, body, urgency, key) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![id, ts, source, kind, title, body, urgency, (!key.is_empty()).then_some(key)],
        );
        id
    }

    pub fn history(&self, limit: u64) -> Vec<Value> {
        let mut st = match self
            .db
            .prepare("SELECT id, ts, source, kind, title, body, urgency FROM events ORDER BY ts DESC, rowid DESC LIMIT ?1")
        {
            Ok(s) => s,
            Err(_) => return vec![],
        };
        st.query_map(params![limit.min(500) as i64], |r| {
            Ok(json!({
                "id": r.get::<_, String>(0)?, "ts": r.get::<_, i64>(1)?, "source": r.get::<_, String>(2)?,
                "kind": r.get::<_, String>(3)?, "title": r.get::<_, String>(4)?, "body": r.get::<_, String>(5)?,
                "urgency": r.get::<_, String>(6)?,
            }))
        })
        .map(|rows| rows.flatten().collect())
        .unwrap_or_default()
    }

    pub fn card_opened(&self, c: &Card) {
        let _ = self.db.execute(
            "INSERT OR REPLACE INTO cards(id, kind, source, created, payload) VALUES(?1,?2,?3,?4,?5)",
            params![c.id, c.kind, c.source, c.created_ms as i64, json!({"title": c.title, "project": c.project}).to_string()],
        );
    }

    pub fn card_closed(&self, id: &str, action: Option<&str>, reason: &str) {
        let _ = self.db.execute(
            "UPDATE cards SET answered=?1, action=?2, reason=?3 WHERE id=?4",
            params![companion_core::id::now_ms() as i64, action, reason, id],
        );
    }

    pub fn session_put(&self, s: &Session, ended: bool) {
        let _ = self.db.execute(
            "INSERT INTO sessions(id, agent, cwd, started, ended, state, term, files_changed, last_event, last)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
             ON CONFLICT(id) DO UPDATE SET cwd=excluded.cwd, ended=excluded.ended, state=excluded.state,
               term=excluded.term, files_changed=excluded.files_changed, last_event=excluded.last_event,
               last=excluded.last",
            params![
                s.id, s.agent, s.cwd, s.started, ended.then_some(s.updated), s.state, s.term.to_string(),
                s.files_changed, s.updated, s.last
            ],
        );
    }

    pub fn session_edit(&self, session: &str, file: &str, tool: &str) {
        let _ = self.db.execute(
            "INSERT INTO session_edits(session, ts, file, tool) VALUES(?1,?2,?3,?4)",
            params![session, now_s(), file, tool],
        );
    }

    /// Sessions that were live within `hours`, for a daemon restart.
    pub fn recent_sessions(&self, hours: i64) -> Vec<(Session, Vec<String>)> {
        let cut = now_s() - hours * 3600;
        let Ok(mut st) = self.db.prepare(
            "SELECT id, agent, cwd, started, state, term, files_changed, last_event, last FROM sessions
             WHERE ended IS NULL AND last_event > ?1 ORDER BY last_event DESC LIMIT 32",
        ) else {
            return vec![];
        };
        let rows: Vec<Session> = st
            .query_map(params![cut], |r| {
                let cwd: String = r.get::<_, Option<String>>(2)?.unwrap_or_default();
                Ok(Session {
                    id: r.get(0)?,
                    agent: r.get(1)?,
                    project: companion_core::paths::project_name(&cwd),
                    cwd,
                    started: r.get::<_, Option<i64>>(3)?.unwrap_or(0),
                    state: r
                        .get::<_, Option<String>>(4)?
                        .unwrap_or_else(|| "idle".into()),
                    term: r
                        .get::<_, Option<String>>(5)?
                        .and_then(|t| serde_json::from_str(&t).ok())
                        .unwrap_or(Value::Null),
                    files_changed: r.get::<_, i64>(6)? as u32,
                    updated: r.get::<_, Option<i64>>(7)?.unwrap_or(0),
                    last: r.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    steps: Vec::new(),
                })
            })
            .map(|rows| rows.flatten().collect())
            .unwrap_or_default();
        rows.into_iter()
            .map(|s| {
                let files = self
                    .db
                    .prepare("SELECT DISTINCT file FROM session_edits WHERE session=?1")
                    .and_then(|mut st| {
                        st.query_map(params![s.id], |r| r.get::<_, String>(0))
                            .map(|r| r.flatten().collect())
                    })
                    .unwrap_or_default();
                (s, files)
            })
            .collect()
    }

    pub fn count(&self, table: &str) -> i64 {
        // table names come from our own code, never from a client
        self.db
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_once() {
        let s = Store::memory();
        let v: i64 =
            s.db.pragma_query_value(None, "user_version", |r| r.get(0))
                .unwrap();
        assert_eq!(v as usize, MIGRATIONS.len());
        assert_eq!(s.count("events"), 0);
    }

    #[test]
    fn keyed_events_update_in_place() {
        let s = Store::memory();
        s.event("cli", "say", "one", "", "info", "k");
        s.event("cli", "say", "two", "", "info", "k");
        s.event("cli", "say", "three", "", "info", "");
        let h = s.history(10);
        assert_eq!(h.len(), 2);
        assert!(h.iter().any(|e| e["title"] == "two"));
    }

    #[test]
    fn sessions_survive_a_restart() {
        let s = Store::memory();
        let sess = Session {
            id: "s1".into(),
            agent: "claude".into(),
            cwd: "/w/ewe".into(),
            project: "ewe".into(),
            state: "working".into(),
            updated: now_s(),
            started: now_s(),
            ..Default::default()
        };
        s.session_put(&sess, false);
        s.session_edit("s1", "/w/ewe/a.qml", "Edit");
        let back = s.recent_sessions(12);
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].0.project, "ewe");
        assert_eq!(back[0].1, vec!["/w/ewe/a.qml".to_string()]);
        s.session_put(&sess, true);
        assert!(s.recent_sessions(12).is_empty());
    }
}
