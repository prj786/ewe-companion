-- 003 (Shaun gets wiser): what the user asked Shaun to remember ("remember
-- that my dentist is on friday"). One row per memory, the user's own words
-- in `text` (never a secret: memory.rs refuses what looks like one), at
-- most 100 (memory.rs). `created` is epoch milliseconds. The memories go to
-- the model's prompt (the user asked for that); logs carry ids and counts
-- only. The `memory` table of 001 (PLAN §6's facts with a source) stays for
-- a later phase.

CREATE TABLE memories(
    id TEXT PRIMARY KEY, text TEXT NOT NULL, created INTEGER NOT NULL);
CREATE INDEX memories_created ON memories(created);
