//! What ureq does not keep of an answer: its status line as sent, and whether any byte of it came at all. .NET's words
//! carry the server's reason phrase and an invalid line as written, and it sends again a request no byte answered, so
//! the client watches each connection's first line after a request (EmuSen_Backend.md §16.1).

use std::sync::{Arc, Mutex};

use ureq::Error;
use ureq::unversioned::transport::{Buffers, ConnectionDetails, Connector, NextTimeout, Transport};

/// What the latest request's connection saw. One request runs at a time on a `ureq` Agent, so one record serves it.
#[derive(Debug, Default, Clone)]
pub struct Seen {
    /// Whether the request was sent.
    pub sent: bool,
    /// Whether any byte came after the request was sent.
    pub answered: bool,
    /// The first line that came, without its line end, each byte read as Latin-1, as .NET reads a reason phrase.
    pub status_line: Option<String>,
}

pub type Record = Arc<Mutex<Seen>>;

#[derive(Debug)]
pub struct Watch {
    pub record: Record,
}

impl<In: Transport> Connector<In> for Watch {
    type Out = Watched<In>;

    fn connect(&self, _: &ConnectionDetails, chained: Option<In>) -> Result<Option<Self::Out>, Error> {
        Ok(chained.map(|inner| Watched { inner, record: self.record.clone(), watching: false }))
    }
}

#[derive(Debug)]
pub struct Watched<T> {
    inner: T,
    record: Record,
    watching: bool,
}

impl<T: Transport> Transport for Watched<T> {
    fn buffers(&mut self) -> &mut dyn Buffers {
        self.inner.buffers()
    }

    fn transmit_output(&mut self, amount: usize, timeout: NextTimeout) -> Result<(), Error> {
        if !self.watching {
            self.watching = true;
            *self.record.lock().unwrap_or_else(|p| p.into_inner()) = Seen { sent: true, ..Seen::default() };
        }
        self.inner.transmit_output(amount, timeout)
    }

    fn await_input(&mut self, timeout: NextTimeout) -> Result<bool, Error> {
        let more = self.inner.await_input(timeout)?;
        if self.watching {
            let input = self.inner.buffers().input();
            let mut seen = self.record.lock().unwrap_or_else(|p| p.into_inner());
            seen.answered |= !input.is_empty();
            if let Some(end) = input.iter().position(|&b| b == b'\n') {
                let line = &input[..end];
                let line = line.strip_suffix(b"\r").unwrap_or(line);
                seen.status_line = Some(line.iter().map(|&b| b as char).collect());
                self.watching = false;
            }
        }
        Ok(more)
    }

    fn is_open(&mut self) -> bool {
        self.inner.is_open()
    }

    fn is_tls(&self) -> bool {
        self.inner.is_tls()
    }
}

/// The reason phrase of a status line, empty where it carries none.
pub fn reason(status_line: &str) -> &str {
    let mut parts = status_line.splitn(3, ' ');
    parts.next();
    parts.next();
    parts.next().unwrap_or("")
}
