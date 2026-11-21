#!/usr/bin/env python3
"""Write knowledge/coverage.json: everything ewe ships that Shaun must know.

The unit test `every_shipped_feature_has_a_topic` (crates/core/src/knowledge.rs)
reads the list and fails the build when an item has no topic whose title or
keys name it. That is what keeps the pack from going stale: a new Settings
page, `ewe-*` tool, first-party app or built-in plugin without a topic is a
red build, not a quiet gap.

Sources (the repos win over the vault, the vault over memory):

  ewe-settings  src/lib/pages.js            every Settings page (its label)
  ewe           bin/*, packaging/ewe-setup   every ewe-* tool
  ewe           plugins/bundle.json + plugins/<id>/manifest.json
                                             every built-in plugin (its name)
  project-map   ewe-facts.json               cli_tools (tools that live in other
                                             repos: komble, ewe-safe, ewe-player …)
                                             and repos (first-party apps)

Usage, from the repo root (siblings found from the workspace):

  python3 scripts/coverage.py                       # write knowledge/coverage.json
  python3 scripts/coverage.py --check               # exit 1 if the file is out of date
  python3 scripts/coverage.py --facts /tmp/facts.json --ewe ../ewe

The vault's own checkout may lag its origin: for the newest facts use
`git -C <workspace>/project-map show origin/main:ewe-facts.json > /tmp/facts.json`
and pass `--facts /tmp/facts.json`. After writing, `cargo test -p companion-core
every_shipped_feature` says which items still need a topic (knowledge/*.md).
"""

import argparse
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
OUT = os.path.join(REPO, "knowledge", "coverage.json")

# A first-party app per repo of ewe-facts.json `repos`, as people name it.
# A repo that is neither here nor in NOT_APPS stops the script: decide.
APPS = {
    "ewe-settings": ["ewe Settings"],
    "komble-arch": ["Komble"],
    "ewe-sync": ["ewe Sync"],
    "ewe-browser": ["ewe Browser"],
    "ewe-apps": ["ewe Files", "ewe Player", "ewe Viewer"],
    "ewe-safe": ["ewe Safe"],
    "ewe-companion": ["Shaun"],
    "ewe-cast": ["Cast to TV"],
}
NOT_APPS = {"ewe", "ewe-os", "ewe-repo", "prj786.github.io"}

# Names that people say differently from the label: the extra words a topic
# may use instead (any one of the terms is enough).
ALIASES = {
    "Clipboard & emoji": ["clipboard"],
    "Cast to TV": ["cast"],
    "Window switcher": ["window switcher", "alt tab"],
    "ewe Safe": ["ewe safe", "safe"],
    "Music": ["music plugin", "music"],
    "About this computer": ["about this computer", "version"],
    "Time and place": ["time and place", "time zone"],
}

# Tools nobody types (a portal's window, a hook): still listed, so a topic
# says what they are when someone meets the name.
TOOL_SKIP = {"__pycache__"}


def workspace():
    d = REPO
    for _ in range(6):
        if os.path.isdir(os.path.join(d, "ewe", "bin")) and os.path.isdir(
            os.path.join(d, "ewe-settings")
        ):
            return d
        d = os.path.dirname(d)
    sys.exit("coverage.py: no workspace with ewe/ and ewe-settings/ above this repo; pass --ewe, --settings, --facts")


def settings_pages(settings_dir):
    text = open(os.path.join(settings_dir, "src", "lib", "pages.js"), encoding="utf-8").read()
    out = []
    for m in re.finditer(r'\{\s*id:\s*"([a-z0-9-]+)",\s*label:\s*"([^"]+)"', text):
        out.append({"kind": "settings page", "name": "ewe Settings → " + m.group(2), "page": m.group(1),
                    "terms": ALIASES.get(m.group(2), [m.group(2)])})
    if not out:
        sys.exit("coverage.py: no pages in pages.js (its format changed?)")
    return out


def tools(ewe_dir, facts):
    names = set()
    for n in os.listdir(os.path.join(ewe_dir, "bin")):
        if n not in TOOL_SKIP and not n.startswith("."):
            names.add(n)
    if os.path.exists(os.path.join(ewe_dir, "packaging", "ewe-setup")):
        names.add("ewe-setup")
    for t in facts.get("cli_tools", []):
        for n in str(t.get("tool", "")).split("|"):
            n = n.strip()
            if n:
                names.add(n)
    return [{"kind": "command-line tool", "name": n, "terms": [n]} for n in sorted(names)]


def apps(facts):
    out = []
    for r in facts.get("repos", []):
        name = r.get("name", "")
        if name.startswith("ewe-plugin-") or name in NOT_APPS:
            continue
        if name not in APPS:
            sys.exit("coverage.py: repo %r is new: add it to APPS (an app people use) or NOT_APPS" % name)
        for a in APPS[name]:
            out.append({"kind": "app", "name": a, "repo": name, "terms": ALIASES.get(a, [a])})
    return out


def plugins(ewe_dir):
    bundle = json.load(open(os.path.join(ewe_dir, "plugins", "bundle.json"), encoding="utf-8"))
    out = []
    for pid in sorted(bundle.get("plugins", {})):
        name = pid
        mf = os.path.join(ewe_dir, "plugins", pid, "manifest.json")
        if os.path.exists(mf):
            name = json.load(open(mf, encoding="utf-8")).get("name", pid)
        out.append({"kind": "plugin", "name": name, "id": pid,
                    "terms": ALIASES.get(name, [name])})
    return out


def main():
    ws = None
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--ewe")
    ap.add_argument("--settings")
    ap.add_argument("--facts")
    ap.add_argument("--check", action="store_true", help="fail if knowledge/coverage.json differs")
    a = ap.parse_args()
    if not (a.ewe and a.settings and a.facts):
        ws = workspace()
    ewe_dir = a.ewe or os.path.join(ws, "ewe")
    settings_dir = a.settings or os.path.join(ws, "ewe-settings")
    facts_path = a.facts or os.path.join(ws, "project-map", "ewe-facts.json")
    facts = json.load(open(facts_path, encoding="utf-8"))
    items = settings_pages(settings_dir) + tools(ewe_dir, facts) + apps(facts) + plugins(ewe_dir)
    doc = {
        "about": "Everything ewe ships that Shaun must have a topic for. Written by scripts/coverage.py; "
                 "checked by the unit test every_shipped_feature_has_a_topic. Do not edit by hand: "
                 "rerun the script (docs/EXTENDING.md, recipe 4).",
        "facts_generated": facts.get("generated", ""),
        "ewe_version": facts.get("versions", {}).get("ewe_de", ""),
        "items": items,
    }
    text = json.dumps(doc, indent=1, ensure_ascii=False) + "\n"
    if a.check:
        old = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if old != text:
            print(json.dumps({"ok": False, "error": "knowledge/coverage.json is out of date: run scripts/coverage.py"}))
            sys.exit(1)
        print(json.dumps({"ok": True, "items": len(items)}))
        return
    open(OUT, "w", encoding="utf-8").write(text)
    kinds = {}
    for i in items:
        kinds[i["kind"]] = kinds.get(i["kind"], 0) + 1
    print(json.dumps({"ok": True, "items": len(items), "kinds": kinds, "file": os.path.relpath(OUT, REPO)}))


if __name__ == "__main__":
    main()
