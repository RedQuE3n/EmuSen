//! The proxy's side of one runner: started with its socket and descriptors, each call answered within its deadline, and
//! a runner that ends or stops answering reported as the statuses the proxy owns (EmuSen_LibretroHost.md §3.2, §3.4).

use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;

use crate::sandbox::Policy;
use crate::wire::{self, Op, Reply};

/// The core's process ended: it crashed, aborted or was killed.
pub const STATUS_ENDED: i32 = -10;
/// The core stopped answering within its deadline and was stopped.
pub const STATUS_HUNG: i32 = -11;

/// How a runner is started.
#[derive(Clone, Debug)]
pub struct Spawn {
    pub runner: PathBuf,
    pub target: PathBuf,
    pub policy: Policy,
    pub env: Vec<(String, String)>,
    /// Descriptors the runner receives from 4 up, in order.
    pub fds: Vec<RawFd>,
    pub require_abi: i64,
}

pub struct Link {
    child: Option<Child>,
    sock: UnixStream,
    /// Why the runner is gone: its status and its words.
    pub dead: Option<(i32, String)>,
    /// What the runner said when it was ready: how it is confined.
    pub hello: String,
    pub pid: u32,
}

fn describe(status: std::process::ExitStatus) -> String {
    match (status.code(), status.signal()) {
        (_, Some(sig)) => format!("the core's process ended on signal {sig}"),
        (Some(code), None) => format!("the core's process exited with code {code}"),
        _ => "the core's process ended".to_owned(),
    }
}

impl Link {
    /// A runner started and ready, or why it could not be.
    pub fn spawn(s: &Spawn, deadline: Duration) -> Result<Link, String> {
        let (ours, theirs) = UnixStream::pair().map_err(|e| format!("the runner's socket: {e}"))?;
        let child_fd = theirs.as_raw_fd();
        let fds = s.fds.clone();
        let mut cmd = Command::new(&s.runner);
        cmd.arg("--target").arg(&s.target).arg("--require-abi").arg(s.require_abi.to_string()).args(s.policy.to_args());
        for (k, v) in &s.env {
            cmd.env(k, v);
        }
        unsafe {
            cmd.pre_exec(move || {
                // Every source moved above the targets first, so placing one never overwrites another still to come.
                let mut high = [-1i32; 17];
                for (i, fd) in std::iter::once(child_fd).chain(fds.iter().copied()).enumerate().take(high.len()) {
                    high[i] = libc::fcntl(fd, libc::F_DUPFD, 100);
                    if high[i] < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                for (i, &fd) in high.iter().enumerate().take(1 + fds.len()) {
                    if libc::dup2(fd, 3 + i as i32) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
        let child = cmd.spawn().map_err(|e| format!("{} does not start: {e}", s.runner.display()))?;
        drop(theirs);
        let pid = child.id();
        let mut link = Link { child: Some(child), sock: ours, dead: None, hello: String::new(), pid };
        let hello = link.receive(deadline).map_err(|(_, why)| why)?;
        if hello.ret != 0 {
            return Err(String::from_utf8_lossy(&hello.bytes).into_owned());
        }
        link.hello = String::from_utf8_lossy(&hello.bytes).into_owned();
        Ok(link)
    }

    fn receive(&mut self, deadline: Duration) -> Result<Reply, (i32, String)> {
        if let Some(d) = &self.dead {
            return Err(d.clone());
        }
        let _ = self.sock.set_read_timeout(Some(deadline.max(Duration::from_millis(1))));
        match wire::read_reply(&mut self.sock) {
            Ok(r) => Ok(r),
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {
                self.stop();
                let d = (STATUS_HUNG, format!("the core stopped answering within {} s and was stopped", deadline.as_secs_f64()));
                self.dead = Some(d.clone());
                Err(d)
            }
            Err(_) => {
                let words = self.reap();
                let d = (STATUS_ENDED, words);
                self.dead = Some(d.clone());
                Err(d)
            }
        }
    }

    /// One call: its reply, or the status of a runner that ended or hung.
    pub fn call(&mut self, op: Op, out_null: bool, out_len: u64, args: &[u8], deadline: Duration) -> Result<Reply, (i32, String)> {
        if let Some(d) = &self.dead {
            return Err(d.clone());
        }
        if wire::send_request(&mut self.sock, op, out_null, out_len, args).is_err() {
            let d = (STATUS_ENDED, self.reap());
            self.dead = Some(d.clone());
            return Err(d);
        }
        self.receive(deadline)
    }

    /// A call whose descriptors follow its request in a message of their own (`RingLend`); they stay the caller's.
    pub fn call_with_fds(&mut self, op: Op, args: &[u8], fds: &[RawFd], deadline: Duration) -> Result<Reply, (i32, String)> {
        if let Some(d) = &self.dead {
            return Err(d.clone());
        }
        if wire::send_request(&mut self.sock, op, false, 0, args).is_err() || wire::send_fds(&self.sock, fds).is_err() {
            let d = (STATUS_ENDED, self.reap());
            self.dead = Some(d.clone());
            return Err(d);
        }
        self.receive(deadline)
    }

    fn stop(&mut self) {
        if let Some(c) = self.child.as_mut() {
            let _ = c.kill();
            let _ = c.wait();
        }
        self.child = None;
    }

    fn reap(&mut self) -> String {
        match self.child.take() {
            Some(mut c) => {
                let waited = (0..200).find_map(|_| match c.try_wait() {
                    Ok(Some(s)) => Some(s),
                    _ => {
                        std::thread::sleep(Duration::from_millis(5));
                        None
                    }
                });
                match waited {
                    Some(s) => describe(s),
                    None => {
                        let _ = c.kill();
                        c.wait().map(describe).unwrap_or_else(|_| "the core's process ended".into())
                    }
                }
            }
            None => "the core's process ended".into(),
        }
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        let _ = self.sock.shutdown(std::net::Shutdown::Both);
        if let Some(mut c) = self.child.take() {
            let quick = (0..100).any(|_| {
                if matches!(c.try_wait(), Ok(Some(_))) {
                    return true;
                }
                std::thread::sleep(Duration::from_millis(5));
                false
            });
            if !quick {
                let _ = c.kill();
                let _ = c.wait();
            }
        }
    }
}
