@AGENTS.md

Claude-specific notes:

- Start with `AGENTS.md` (above), then the guide for the change at hand:
  `docs/EXTENDING.md` for building, `docs/TESTING.md` for checking.
- Most features cross into `../ewe-plugin-companion`; read its `AGENTS.md`
  before a QML edit.
- Keep the island harness's files in your scratchpad, and give the work
  dir on **every** call (shell variables do not survive between Bash
  calls): `ISLAND_WORK=<scratchpad>/island tests/island.sh up --scale 1.8`,
  then the same prefix on `ctl`, `shot`, `down` …. Read every screenshot you
  take before you report on it, and `down` when done.
- Run `cargo build` before `tests/chat.sh` and `tests/relay-failsafe.sh`:
  they run `target/debug` as it is.
- The scratch daemon acts on the real machine: never test volume, mute,
  brightness, media, game mode or lock (it locks the owner's screen); never
  press Run on anything but a harmless `echo`.
- Never point a test at the owner's real `~/.claude`, and never restart
  `ewe.service` or `ewe-companion.service` unless the owner asked to try a
  change: their shell restarts in front of them.
- Commit or push only when asked; branch off `main` first. End commit
  messages with the Co-Authored-By line the session gives you.
- When something lands, tick it in `docs/PLAN.md` §20, add what it taught
  to §25 and to *Traps* in `AGENTS.md`, and update the vault (then run
  `../project-map/.vault/vault-check.sh --diagrams`).
