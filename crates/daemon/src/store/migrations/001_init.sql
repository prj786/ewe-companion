-- 001: the whole PLAN §6 schema. Tables later phases fill (notifications,
-- chats, messages, memory, nudges, usage) exist from the start so a
-- migration never has to rename one.

CREATE TABLE events(
    id TEXT PRIMARY KEY, ts INTEGER NOT NULL, source TEXT NOT NULL, kind TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '', body TEXT NOT NULL DEFAULT '', urgency TEXT NOT NULL DEFAULT 'info',
    key TEXT, privacy TEXT NOT NULL DEFAULT 'local', ref TEXT, shown_level TEXT, handled INTEGER NOT NULL DEFAULT 0);
CREATE INDEX events_ts ON events(ts);
CREATE INDEX events_key ON events(key);

CREATE TABLE notifications(
    id INTEGER PRIMARY KEY, shell_id INTEGER, app TEXT, desktop_entry TEXT, summary TEXT, body TEXT,
    urgency TEXT, category TEXT, ts INTEGER, closed_ts INTEGER, close_reason TEXT,
    invoked TEXT, seen INTEGER NOT NULL DEFAULT 0, privacy TEXT NOT NULL DEFAULT 'ai');

CREATE TABLE sessions(
    id TEXT PRIMARY KEY, agent TEXT NOT NULL, cwd TEXT, started INTEGER, ended INTEGER, state TEXT,
    term TEXT, files_changed INTEGER NOT NULL DEFAULT 0, last_event INTEGER, title TEXT, last TEXT);

CREATE TABLE session_edits(session TEXT NOT NULL, ts INTEGER NOT NULL, file TEXT NOT NULL,
    added INTEGER, removed INTEGER, tool TEXT);
CREATE INDEX session_edits_session ON session_edits(session);

CREATE TABLE cards(id TEXT PRIMARY KEY, kind TEXT NOT NULL, source TEXT NOT NULL, created INTEGER NOT NULL,
    answered INTEGER, action TEXT, reason TEXT, payload TEXT);

CREATE TABLE chats(id TEXT PRIMARY KEY, created INTEGER, title TEXT, provider TEXT);
CREATE TABLE messages(chat TEXT NOT NULL, seq INTEGER NOT NULL, role TEXT, content TEXT, usage TEXT, ts INTEGER,
    PRIMARY KEY(chat, seq));
CREATE TABLE memory(id INTEGER PRIMARY KEY, fact TEXT, source TEXT, created INTEGER, last_used INTEGER);
CREATE TABLE nudges(topic TEXT PRIMARY KEY, last_shown INTEGER, dismissals INTEGER NOT NULL DEFAULT 0,
    muted_until INTEGER, never INTEGER NOT NULL DEFAULT 0);
CREATE TABLE usage(provider TEXT NOT NULL, day TEXT NOT NULL, input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0, cost_usd REAL NOT NULL DEFAULT 0, PRIMARY KEY(provider, day));
