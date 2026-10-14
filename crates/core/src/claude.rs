//! Claude Code's hook dialect (PLAN §10.1; code.claude.com/docs/en/hooks,
//! checked against Claude Code 2.1.296 on 2026-10-14).
//!
//! Three jobs, all pure:
//!
//! 1. [`waits`]: which events hold Claude until a human answers
//!    (`PermissionRequest`, and `PreToolUse` for `AskUserQuestion`).
//! 2. [`card_for`]: the card such an event becomes.
//! 3. [`render`]: the exact stdout JSON for the daemon's [`Reply`], or
//!    `None` for "no opinion" (print nothing, exit 0, and Claude asks in its
//!    own terminal; C2).
//!
//! [`render`] never produces an allow the reply did not carry, and "Always"
//! only ever echoes one of Claude's own `permission_suggestions` back: the
//! companion stores no permission rules of its own.

use serde_json::{Value, json};

use crate::paths::project_name;
use crate::proto::{Action, Card, Reply};
use crate::trim::cut;

pub const ASK_TOOL: &str = "AskUserQuestion";
pub const DENY_MESSAGE: &str = "Denied from the ewe companion";

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

pub fn tool_name(payload: &Value) -> &str {
    s(payload, "tool_name")
}

/// Does this event hold Claude until someone answers?
pub fn waits(event: &str, payload: &Value) -> bool {
    match event {
        "PermissionRequest" => true,
        "PreToolUse" => tool_name(payload) == ASK_TOOL,
        _ => false,
    }
}

fn suggestions(payload: &Value) -> &[Value] {
    payload
        .get("permission_suggestions")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn rel(path: &str, cwd: &str) -> String {
    let cwd = cwd.trim_end_matches('/');
    match path.strip_prefix(cwd).and_then(|r| r.strip_prefix('/')) {
        Some(r) if !cwd.is_empty() => r.to_string(),
        _ => path.to_string(),
    }
}

fn base(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(path)
}

/// `mcp__github__create_issue` → (`github`, `create_issue`)
fn mcp_parts(tool: &str) -> Option<(&str, &str)> {
    let rest = tool.strip_prefix("mcp__")?;
    rest.split_once("__")
}

/// The card's title and main line, plus the preview the card renders.
fn describe_tool(tool: &str, input: &Value, cwd: &str) -> (String, String, Value) {
    match tool {
        "Bash" => {
            let cmd = s(input, "command");
            (
                "Claude wants to run".into(),
                cut(cmd, 400),
                json!({"kind": "command", "text": cmd, "description": s(input, "description")}),
            )
        }
        "Edit" | "MultiEdit" | "Write" | "NotebookEdit" => {
            let file = [s(input, "file_path"), s(input, "notebook_path")]
                .into_iter()
                .find(|f| !f.is_empty())
                .unwrap_or("");
            let verb = if tool == "Write" { "write" } else { "edit" };
            let edits: Vec<Value> = match tool {
                "Write" => vec![json!({"old": "", "new": s(input, "content")})],
                "NotebookEdit" => vec![json!({"old": "", "new": s(input, "new_source")})],
                "MultiEdit" => input
                    .get("edits")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .map(|e| json!({"old": s(e, "old_string"), "new": s(e, "new_string")}))
                            .collect()
                    })
                    .unwrap_or_default(),
                _ => vec![json!({"old": s(input, "old_string"), "new": s(input, "new_string")})],
            };
            (
                format!("Claude wants to {verb} {}", base(file)),
                rel(file, cwd),
                json!({"kind": "diff", "file": rel(file, cwd), "edits": edits}),
            )
        }
        "WebFetch" => {
            let url = s(input, "url");
            (
                "Claude wants to fetch a page".into(),
                cut(url, 400),
                json!({"kind": "text", "text": url}),
            )
        }
        "WebSearch" => {
            let q = s(input, "query");
            (
                "Claude wants to search the web".into(),
                cut(q, 400),
                json!({"kind": "text", "text": q}),
            )
        }
        _ => {
            let title = match mcp_parts(tool) {
                Some((server, t)) => format!("Claude wants to use {t} ({server})"),
                None => format!("Claude wants to use {tool}"),
            };
            let pretty = serde_json::to_string_pretty(input).unwrap_or_default();
            let one = input.to_string();
            (
                title,
                cut(&one, 400),
                json!({"kind": "json", "text": cut(&pretty, 4000)}),
            )
        }
    }
}

/// A rule in words: `Bash(npm install:*)` → `npm install`.
fn rule_words(rule: &Value) -> String {
    let tool = s(rule, "toolName");
    let content = s(rule, "ruleContent").trim();
    let content = content
        .trim_end_matches(":*")
        .trim_end_matches(" *")
        .trim_end_matches('*')
        .trim();
    let tool_word = mcp_parts(tool).map(|(_, t)| t).unwrap_or(tool);
    if content.is_empty() {
        return tool_word.to_string();
    }
    match tool {
        "Bash" => content.to_string(),
        "WebFetch" => content
            .strip_prefix("domain:")
            .unwrap_or(content)
            .to_string(),
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" | "Read" => {
            format!("{} in {content}", tool_word.to_lowercase())
        }
        _ => format!("{tool_word} {content}"),
    }
}

fn where_words(destination: &str, project: &str) -> String {
    match destination {
        "session" => "This session only".into(),
        "localSettings" => format!("Saved for {project}/, just for you"),
        "projectSettings" => format!("Saved for {project}/, shared with the project"),
        "userSettings" => "Saved for every project".into(),
        _ => String::new(),
    }
}

/// Claude's own suggestion in words, as (button label, hint). `None` for
/// suggestion shapes the island does not offer (the terminal still does).
pub fn describe_suggestion(sugg: &Value, project: &str) -> Option<(String, String)> {
    let dest = s(sugg, "destination");
    let label = match s(sugg, "type") {
        "addRules" if s(sugg, "behavior") == "allow" => {
            let rules = sugg.get("rules")?.as_array()?;
            if rules.is_empty() {
                return None;
            }
            let words: Vec<String> = rules.iter().map(rule_words).collect();
            format!("Always for {}", words.join(", "))
        }
        "addDirectories" => {
            let dirs = sugg.get("directories")?.as_array()?;
            let names: Vec<&str> = dirs.iter().filter_map(Value::as_str).map(base).collect();
            if names.is_empty() {
                return None;
            }
            format!("Always allow {}/", names.join("/, "))
        }
        "setMode" if s(sugg, "mode") == "acceptEdits" => "Accept all edits".into(),
        _ => return None,
    };
    Some((cut(&label, 60), where_words(dest, project)))
}

/// The card a waiting event becomes. `None`: nothing the island can ask.
pub fn card_for(event: &str, payload: &Value, id: &str, now_ms: u64, wait_ms: u64) -> Option<Card> {
    let cwd = s(payload, "cwd");
    let project = project_name(cwd);
    let session = Some(s(payload, "session_id").to_string()).filter(|x| !x.is_empty());
    let mut card = Card {
        id: id.into(),
        source: "agent:claude".into(),
        project: project.clone(),
        session,
        created_ms: now_ms,
        deadline_ms: now_ms + wait_ms,
        ..Default::default()
    };
    match event {
        "PermissionRequest" => {
            let tool = tool_name(payload);
            let (title, body, detail) =
                describe_tool(tool, payload.get("tool_input").unwrap_or(&Value::Null), cwd);
            let mut actions = vec![
                Action::new("allow", "Allow", "primary"),
                Action::new("deny", "Deny", "secondary"),
            ];
            for (i, sugg) in suggestions(payload).iter().enumerate() {
                if let Some((label, hint)) = describe_suggestion(sugg, &project) {
                    actions.push(Action {
                        id: format!("always:{i}"),
                        label,
                        style: "ghost".into(),
                        hint,
                    });
                }
            }
            actions.push(Action::new("terminal", "Answer in terminal", "ghost"));
            card.kind = "permission".into();
            card.title = title;
            card.body = body;
            card.actions = actions;
            card.payload = json!({"tool": tool, "detail": detail});
            Some(card)
        }
        "PreToolUse" if tool_name(payload) == ASK_TOOL => {
            let questions = payload.get("tool_input")?.get("questions")?.as_array()?;
            if questions.is_empty() || questions.len() > 4 {
                return None;
            }
            let qs: Vec<Value> = questions
                .iter()
                .map(|q| {
                    json!({
                        "question": s(q, "question"),
                        "header": s(q, "header"),
                        "multiSelect": q.get("multiSelect").and_then(Value::as_bool).unwrap_or(false),
                        "options": q.get("options").and_then(Value::as_array).map(|o| o.iter().map(|o| json!({
                            "label": s(o, "label"), "description": s(o, "description")
                        })).collect::<Vec<_>>()).unwrap_or_default(),
                    })
                })
                .collect();
            card.kind = "question".into();
            card.title = if qs.len() == 1 {
                "Claude asks".into()
            } else {
                format!("Claude asks {} things", qs.len())
            };
            card.body = s(&questions[0], "question").to_string();
            card.actions = vec![Action::new("terminal", "Answer in terminal", "ghost")];
            card.payload = json!({"questions": qs});
            Some(card)
        }
        _ => None,
    }
}

/// Check the island's answers against the questions Claude asked: every
/// question answered, every label one of its options. Multi-select answers
/// arrive as arrays and leave joined with commas (the documented form).
fn valid_answers(questions: &[Value], answers: &Value) -> Option<Value> {
    let given = answers.as_object()?;
    let mut out = serde_json::Map::new();
    for q in questions {
        let text = s(q, "question");
        let labels: Vec<&str> = q
            .get("options")?
            .as_array()?
            .iter()
            .map(|o| s(o, "label"))
            .collect();
        let multi = q
            .get("multiSelect")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let picked: Vec<&str> = match given.get(text)? {
            Value::String(one) => vec![one.as_str()],
            Value::Array(many) if multi => many.iter().filter_map(Value::as_str).collect(),
            _ => return None,
        };
        if picked.is_empty()
            || (!multi && picked.len() != 1)
            || !picked.iter().all(|p| labels.contains(p))
        {
            return None;
        }
        out.insert(text.to_string(), Value::String(picked.join(", ")));
    }
    Some(Value::Object(out))
}

/// The hook's stdout for a reply, or `None` = print nothing (no opinion).
pub fn render(event: &str, payload: &Value, reply: &Reply) -> Option<String> {
    let out = match (event, reply.decision.as_str()) {
        ("PermissionRequest", "allow") => {
            let mut decision = json!({"behavior": "allow"});
            if let Some(i) = reply.always {
                // an index Claude did not offer is not an allow of any kind
                let sugg = suggestions(payload).get(i)?.clone();
                decision["updatedPermissions"] = json!([sugg]);
            }
            json!({"hookSpecificOutput": {"hookEventName": "PermissionRequest", "decision": decision}})
        }
        ("PermissionRequest", "deny") => {
            let msg = reply
                .message
                .clone()
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| DENY_MESSAGE.into());
            json!({"hookSpecificOutput": {"hookEventName": "PermissionRequest",
                   "decision": {"behavior": "deny", "message": msg}}})
        }
        ("PreToolUse", "allow") if tool_name(payload) == ASK_TOOL => {
            let input = payload.get("tool_input")?;
            let questions = input.get("questions")?.as_array()?;
            let answers = valid_answers(questions, reply.answers.as_ref()?)?;
            let mut updated = input.clone();
            updated.as_object_mut()?.insert("answers".into(), answers);
            json!({"hookSpecificOutput": {"hookEventName": "PreToolUse",
                   "permissionDecision": "allow", "updatedInput": updated}})
        }
        _ => return None,
    };
    Some(out.to_string())
}

/// What a non-waiting (or waiting) event changes about its session.
#[derive(Debug, Default, PartialEq)]
pub struct Update {
    pub state: Option<&'static str>,
    pub last: Option<String>,
    /// a file this event edited (counted once per session)
    pub edited: Option<String>,
    /// `finished` or `failed`: the daemon peeks
    pub outcome: Option<&'static str>,
    pub error: Option<String>,
    pub ended: bool,
}

/// `Edit Bar.qml`, `Bash npm test`, `Read docs/PLAN.md`
pub fn tool_summary(tool: &str, input: &Value, cwd: &str) -> String {
    let file = [
        s(input, "file_path"),
        s(input, "notebook_path"),
        s(input, "path"),
    ]
    .into_iter()
    .find(|f| !f.is_empty());
    let what = match tool {
        "Bash" => cut(s(input, "command").lines().next().unwrap_or(""), 60),
        "Grep" | "Glob" => cut(s(input, "pattern"), 60),
        "WebFetch" => cut(s(input, "url"), 60),
        "WebSearch" => cut(s(input, "query"), 60),
        _ => file.map(|f| rel(f, cwd)).unwrap_or_default(),
    };
    let tool = mcp_parts(tool).map(|(_, t)| t).unwrap_or(tool);
    if what.is_empty() {
        tool.to_string()
    } else {
        format!("{tool} {what}")
    }
}

const EDIT_TOOLS: &[&str] = &["Edit", "MultiEdit", "Write", "NotebookEdit"];

pub fn update_for(event: &str, payload: &Value) -> Update {
    let cwd = s(payload, "cwd");
    let tool = tool_name(payload);
    let input = payload.get("tool_input").unwrap_or(&Value::Null);
    let mut u = Update::default();
    match event {
        "SessionStart" => {
            u.state = Some("idle");
            u.last = Some(String::new());
        }
        "UserPromptSubmit" => u.state = Some("working"),
        "PermissionRequest" => {
            u.state = Some("waiting");
            u.last = Some(format!("Waiting · {}", tool_summary(tool, input, cwd)));
        }
        "PreToolUse" if tool == ASK_TOOL => {
            u.state = Some("waiting");
            u.last = Some("Asking a question".into());
        }
        "PreToolUse" | "PostToolUse" => {
            u.state = Some("working");
            u.last = Some(tool_summary(tool, input, cwd));
            if event == "PostToolUse" && EDIT_TOOLS.contains(&tool) {
                let f = [s(input, "file_path"), s(input, "notebook_path")]
                    .into_iter()
                    .find(|f| !f.is_empty());
                u.edited = f.map(str::to_string);
            }
        }
        "PostToolUseFailure" => {
            u.state = Some("working");
            u.last = Some(format!("Failed · {}", tool_summary(tool, input, cwd)));
        }
        "Notification" => match s(payload, "notification_type") {
            "idle_prompt" => u.state = Some("idle"),
            "permission_prompt" | "agent_needs_input" | "elicitation_dialog" => {
                u.state = Some("waiting")
            }
            _ => {}
        },
        "SubagentStart" => u.last = Some(format!("Subagent · {}", s(payload, "agent_type"))),
        "Stop" => {
            u.state = Some("done");
            u.outcome = Some("finished");
        }
        "StopFailure" => {
            u.state = Some("failed");
            u.outcome = Some("failed");
            let e = [s(payload, "last_assistant_message"), s(payload, "error")]
                .into_iter()
                .find(|e| !e.is_empty());
            u.error = e.map(|e| cut(e, 120));
        }
        "SessionEnd" => u.ended = true,
        _ => {}
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bash_request() -> Value {
        json!({
            "session_id": "abc123", "cwd": "/home/u/Projects/ewe-browser",
            "permission_mode": "default", "hook_event_name": "PermissionRequest",
            "tool_name": "Bash", "tool_input": {"command": "npm install", "description": "Install"},
            "permission_suggestions": [{
                "type": "addRules", "rules": [{"toolName": "Bash", "ruleContent": "npm install:*"}],
                "behavior": "allow", "destination": "localSettings"
            }]
        })
    }

    fn question() -> Value {
        json!({
            "session_id": "abc123", "cwd": "/home/u/p", "hook_event_name": "PreToolUse",
            "tool_name": "AskUserQuestion",
            "tool_input": {"questions": [
                {"question": "Which framework?", "header": "Framework", "multiSelect": false,
                 "options": [{"label": "React", "description": "Component library"}, {"label": "Vue", "description": "Progressive"}]},
                {"question": "Which extras?", "header": "Extras", "multiSelect": true,
                 "options": [{"label": "Router", "description": ""}, {"label": "Store", "description": ""}, {"label": "Tests", "description": ""}]}
            ]}
        })
    }

    fn reply(decision: &str) -> Reply {
        Reply {
            decision: decision.into(),
            ..Default::default()
        }
    }

    #[test]
    fn what_waits() {
        assert!(waits("PermissionRequest", &bash_request()));
        assert!(waits("PreToolUse", &question()));
        assert!(!waits("PreToolUse", &json!({"tool_name": "Bash"})));
        assert!(!waits("Stop", &json!({})));
    }

    #[test]
    fn permission_card_reads_like_the_house_voice() {
        let c = card_for("PermissionRequest", &bash_request(), "c1", 1000, 108_000).unwrap();
        assert_eq!(c.kind, "permission");
        assert_eq!(c.title, "Claude wants to run");
        assert_eq!(c.body, "npm install");
        assert_eq!(c.project, "ewe-browser");
        assert_eq!(c.session.as_deref(), Some("abc123"));
        assert_eq!(c.deadline_ms, 109_000);
        let ids: Vec<&str> = c.actions.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, ["allow", "deny", "always:0", "terminal"]);
        assert_eq!(c.actions[2].label, "Always for npm install");
        assert_eq!(c.actions[2].hint, "Saved for ewe-browser/, just for you");
    }

    #[test]
    fn edit_card_previews_the_diff() {
        let p = json!({"cwd": "/w/ewe", "tool_name": "Edit",
            "tool_input": {"file_path": "/w/ewe/dotfiles/Bar.qml", "old_string": "a", "new_string": "b"}});
        let c = card_for("PermissionRequest", &p, "c2", 0, 1).unwrap();
        assert_eq!(c.title, "Claude wants to edit Bar.qml");
        assert_eq!(c.body, "dotfiles/Bar.qml");
        assert_eq!(c.payload["detail"]["kind"], "diff");
        assert_eq!(c.payload["detail"]["edits"][0]["new"], "b");
        // no suggestions: no Always button
        assert!(!c.actions.iter().any(|a| a.id.starts_with("always")));
    }

    #[test]
    fn mcp_tools_name_their_server() {
        let p = json!({"cwd": "/w", "tool_name": "mcp__github__create_issue", "tool_input": {"title": "x"}});
        let c = card_for("PermissionRequest", &p, "c3", 0, 1).unwrap();
        assert_eq!(c.title, "Claude wants to use create_issue (github)");
    }

    #[test]
    fn allow_maps_exactly() {
        let out = render("PermissionRequest", &bash_request(), &reply("allow")).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            v,
            json!({"hookSpecificOutput": {"hookEventName": "PermissionRequest", "decision": {"behavior": "allow"}}})
        );
    }

    #[test]
    fn always_echoes_claudes_own_suggestion() {
        let r = Reply {
            decision: "allow".into(),
            always: Some(0),
            ..Default::default()
        };
        let v: Value =
            serde_json::from_str(&render("PermissionRequest", &bash_request(), &r).unwrap())
                .unwrap();
        assert_eq!(
            v["hookSpecificOutput"]["decision"]["updatedPermissions"][0],
            bash_request()["permission_suggestions"][0]
        );
    }

    #[test]
    fn an_always_claude_did_not_offer_is_no_opinion() {
        let r = Reply {
            decision: "allow".into(),
            always: Some(5),
            ..Default::default()
        };
        assert_eq!(render("PermissionRequest", &bash_request(), &r), None);
    }

    #[test]
    fn deny_carries_a_message() {
        let v: Value = serde_json::from_str(
            &render("PermissionRequest", &bash_request(), &reply("deny")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            v["hookSpecificOutput"]["decision"],
            json!({"behavior": "deny", "message": DENY_MESSAGE})
        );
    }

    #[test]
    fn none_and_unknown_decisions_print_nothing() {
        assert_eq!(
            render("PermissionRequest", &bash_request(), &reply("none")),
            None
        );
        assert_eq!(
            render("PermissionRequest", &bash_request(), &reply("maybe")),
            None
        );
        assert_eq!(render("Stop", &json!({}), &reply("allow")), None);
        // a plain Bash PreToolUse is never answered by us
        assert_eq!(
            render("PreToolUse", &json!({"tool_name": "Bash"}), &reply("allow")),
            None
        );
    }

    #[test]
    fn question_answers_go_into_updated_input() {
        let r = Reply {
            decision: "allow".into(),
            answers: Some(json!({"Which framework?": "Vue", "Which extras?": ["Router", "Tests"]})),
            ..Default::default()
        };
        let v: Value =
            serde_json::from_str(&render("PreToolUse", &question(), &r).unwrap()).unwrap();
        let h = &v["hookSpecificOutput"];
        assert_eq!(h["hookEventName"], "PreToolUse");
        assert_eq!(h["permissionDecision"], "allow");
        assert_eq!(
            h["updatedInput"]["questions"],
            question()["tool_input"]["questions"]
        );
        assert_eq!(
            h["updatedInput"]["answers"],
            json!({"Which framework?": "Vue", "Which extras?": "Router, Tests"})
        );
    }

    #[test]
    fn bad_answers_are_no_opinion() {
        let bad = [
            json!({"Which framework?": "Svelte", "Which extras?": ["Router"]}), // not an option
            json!({"Which framework?": "Vue"}),                                 // one unanswered
            json!({"Which framework?": ["Vue", "React"], "Which extras?": ["Router"]}), // multi on single
            json!("Vue"),
        ];
        for a in bad {
            let r = Reply {
                decision: "allow".into(),
                answers: Some(a.clone()),
                ..Default::default()
            };
            assert_eq!(render("PreToolUse", &question(), &r), None, "{a}");
        }
    }

    #[test]
    fn question_card_lists_the_questions() {
        let c = card_for("PreToolUse", &question(), "q1", 0, 1).unwrap();
        assert_eq!(c.kind, "question");
        assert_eq!(c.title, "Claude asks 2 things");
        assert_eq!(c.payload["questions"][1]["multiSelect"], true);
        assert_eq!(c.payload["questions"][0]["options"][1]["label"], "Vue");
    }

    #[test]
    fn session_updates() {
        let edit =
            json!({"cwd": "/w", "tool_name": "Edit", "tool_input": {"file_path": "/w/a/B.qml"}});
        let u = update_for("PostToolUse", &edit);
        assert_eq!(u.state, Some("working"));
        assert_eq!(u.last.as_deref(), Some("Edit a/B.qml"));
        assert_eq!(u.edited.as_deref(), Some("/w/a/B.qml"));
        assert_eq!(update_for("Stop", &json!({})).outcome, Some("finished"));
        assert_eq!(
            update_for("Notification", &json!({"notification_type": "idle_prompt"})).state,
            Some("idle")
        );
        assert!(update_for("SessionEnd", &json!({})).ended);
        let f = update_for(
            "StopFailure",
            &json!({"error": "rate_limit", "last_assistant_message": "API Error: Rate limit reached"}),
        );
        assert_eq!(f.error.as_deref(), Some("API Error: Rate limit reached"));
    }

    /// Contract test: a payload captured from Claude Code 2.1.296 (2026-10-14,
    /// tests/payloads/claude/README.md). If Claude changes the shape, this
    /// fails before a user's prompt does.
    #[test]
    fn captured_permission_request() {
        let p: Value = serde_json::from_str(include_str!(
            "../../../tests/payloads/claude/captured-permission-request-bash.json"
        ))
        .unwrap();
        assert!(waits("PermissionRequest", &p));
        let c = card_for("PermissionRequest", &p, "c", 0, 1).unwrap();
        assert_eq!(c.title, "Claude wants to run");
        assert_eq!(c.body, "touch made-by-claude.txt");
        assert_eq!(c.project, "scratch");
        let labels: Vec<&str> = c.actions.iter().map(|a| a.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Allow",
                "Deny",
                "Always allow scratch/",
                "Accept all edits",
                "Answer in terminal"
            ]
        );
        assert_eq!(c.actions[2].hint, "This session only");
        let r = Reply {
            decision: "allow".into(),
            always: Some(1),
            ..Default::default()
        };
        let v: Value = serde_json::from_str(&render("PermissionRequest", &p, &r).unwrap()).unwrap();
        assert_eq!(
            v["hookSpecificOutput"]["decision"]["updatedPermissions"][0]["mode"],
            "acceptEdits"
        );
    }

    #[test]
    fn captured_session_events() {
        let stop: Value = serde_json::from_str(include_str!(
            "../../../tests/payloads/claude/captured-stop.json"
        ))
        .unwrap();
        assert_eq!(update_for("Stop", &stop).outcome, Some("finished"));
        let post: Value = serde_json::from_str(include_str!(
            "../../../tests/payloads/claude/captured-post-tool-use-bash.json"
        ))
        .unwrap();
        assert_eq!(
            update_for("PostToolUse", &post).last.as_deref(),
            Some("Bash touch made-by-claude.txt")
        );
        let end: Value = serde_json::from_str(include_str!(
            "../../../tests/payloads/claude/captured-session-end.json"
        ))
        .unwrap();
        assert!(update_for("SessionEnd", &end).ended);
    }

    #[test]
    fn suggestion_words() {
        let p = "ewe";
        let d = |v: Value| describe_suggestion(&v, p).map(|(l, _)| l);
        assert_eq!(
            d(json!({"type": "setMode", "mode": "acceptEdits", "destination": "session"}))
                .as_deref(),
            Some("Accept all edits")
        );
        assert_eq!(
            d(json!({"type": "setMode", "mode": "bypassPermissions"})),
            None
        );
        assert_eq!(
            d(json!({"type": "addRules", "behavior": "allow", "rules": [{"toolName": "WebFetch", "ruleContent": "domain:github.com"}]})).as_deref(),
            Some("Always for github.com")
        );
        assert_eq!(
            d(json!({"type": "addRules", "behavior": "deny", "rules": [{"toolName": "Bash"}]})),
            None
        );
        assert_eq!(
            d(json!({"type": "addDirectories", "directories": ["/home/u/other"], "destination": "session"})).as_deref(),
            Some("Always allow other/")
        );
    }
}
