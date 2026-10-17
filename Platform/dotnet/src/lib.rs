//! .NET's rules, reproduced where an answer the player reads depends on them: text (casing, ordinal comparison,
//! lengths in UTF-16 code units, white space), numbers (`TryParse`, `Convert.ToInt32` in base 16), text files
//! (`File.ReadAllText`, its lines, its writes and the words of its exceptions), JSON as System.Text.Json reads and
//! writes it, and the C# `Suggestion`. Pure functions, no state, no dependency: Galaxia, the shell and the
//! shell's command modules share them, and a module may not link Galaxia (EmuSen_DianaSh.md §16 Q11).

pub mod file;
pub mod json;
mod letter_or_digit;
pub mod number;
pub mod suggestion;
pub mod text;
