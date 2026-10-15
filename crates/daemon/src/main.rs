//! `ewe-companiond`: the companion's daemon (RFC-007).
//!
//! It holds what must outlive a shell restart: the cards agents wait on,
//! their sessions, and the history. The island (plugin `ewe.companion`)
//! starts it on demand (`systemctl --user start ewe-companion.service`, or
//! straight `ewe-companiond` in dev) and reconnects after a restart. The
//! hooks never start it: no socket means "no opinion" (C2).
//!
//! Logs (`journalctl --user -u ewe-companion`) carry types, ids and timings,
//! never prompt text, commands or answers.

mod browser;
mod chat;
mod commands;
mod context;
mod finder;
mod glance;
mod hub;
mod keyring;
mod places;
mod providers;
mod runner;
mod safe;
mod server;
mod store;
mod term;

use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use companion_core::{VERSION, paths};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

fn main() {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("{{\"ok\":true,\"version\":\"{VERSION}\"}}");
        return;
    }
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("EWE_COMPANION_LOG")
                .unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .without_time() // journald stamps every line
        .with_writer(std::io::stderr)
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stderr()))
        .init();
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!("no runtime: {e}");
            std::process::exit(1);
        }
    };
    std::process::exit(rt.block_on(run()));
}

async fn run() -> i32 {
    let Some(sock) = paths::socket_path() else {
        error!("no private XDG_RUNTIME_DIR: refusing to open a socket");
        return 1;
    };
    // someone answers on the socket already: that daemon keeps it
    if UnixStream::connect(&sock).await.is_ok() {
        info!("another ewe-companiond is running; leaving it be");
        return 0;
    }
    let _ = std::fs::remove_file(&sock); // a stale file from a crash

    let dir = paths::state_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        error!("state dir {}: {e}", dir.display());
        return 1;
    }
    let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
    let store = match store::Store::open(&dir.join("companion.db")) {
        Ok(s) => s,
        Err(e) => {
            error!("store: {e}");
            return 1;
        }
    };
    let debug_verbs = std::env::var("EWE_COMPANION_DEBUG").as_deref() == Ok("1");
    let mut hub = hub::Hub::new(store, debug_verbs);
    if debug_verbs
        && let Some(ms) = std::env::var("EWE_COMPANION_WAIT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
    {
        hub.wait = std::time::Duration::from_millis(ms.clamp(500, 108_000));
    }
    let hub = Arc::new(hub);

    // the system glance: now, then every ten minutes
    {
        let hub = hub.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(600));
            loop {
                tick.tick().await;
                hub.set_glance(glance::read());
            }
        });
    }

    // 0600 from birth: bind under a umask, then chmod to be sure
    // SAFETY: umask only changes this process's file-creation mask.
    let old = unsafe { libc::umask(0o177) };
    let listener = UnixListener::bind(&sock);
    unsafe { libc::umask(old) };
    let listener = match listener {
        Ok(l) => l,
        Err(e) => {
            error!("bind {}: {e}", sock.display());
            return 1;
        }
    };
    let _ = std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o600));
    info!(
        version = VERSION,
        debug_verbs,
        "listening on {}",
        sock.display()
    );

    let mut term = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Ok(s) => s,
        Err(e) => {
            error!("signals: {e}");
            return 1;
        }
    };
    loop {
        tokio::select! {
            r = listener.accept() => match r {
                Ok((stream, _)) => { tokio::spawn(server::handle(hub.clone(), stream)); }
                Err(e) => warn!("accept: {e}"),
            },
            _ = term.recv() => break,
            _ = tokio::signal::ctrl_c() => break,
        }
    }
    // waiting hooks get "no opinion" now rather than a dead socket later
    hub.release_waits("daemon-stopping");
    let _ = std::fs::remove_file(&sock);
    info!("stopped");
    0
}
