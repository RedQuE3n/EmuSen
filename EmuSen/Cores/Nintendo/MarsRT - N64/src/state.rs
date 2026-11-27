//! MarsRT's state: the shared codec (`emusen-native`) and the refusals only Mars's format has. See Mars_Native.md §5.1.

pub use emusen_native::{State, StateReader, StateWriter, Truncated};

emusen_native::state_error! {
    /// Why a state could not be read or written; each has a status code for the C ABI.
    pub enum StateError {
        RdramSize(i32) => -12,
        PendingOverflow(i32) => -13,
        PendingInState(usize) => -14,
        /// Corrupted or out-of-range state payload.
        Corrupt => emusen_native::core::status::FOREIGN,
    }
}

pub type StateResult<T = ()> = Result<T, StateError>;

/// A zeroed array on the heap, too large to build on the stack.
pub fn boxed<T: Copy, const N: usize>(value: T) -> Box<[T; N]> {
    vec![value; N].into_boxed_slice().try_into().unwrap_or_else(|_| unreachable!())
}
