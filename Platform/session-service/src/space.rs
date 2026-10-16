//! The memory spaces as commands find them: `FindSpace`'s rule and words, and a bulk read.

use crate::{Record, Session};
use emusen_dotnet::text::equals_ignore_case;

/// One memory space as `spaces` lists it.
pub struct Space {
    pub name: String,
    pub size: i32,
    pub writable: bool,
    pub side_effects: bool,
}

/// Every space of the session, in its order.
pub fn all(session: &Session) -> Result<Vec<Space>, String> {
    Ok(session
        .spaces()?
        .items()
        .iter()
        .map(|s| Space { name: s.text("name"), size: s.int("size") as i32, writable: s.flag("writable"), side_effects: s.flag("side_effects") })
        .collect())
}

/// `FindSpace`: a space by name ignoring case, or the exception that lists the rest.
pub fn find(session: &Session, name: &str) -> Result<Space, String> {
    let spaces = all(session)?;
    let names = spaces.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", ");
    spaces.into_iter().find(|s| equals_ignore_case(&s.name, name)).ok_or_else(|| format!("No memory space named '{name}'. Available: {names}"))
}

/// `len` bytes from `address` on, each read as the space reads it.
pub fn read(session: &Session, space: &Space, address: i32, len: i32) -> Result<Vec<u8>, String> {
    if len <= 0 {
        return Ok(Vec::new());
    }
    Ok(session.space_read(&space.name, address as u32, len as usize)?)
}
