//! companion-core — what the daemon, the CLI and the hook relay share.
//!
//! - [`proto`]: the socket protocol (docs/PROTOCOL.md), one JSON object per
//!   line.
//! - [`claude`]: Claude Code's hook dialect: which events wait, the cards
//!   they become, and the exact JSON a human's answer maps back to.
//! - [`trim`]: shrinking a hook payload before it crosses the socket.
//! - [`paths`]: the socket, the state dir, and the checks on both.
//! - [`commands`], [`intent`], [`persona`]: what the companion can do, the
//!   offline matcher that understands plain commands, and how a model asks
//!   for one.
//! - [`find`]: what "find pdfs about tax" asks for, and how well a name
//!   answers it (the daemon's finder walks the disk with it).
//!
//! Pure code only: no I/O beyond reading the environment, so every rule here
//! is unit-tested.

pub mod claude;
pub mod commands;
pub mod find;
pub mod id;
pub mod intent;
pub mod knowledge;
pub mod paths;
pub mod persona;
pub mod proto;
pub mod shell;
pub mod trim;

/// The version every binary reports (`VERSION` at the repo root).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
