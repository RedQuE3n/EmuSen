//! The save-state shell every core's `state.rs` shared: the refusals a header can earn, the sizing, saving and listing
//! around a core's own fields, and the checks of its tests (EmuSen_CoreAPI.md §32).

use crate::ffi::{StateMachine, Status, status};
use crate::{StateReader, StateWriter, Truncated};

/// What a state's frame refuses before any core field is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    Truncated(Truncated),
    Foreign(u32),
    Version(i32),
    BufferTooSmall { needed: usize },
}

/// The reader past the magic and the version, once `accepts` takes the version.
pub fn open_state(data: &[u8], magic: u32, accepts: impl Fn(i32) -> bool) -> Result<(StateReader<'_>, i32), Refusal> {
    let mut r = StateReader::new(data);
    let found = r.u32().map_err(|_| Refusal::Foreign(0))?;
    if found != magic {
        return Err(Refusal::Foreign(found));
    }
    let version = r.i32().map_err(Refusal::Truncated)?;
    if !accepts(version) {
        return Err(Refusal::Version(version));
    }
    Ok((r, version))
}

/// The bytes a state of `write` takes.
pub fn size_of_state(write: impl FnOnce(&mut StateWriter)) -> usize {
    let mut w = StateWriter::counter();
    write(&mut w);
    w.len()
}

/// `write` into `out`, refused when `out` is shorter than the state; the bytes written.
pub fn save_state_with(out: &mut [u8], write: impl Fn(&mut StateWriter)) -> Result<usize, Refusal> {
    let needed = size_of_state(&write);
    if out.len() < needed {
        return Err(Refusal::BufferTooSmall { needed });
    }
    let mut w = StateWriter::new(out);
    write(&mut w);
    Ok(w.len())
}

/// The layout listing of `write`.
pub fn layout_of_state(write: impl FnOnce(&mut StateWriter)) -> String {
    let mut w = StateWriter::layout();
    write(&mut w);
    w.into_layout()
}

/// A core's `StateError`: the four shared refusals, then the core's own variants each with its status.
#[macro_export]
macro_rules! state_error {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $variant:ident $(($($tuple:tt)*))? $({$($fields:tt)*})? => $code:expr),* $(,)?
    }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        $vis enum $name {
            Truncated { at: usize, wanted: usize },
            Foreign(u32),
            Version(i32),
            BufferTooSmall { needed: usize },
            $($(#[$vmeta])* $variant $(($($tuple)*))? $({$($fields)*})?),*
        }

        impl $crate::ffi::Status for $name {
            fn status(&self) -> i32 {
                match self {
                    $name::Truncated { .. } => $crate::ffi::status::TRUNCATED,
                    $name::Foreign(_) => $crate::ffi::status::FOREIGN,
                    $name::Version(_) => $crate::ffi::status::VERSION,
                    $name::BufferTooSmall { .. } => $crate::ffi::status::BUFFER_TOO_SMALL,
                    $($name::$variant { .. } => $code),*
                }
            }
        }

        impl From<$crate::Truncated> for $name {
            fn from(t: $crate::Truncated) -> Self {
                $name::Truncated { at: t.at, wanted: t.wanted }
            }
        }

        impl From<$crate::shell::Refusal> for $name {
            fn from(r: $crate::shell::Refusal) -> Self {
                match r {
                    $crate::shell::Refusal::Truncated(t) => t.into(),
                    $crate::shell::Refusal::Foreign(m) => $name::Foreign(m),
                    $crate::shell::Refusal::Version(v) => $name::Version(v),
                    $crate::shell::Refusal::BufferTooSmall { needed } => $name::BufferTooSmall { needed },
                }
            }
        }
    };
}

/// `state_size`, `save_state` and `layout` of a `StateMachine` whose fields a method `$write(&self, &mut StateWriter)`
/// writes; a second method `$ready(&self) -> Result<(), Error>` may refuse a save after the length check.
#[macro_export]
macro_rules! state_write_methods {
    ($write:ident) => {
        fn state_size(&self) -> usize {
            $crate::shell::size_of_state(|w| self.$write(w))
        }

        fn save_state(&self, out: &mut [u8]) -> Result<usize, Self::Error> {
            Ok($crate::shell::save_state_with(out, |w| self.$write(w))?)
        }

        fn layout(&self) -> String {
            $crate::shell::layout_of_state(|w| self.$write(w))
        }
    };
    ($write:ident, $ready:ident) => {
        fn state_size(&self) -> usize {
            $crate::shell::size_of_state(|w| self.$write(w))
        }

        fn save_state(&self, out: &mut [u8]) -> Result<usize, Self::Error> {
            let needed = self.state_size();
            if out.len() < needed {
                return Err($crate::shell::Refusal::BufferTooSmall { needed }.into());
            }
            self.$ready()?;
            Ok($crate::shell::save_state_with(out, |w| self.$write(w))?)
        }

        fn layout(&self) -> String {
            $crate::shell::layout_of_state(|w| self.$write(w))
        }
    };
}

/// A state refused as truncated, as foreign by a wrong or missing magic, and as an older or newer version than any core reads; `m` is unchanged.
pub fn assert_header_refusals<M: StateMachine>(m: &mut M, state: &[u8]) {
    assert_eq!(m.load_state(&state[..state.len() - 1]).map_err(|e| e.status()), Err(status::TRUNCATED));
    assert_eq!(m.load_state(b"NPHR0000").map_err(|e| e.status()), Err(status::FOREIGN));
    assert_eq!(m.load_state(&[]).map_err(|e| e.status()), Err(status::FOREIGN));
    let mut v1 = state.to_vec();
    v1[4] = 1;
    assert_eq!(m.load_state(&v1).map_err(|e| e.status()), Err(status::VERSION));
    let mut newer = state.to_vec();
    newer[4..8].copy_from_slice(&0x7FFFi32.to_le_bytes());
    assert_eq!(m.load_state(&newer).map_err(|e| e.status()), Err(status::VERSION));
}
