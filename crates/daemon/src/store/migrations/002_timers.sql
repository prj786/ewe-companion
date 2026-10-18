-- 002 (wave 2): timers and reminders. A row waits until it is due; the
-- daemon then shows a notification and deletes the row. Kept here, not in
-- memory, so a timer survives a daemon restart (one that fell due while
-- the daemon was down fires at its next start, saying when it was due).
-- `due` and `created` are epoch milliseconds; `seconds` is what a timer
-- was set for (null for a reminder at a time of day); `kind` is `timer`
-- or `reminder` ("remind me in 5 minutes" has no words of its own);
-- `text` is the user's own words ("call mum"), never logged.

CREATE TABLE timers(
    id TEXT PRIMARY KEY, due INTEGER NOT NULL, created INTEGER NOT NULL,
    seconds INTEGER, kind TEXT NOT NULL DEFAULT 'timer', text TEXT NOT NULL DEFAULT '');
CREATE INDEX timers_due ON timers(due);
