//! API keys live in the Secret Service keyring (gnome-keyring, unlocked by
//! PAM at login), never in `ewe.conf`, which syncs to the cloud (rule 2).
//!
//! Service `ewe-companion`, attribute `provider <id>`, through `secret-tool`
//! with the key on **stdin**, never on a command line. Nothing here logs a
//! key, and nothing hands one to a model: a key only ever goes into the
//! `Authorization` header of its own provider.

use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const SERVICE: &str = "ewe-companion";
const BUDGET: Duration = Duration::from_secs(5);

pub async fn get(provider: &str) -> Option<String> {
    let mut c = Command::new("secret-tool");
    c.args(["lookup", "service", SERVICE, "provider", provider])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let out = tokio::time::timeout(BUDGET, c.output()).await.ok()?.ok()?;
    let key = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (out.status.success() && !key.is_empty()).then_some(key)
}

pub async fn set(provider: &str, key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() || key.len() > 4096 || key.contains('\n') {
        return Err("That does not look like an API key.".into());
    }
    let label = format!("ewe companion: {provider} API key");
    let mut child = Command::new("secret-tool")
        .args([
            "store", "--label", &label, "service", SERVICE, "provider", provider,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| "secret-tool is missing (package libsecret).".to_string())?;
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    stdin
        .write_all(key.as_bytes())
        .await
        .map_err(|_| "Could not write to the keyring.".to_string())?;
    drop(stdin);
    match tokio::time::timeout(BUDGET, child.wait()).await {
        Ok(Ok(s)) if s.success() => Ok(()),
        _ => Err("The keyring did not take the key. Is it unlocked?".into()),
    }
}

pub async fn clear(provider: &str) -> bool {
    let mut c = Command::new("secret-tool");
    c.args(["clear", "service", SERVICE, "provider", provider])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    matches!(tokio::time::timeout(BUDGET, c.status()).await, Ok(Ok(s)) if s.success())
}
