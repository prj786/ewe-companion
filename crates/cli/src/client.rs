//! The daemon's socket, from the client side: connect within a budget,
//! check who is on the other end, one line out, one line back.

use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use companion_core::paths;
use serde_json::Value;
use socket2::{Domain, SockAddr, Socket, Type};

/// The uid of the process on the other end of a Unix socket.
fn peer_uid(s: &UnixStream) -> Option<u32> {
    let mut cred = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: SO_PEERCRED fills a ucred of exactly `len` bytes on a valid fd.
    let rc = unsafe {
        libc::getsockopt(
            s.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&mut cred as *mut libc::ucred).cast(),
            &mut len,
        )
    };
    (rc == 0).then_some(cred.uid)
}

/// Connect within `budget`, or `None`: no safe runtime dir, no socket,
/// nobody listening, or a listener that is not us.
pub fn connect(budget: Duration) -> Option<UnixStream> {
    let path = paths::socket_path()?;
    let addr = SockAddr::unix(&path).ok()?;
    let sock = Socket::new(Domain::UNIX, Type::STREAM, None).ok()?;
    sock.connect_timeout(&addr, budget).ok()?;
    let stream: UnixStream = sock.into();
    if peer_uid(&stream)? != paths::uid() {
        return None;
    }
    Some(stream)
}

pub fn send_line(stream: &mut UnixStream, msg: &Value, budget: Duration) -> Option<()> {
    stream.set_write_timeout(Some(budget)).ok()?;
    let mut line = msg.to_string();
    line.push('\n');
    stream.write_all(line.as_bytes()).ok()?;
    stream.flush().ok()
}

/// Read one `\n`-terminated line before `deadline`, at most `cap` bytes.
pub fn read_line(stream: &mut UnixStream, deadline: Instant, cap: usize) -> Option<String> {
    let mut buf = Vec::with_capacity(256);
    let mut chunk = [0u8; 4096];
    loop {
        let left = deadline.checked_duration_since(Instant::now())?;
        if left.is_zero() {
            return None;
        }
        stream.set_read_timeout(Some(left)).ok()?;
        let n = stream.read(&mut chunk).ok()?;
        if n == 0 {
            return None; // closed before a full line
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            buf.truncate(pos);
            return String::from_utf8(buf).ok();
        }
        if buf.len() > cap {
            return None;
        }
    }
}

/// One request, one reply: what every CLI verb does.
pub fn request(msg: &Value, wait: Duration) -> Option<Value> {
    let mut s = connect(Duration::from_millis(300))?;
    send_line(&mut s, msg, Duration::from_secs(2))?;
    let line = read_line(
        &mut s,
        Instant::now() + wait,
        companion_core::proto::MAX_LINE,
    )?;
    serde_json::from_str(&line).ok()
}
