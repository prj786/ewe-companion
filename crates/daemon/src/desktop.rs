//! The desktop's switches, the way ewe's own panels flip them:
//!
//! - **Wi-Fi**: `nmcli radio wifi on|off`, read with `nmcli -t -f WIFI
//!   radio` (Quick settings' own toggle, `QuickSettings.qml`);
//! - **Bluetooth**: `ewe-bt power on|off`, read with `ewe-bt status`, and a
//!   paired device by its name: `ewe-bt status` finds the address, then
//!   `ewe-bt connect|disconnect <MAC>` (ewe-bt takes addresses only);
//! - **airplane mode**: Wi-Fi and Bluetooth off, remembering how they were
//!   (ewe has no airplane switch of its own); off puts them back;
//! - **keeping the screen awake**: the Insomnia plugin, `qs ipc call
//!   ewe.insomnia on|off|status` (it holds a Wayland idle inhibitor, so
//!   the dim, the lock and the suspend on battery wait);
//!
//! each T1 with Undo (decision 6), read before the change. Device names
//! are the user's: a line that names one stays on the island (`private`).
//!
//! And the T2/T3 ones, which never run from here: dark or light mode
//! (`ewe-theme scheme apply`), suspend, restart, shut down and log out
//! (ewe's own `~/.config/hypr/scripts/power.sh`, what Quick settings' Power
//! menu runs) and emptying the trash (`gio trash --empty`) come back as a
//! proposal: the exact command, with Run.
//!
//! Tests run all of these against PATH shims (`tests/fakes/`): never the
//! owner's real radios, power or shell.

use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use companion_core::commands::Call;
use serde_json::{Value, json};
use tokio::process::Command;

use crate::commands::{Outcome, done, failed, missed, run_argv};

/// Connecting a Bluetooth device can take a while.
const BT_CONNECT: Duration = Duration::from_secs(20);
const READ: Duration = Duration::from_secs(5);

/// An ewe tool's one JSON object (the last line it printed), found on PATH
/// or in ~/.local/bin.
async fn tool_json(name: &str, args: &[&str], budget: Duration) -> Result<Value, String> {
    let Some(bin) = crate::context::tool(name) else {
        return Err(format!("{name} is not installed."));
    };
    let mut c = Command::new(bin);
    c.args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    match tokio::time::timeout(budget, c.output()).await {
        Ok(Ok(o)) => crate::browser::last_json(&String::from_utf8_lossy(&o.stdout))
            .ok_or_else(|| format!("{name} did not answer.")),
        Ok(Err(_)) => Err(format!("{name} is not installed.")),
        Err(_) => Err(format!("{name} did not answer in time.")),
    }
}

/// ewe-bt's reply, or its own sentence for what went wrong ("No Bluetooth
/// adapter found", "bluetooth.service is not running").
async fn bt(args: &[&str], budget: Duration) -> Result<Value, String> {
    let v = tool_json("ewe-bt", args, budget).await?;
    if v["ok"] == true {
        Ok(v)
    } else {
        Err(match v["error"].as_str() {
            Some(e) if !e.is_empty() => format!("Bluetooth: {e}."),
            _ => "Bluetooth did not answer.".into(),
        })
    }
}

// ── Wi-Fi ───────────────────────────────────────────────────────────────────

async fn wifi_on() -> Option<bool> {
    match run_argv("nmcli", &["-t", "-f", "WIFI", "radio"], "")
        .await
        .ok()?
        .trim()
    {
        "enabled" => Some(true),
        "disabled" => Some(false),
        _ => None,
    }
}

async fn set_wifi(on: bool) -> Result<(), String> {
    run_argv(
        "nmcli",
        &["radio", "wifi", if on { "on" } else { "off" }],
        "",
    )
    .await
    .map(|_| ())
    .map_err(|_| "Could not change Wi-Fi (nmcli).".to_string())
}

pub async fn wifi(on: bool) -> Outcome {
    let was = wifi_on().await;
    if was == Some(on) {
        return done(
            if on {
                "Wi-Fi is already on"
            } else {
                "Wi-Fi is already off"
            },
            None,
        );
    }
    match set_wifi(on).await {
        Ok(()) => done(
            if on { "Wi-Fi on" } else { "Wi-Fi off" },
            was.map(|w| Call::new("wifi", json!({"on": w}))),
        ),
        Err(e) => failed(e),
    }
}

// ── Bluetooth ───────────────────────────────────────────────────────────────

/// Is the adapter on? Err when there is none, or bluez is not running.
async fn bt_on() -> Result<bool, String> {
    let v = bt(&["status"], READ).await?;
    match v["adapter"]["powered"].as_bool() {
        Some(p) => Ok(p),
        None => Err("There is no Bluetooth adapter here.".into()),
    }
}

async fn set_bt(on: bool) -> Result<(), String> {
    bt(&["power", if on { "on" } else { "off" }], READ)
        .await
        .map(|_| ())
}

pub async fn bluetooth(on: bool) -> Outcome {
    let was = match bt_on().await {
        Ok(w) => w,
        Err(e) => return failed(e),
    };
    if was == on {
        return done(
            if on {
                "Bluetooth is already on"
            } else {
                "Bluetooth is already off"
            },
            None,
        );
    }
    match set_bt(on).await {
        Ok(()) => done(
            if on { "Bluetooth on" } else { "Bluetooth off" },
            Some(Call::new("bluetooth", json!({"on": was}))),
        ),
        Err(e) => failed(e),
    }
}

/// Every word in the device's name ("sony headphones" fits "Sony
/// WH-1000XM4 headphones"); the words "my", "the" and "bluetooth" count
/// for nothing.
fn device_fits(name: &str, words: &str) -> bool {
    let hay = name.to_lowercase();
    let ws: Vec<String> = words
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|w| !matches!(w.as_str(), "my" | "the" | "bluetooth" | "device"))
        .collect();
    !ws.is_empty() && ws.iter().all(|w| hay.contains(w.trim_end_matches("'s")))
}

fn device_row(d: &Value, connect: bool) -> Value {
    let mac = d["address"].as_str().unwrap_or("");
    let name = d["name"].as_str().unwrap_or(mac);
    let detail = if d["connected"] == true {
        "connected"
    } else {
        "paired"
    };
    json!({"title": name, "detail": detail, "icon": "bluetooth",
           "actions": [{"label": if connect { "Connect" } else { "Disconnect" },
                        "command": "bluetooth_device", "args": {"mac": mac, "connect": connect}}]})
}

/// Connect or disconnect a paired device: by its address (a row's click)
/// or by words in its name. Several that fit become rows.
pub async fn bluetooth_device(a: &Value) -> Outcome {
    let connect = a["connect"].as_bool().unwrap_or(true);
    let status = match bt(&["status"], READ).await {
        Ok(v) => v,
        Err(e) => return failed(e),
    };
    if status["adapter"].is_null() {
        return failed("There is no Bluetooth adapter here.");
    }
    if status["adapter"]["powered"] == false && connect {
        return Outcome {
            action: Some((
                "Turn on Bluetooth".into(),
                Call::new("bluetooth", json!({"on": true})),
            )),
            ..failed("Bluetooth is off.")
        };
    }
    let devices: Vec<Value> = status["devices"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|d| d["paired"] == true)
        .collect();
    let found: Vec<&Value> = match a.get("mac").and_then(Value::as_str) {
        Some(mac) => devices
            .iter()
            .filter(|d| {
                d["address"]
                    .as_str()
                    .is_some_and(|m| m.eq_ignore_ascii_case(mac))
            })
            .collect(),
        None => {
            let words = a["name"].as_str().unwrap_or("");
            devices
                .iter()
                .filter(|d| device_fits(d["name"].as_str().unwrap_or(""), words))
                .collect()
        }
    };
    let d = match found.as_slice() {
        [] => {
            return missed(match a.get("name").and_then(Value::as_str) {
                Some(w) => format!("No paired Bluetooth device is called {w}."),
                None => "That device is not paired any more.".into(),
            })
            .private("No paired Bluetooth device fits.");
        }
        [one] => *one,
        many => {
            return Outcome {
                rows: many.iter().map(|d| device_row(d, connect)).collect(),
                ..done(
                    format!("{} paired devices fit. Which one?", many.len()),
                    None,
                )
            }
            .private(format!(
                "{} Bluetooth devices fit; the user picks one on the island.",
                many.len()
            ));
        }
    };
    let mac = d["address"].as_str().unwrap_or("").to_string();
    let name = d["name"].as_str().unwrap_or(&mac).to_string();
    if (d["connected"] == true) == connect {
        let line = if connect {
            format!("{name} is already connected")
        } else {
            format!("{name} is not connected")
        };
        return done(line, None).private("Nothing to change.");
    }
    let verb = if connect { "connect" } else { "disconnect" };
    match bt(&[verb, &mac], BT_CONNECT).await {
        Ok(_) => done(
            if connect {
                format!("Connected {name}")
            } else {
                format!("Disconnected {name}")
            },
            Some(Call::new(
                "bluetooth_device",
                json!({"mac": mac, "connect": !connect}),
            )),
        )
        .private(if connect {
            "Connected the Bluetooth device."
        } else {
            "Disconnected the Bluetooth device."
        }),
        Err(e) => failed(format!("Could not {verb} {name}. {e}"))
            .private(format!("Could not {verb} the Bluetooth device.")),
    }
}

// ── airplane mode ───────────────────────────────────────────────────────────

/// How Wi-Fi and Bluetooth were before airplane mode, for "airplane mode
/// off" said later (the Undo carries them itself). In memory: after a
/// daemon restart, off turns both on.
fn before() -> &'static Mutex<Option<(bool, bool)>> {
    static B: OnceLock<Mutex<Option<(bool, bool)>>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(None))
}

pub async fn airplane(a: &Value) -> Outcome {
    let on = a["on"].as_bool().unwrap_or(true);
    let wifi_was = wifi_on().await;
    // no adapter, or no bluez: airplane mode is Wi-Fi alone
    let bt_was = bt_on().await.ok();
    if on {
        if wifi_was != Some(true) && bt_was != Some(true) {
            return done(
                "Airplane mode is already on: Wi-Fi and Bluetooth are off",
                None,
            );
        }
        let mut errs = vec![];
        // what this call actually turned off
        let w = wifi_was == Some(true)
            && match set_wifi(false).await {
                Ok(()) => true,
                Err(e) => {
                    errs.push(e);
                    false
                }
            };
        let b = bt_was == Some(true)
            && match set_bt(false).await {
                Ok(()) => true,
                Err(e) => {
                    errs.push(e);
                    false
                }
            };
        // airplane mode already on (something came back on since): add to
        // what it turned off before, never forget that
        let (w, b) = {
            let mut kept = before().lock().unwrap_or_else(|p| p.into_inner());
            let (w, b) = match *kept {
                Some((kw, kb)) => (kw || w, kb || b),
                None => (w, b),
            };
            if w || b {
                *kept = Some((w, b));
            }
            (w, b)
        };
        let undo = (w || b).then(|| {
            Call::new(
                "airplane_mode",
                json!({"on": false, "wifi": w, "bluetooth": b}),
            )
        });
        if !errs.is_empty() {
            // half done: what did go off comes back with Undo
            return Outcome {
                undo,
                ..failed(errs.join(" "))
            };
        }
        done("Airplane mode on · Wi-Fi and Bluetooth off", undo)
    } else {
        let kept = before().lock().unwrap_or_else(|p| p.into_inner()).take();
        let (w, b) = match (a["wifi"].as_bool(), a["bluetooth"].as_bool(), kept) {
            (Some(w), Some(b), _) => (w, b),
            (_, _, Some(k)) => k,
            _ => (true, bt_was.is_some()),
        };
        let mut errs = vec![];
        let mut back = vec![];
        if w && wifi_was != Some(true) {
            match set_wifi(true).await {
                Ok(()) => back.push("Wi-Fi"),
                Err(e) => errs.push(e),
            }
        }
        if b && bt_was == Some(false) {
            match set_bt(true).await {
                Ok(()) => back.push("Bluetooth"),
                Err(e) => errs.push(e),
            }
        }
        if !errs.is_empty() {
            return failed(errs.join(" "));
        }
        done(
            match back.as_slice() {
                [] => "Airplane mode off".to_string(),
                b => format!("Airplane mode off · {} on", b.join(" and ")),
            },
            None,
        )
    }
}

// ── keeping the screen awake (the Insomnia plugin) ─────────────────────────

/// `qs ipc call ewe.insomnia <fn>`, on the shell the island runs in. Err
/// when the shell or the plugin is not there.
async fn insomnia(f: &str) -> Result<String, String> {
    run_argv("qs", &["ipc", "call", "ewe.insomnia", f], "")
        .await
        .map_err(|_| "Keeping the screen awake needs the Insomnia plugin.".to_string())
}

pub async fn keep_awake(on: bool) -> Outcome {
    // the plugin's own answer, `{"on": …}`; anything else means it is not
    // there (or the shell is not)
    let was = insomnia("status")
        .await
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(s.trim()).ok())
        .and_then(|v| v["on"].as_bool());
    let Some(was) = was else {
        return Outcome {
            action: Some((
                "Open Plugins".into(),
                Call::new("open", json!({"what": "komble", "page": "plugins"})),
            )),
            ..failed(
                "Keeping the screen awake needs the Insomnia plugin: add it in Komble → Plugins → From ewe.",
            )
        };
    };
    if was == on {
        return done(
            if on {
                "The screen already stays awake"
            } else {
                "The screen already sleeps as usual"
            },
            None,
        );
    }
    match insomnia(if on { "on" } else { "off" }).await {
        Ok(_) => done(
            if on {
                "The screen stays awake until you say \"keep awake off\""
            } else {
                "The screen sleeps as usual again"
            },
            Some(Call::new("keep_awake", json!({"on": was}))),
        ),
        Err(e) => failed(e),
    }
}

// ── proposals: shown with Run, never run from here ─────────────────────────

fn propose(line: impl Into<String>, command: &str, about: &str) -> Outcome {
    Outcome {
        ok: true,
        line: line.into(),
        propose: Some((command.to_string(), about.to_string())),
        ..Default::default()
    }
}

/// How many things are in the trash (`~/.local/share/Trash/files`), read
/// off the async threads (a trash can hold a great many).
async fn trash_count() -> usize {
    let dir = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .map(|d| d.join("Trash/files"));
    tokio::task::spawn_blocking(move || {
        dir.and_then(|d| std::fs::read_dir(d).ok())
            .map(|r| r.count())
            .unwrap_or(0)
    })
    .await
    .unwrap_or(0)
}

pub async fn proposal(call: &Call) -> Outcome {
    let a = &call.args;
    match call.command.as_str() {
        "theme_mode" => {
            let mode = a["mode"].as_str().unwrap_or("dark");
            // the scheme on now, and whether it is already that variant
            if let Ok(v) = tool_json("ewe-theme", &["scheme", "list"], READ).await
                && let Some(cur) = v["schemes"]
                    .as_array()
                    .and_then(|s| s.iter().find(|s| s["current"] == true))
                && cur["variant"].as_str() == Some(mode)
            {
                let name = cur["name"].as_str().unwrap_or(mode);
                return done(format!("{name} is on: the desktop is already {mode}"), None);
            }
            let slug = if mode == "light" {
                "ewe-light"
            } else {
                "ewe-dark"
            };
            propose(
                format!(
                    "Switch the whole desktop to {mode} mode? This does it (Settings → Appearance → Scheme):"
                ),
                &format!("ewe-theme scheme apply {slug}"),
                &format!("Switch to {mode} mode"),
            )
        }
        "power" => {
            let (arg, line, about) = match a["action"].as_str().unwrap_or("") {
                "restart" => (
                    "reboot",
                    "Restart this computer? Save your work first.",
                    "Restart",
                ),
                "shutdown" => (
                    "poweroff",
                    "Shut down this computer? Save your work first.",
                    "Shut down",
                ),
                "logout" => ("logout", "Log out? Open apps close.", "Log out"),
                _ => ("suspend", "Suspend this computer?", "Suspend"),
            };
            propose(
                line,
                &format!("~/.config/hypr/scripts/power.sh {arg}"),
                about,
            )
        }
        _ => match trash_count().await {
            0 => done("The trash is already empty", None),
            n => propose(
                if n == 1 {
                    "Empty the trash? The 1 item in it goes for good.".to_string()
                } else {
                    format!("Empty the trash? The {n} items in it go for good.")
                },
                "gio trash --empty",
                "Empty the trash",
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_device_fits_every_word() {
        assert!(device_fits("Sony WH-1000XM4", "sony"));
        assert!(device_fits("Sam's AirPods Pro", "my airpods"));
        assert!(device_fits("Sam's AirPods Pro", "sam's airpods"));
        assert!(!device_fits("Sony WH-1000XM4", "airpods"));
        assert!(!device_fits("Sony WH-1000XM4", "the bluetooth"));
    }
}
