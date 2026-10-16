//! The .NET rules the C# shell's words and choices rest on, owned by `emusen-dotnet` and named here as the shell
//! uses them. See EmuSen_DianaSh.md §13.7.

pub use emusen_dotnet::file::{current_directory, io_message, read_all_lines, read_all_text, write_all_text};
pub use emusen_dotnet::number::{parse_i32, parse_i64};
pub use emusen_dotnet::text::{equals_ignore_case as same, fold, is_letter, is_letter_or_digit, pad_left, pad_right, starts_with_ignore_case, to_lower, trim, trim_start};
