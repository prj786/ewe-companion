# Claude Code hook payloads

The relay tests feed these to `ewe-companion hook claude`. They follow the
documented shapes (code.claude.com/docs/en/hooks, 2026-10-14). The
`captured-*.json` files come from a real session: run Claude Code in a
scratch project with `EWE_COMPANION_HOOK_LOG=<file>` set, and the relay
appends every payload it sees. Strip anything personal before committing.
