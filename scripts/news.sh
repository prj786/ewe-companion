#!/usr/bin/env bash
# scripts/news.sh: which ewe releases knowledge/news.md has no topic for,
# with their release notes, so a person (or an agent) can write the plain
# words. The notes are written by hand on purpose: release notes are for
# developers ("vendor the plugin, bump the ledger"), a news topic is for the
# user ("ewe Player plays YouTube links").
#
#   scripts/news.sh            the missing releases' notes, and a skeleton to paste
#   scripts/news.sh --check    one JSON object: {"ok": true|false, "missing": [...]}
#
# Needs `gh` (signed in) for prj786/ewe. One topic per minor release, the
# title exactly "What's new in ewe X.Y" (Pack::news parses it; a patch
# release folds into its minor), keys only version spellings ("ewe 0.46,
# version 0.46"): the `whats_new` command picks the topic for the ewe
# installed. Then `cargo test -p companion-core news` and recipe 4 of
# docs/EXTENDING.md.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NEWS="$ROOT/knowledge/news.md"
REPO="${EWE_REPO:-prj786/ewe}"
LIMIT="${NEWS_LIMIT:-12}"

have=$(grep -oP "^## What's new in ewe \K[0-9]+\.[0-9]+" "$NEWS" | sort -uV)
# releases older than the oldest topic are not owed one
oldest=$(head -n 1 <<<"$have")
tags=$(gh release list -R "$REPO" -L "$LIMIT" --json tagName -q '.[].tagName')
missing=()
for t in $tags; do
  minor=$(echo "$t" | grep -oP '^v?\K[0-9]+\.[0-9]+')
  [ -z "$minor" ] && continue
  [ "$(printf '%s\n%s\n' "$oldest" "$minor" | sort -V | head -n 1)" = "$minor" ] && [ "$minor" != "$oldest" ] && continue
  if ! grep -qx "$minor" <<<"$have" && ! printf '%s\n' "${missing[@]:-}" | grep -qx "$minor"; then
    missing+=("$minor")
  fi
done

if [ "${1:-}" = "--check" ]; then
  python3 -c 'import json,sys; m=[x for x in sys.argv[1:] if x]; print(json.dumps({"ok": not m, "missing": m}))' "${missing[@]:-}"
  exit 0
fi

if [ "${#missing[@]}" -eq 0 ] || [ -z "${missing[0]:-}" ]; then
  echo "news.md has a topic for each of the last $LIMIT releases."
  exit 0
fi
for m in "${missing[@]}"; do
  echo "================ ewe $m ================"
  for t in $tags; do
    case "$t" in
      v"$m".*|"$m".*) gh release view "$t" -R "$REPO" --json tagName,publishedAt,body \
                        -q '"# " + .tagName + " (" + .publishedAt[0:10] + ")\n" + .body'; echo ;;
    esac
  done
  cat <<EOF
---- paste into knowledge/news.md (newest first), in plain words ----
## What's new in ewe $m
keys: ewe $m, version $m, $m update
Released <day month year> as ewe $m.0-beta, with <the apps that came with it>. <3–6 short sentences: what a user will notice, and where to find it.>

EOF
done
