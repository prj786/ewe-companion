//! Where things live, and the checks that make the socket safe to trust.
//!
//! - socket: `$XDG_RUNTIME_DIR/ewe-companion.sock` (tests may point
//!   `EWE_COMPANION_SOCK` elsewhere);
//! - state: `$XDG_STATE_HOME/ewe/companion/` (per machine, never synced).

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub const SOCKET_NAME: &str = "ewe-companion.sock";

/// The real uid of this process.
pub fn uid() -> u32 {
    // SAFETY: getuid has no preconditions and cannot fail.
    unsafe { libc::getuid() }
}

/// A directory we may put a socket in, or look for one: a real directory
/// (not a symlink), owned by us, and writable by nobody else.
pub fn dir_is_private(dir: &Path) -> bool {
    match std::fs::symlink_metadata(dir) {
        Ok(m) => m.is_dir() && m.uid() == uid() && m.mode() & 0o022 == 0,
        Err(_) => false,
    }
}

/// The socket path, or `None` when there is no safe place for it (no
/// `XDG_RUNTIME_DIR`, or one that fails [`dir_is_private`]).
pub fn socket_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("EWE_COMPANION_SOCK").filter(|p| !p.is_empty()) {
        let p = PathBuf::from(p);
        return p.parent().filter(|d| dir_is_private(d)).map(|_| p.clone());
    }
    let dir = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").filter(|p| !p.is_empty())?);
    dir_is_private(&dir).then(|| dir.join(SOCKET_NAME))
}

/// `$XDG_STATE_HOME/ewe/companion`, else `~/.local/state/ewe/companion`.
pub fn state_dir() -> PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| "/tmp".into());
            home.join(".local/state")
        });
    base.join("ewe/companion")
}

/// The last path component of a working directory, for cards: `ewe-browser`.
/// The home directory itself reads `~`.
pub fn project_name(cwd: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let trimmed = cwd.trim_end_matches('/');
    if trimmed.is_empty() {
        return "/".into();
    }
    if !home.is_empty() && trimmed == home.trim_end_matches('/') {
        return "~".into();
    }
    trimmed.rsplit('/').next().unwrap_or(trimmed).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_names() {
        assert_eq!(project_name("/home/x/Projects/ewe-browser"), "ewe-browser");
        assert_eq!(project_name("/home/x/Projects/ewe/"), "ewe");
        assert_eq!(project_name("/"), "/");
    }

    #[test]
    fn a_world_writable_dir_is_not_private() {
        assert!(!dir_is_private(Path::new("/tmp")));
        assert!(!dir_is_private(Path::new("/nonexistent/dir")));
    }
}
