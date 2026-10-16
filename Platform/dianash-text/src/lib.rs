//! `dianash.text`: cat, head, tail, wc, sort, uniq, grep, sed, awk, ls, find, cp, mv, rm, touch and xxd, each
//! written to print what its C# class printed. A failure's words go to standard error (§16 Q10), which the shell
//! prints where the output goes. See EmuSen_DianaSh.md §8.1 and §13.8.

mod clock;
mod files;
mod lines;
mod pattern;

use dianash_module::{Call, Module};

/// What a command gives back: its standard output, its standard error and its exit status.
pub(crate) struct Answer {
    pub out: String,
    pub err: String,
    pub status: i32,
}

impl Answer {
    pub fn ok(out: impl Into<String>) -> Answer {
        Answer { out: out.into(), err: String::new(), status: 0 }
    }

    pub fn with(out: impl Into<String>, status: i32) -> Answer {
        Answer { out: out.into(), err: String::new(), status }
    }

    pub fn fail(err: impl Into<String>) -> Answer {
        Answer { out: String::new(), err: err.into(), status: 1 }
    }
}

/// A failure the C# shell's dispatcher caught as an exception and worded `Error: <message>`.
pub(crate) type Run = Result<Answer, String>;

/// The commands this module has, in the order the shell lists them, with today's usage lines.
pub const COMMANDS: &[(&str, bool, &str)] = &[
    ("sed", true, concat!(
        "  sed s/pat/repl/[gi]           substitute (first match per line, or every match with 'g';\n",
        "                                'i' = case-insensitive). Standalone: sed <expr> <text...>.\n",
        "                                Real use is as a pipeline stage: <command> | sed s/pat/repl/"
    )),
    ("grep", true, concat!(
        "  grep [-i] [-v] [-n] [-c] [-q] <pattern> [text...]\n",
        "                                filter lines matching <pattern> (regex) - -i case-insensitive,\n",
        "                                -v invert (non-matching), -n prefix line numbers, -c print only\n",
        "                                the match count, -q no output (exit code only, for if/while).\n",
        "                                Reads piped stdin if given, else operates on trailing literal text."
    )),
    ("wc", true, concat!(
        "  wc [-l] [-w] [-c] [path]     count lines/words/bytes (all three if no flag given);\n",
        "                                reads piped stdin, or a real file if <path> is given"
    )),
    ("sort", true, concat!(
        "  sort [-n] [-r]                sort input lines (numeric with -n, reversed with -r) -\n",
        "                                reads piped stdin, or trailing literal text"
    )),
    ("uniq", true, concat!(
        "  uniq [-c]                     collapse adjacent duplicate lines (-c prefixes each with its\n",
        "                                repeat count) - sort first for global dedup, not just adjacent"
    )),
    ("awk", true, concat!(
        "  awk [-F sep] 'prog' [path]   run a small AWK subset over stdin or <path>, line by line\n",
        "                                patterns: /regex/, NR==N (also !=,<,<=,>,>=), BEGIN, END\n",
        "                                actions: {print $0|$N|$NF|NR|NF|\"lit\"[, ...][; print ...]}"
    )),
    ("ls", true, concat!(
        "  ls [-a] [-l] [path]           list directory entries (default: current directory);\n",
        "                                -a include dotfiles, -l show type/size/modified-time"
    )),

];

/// The module: one value, no state of its own.
pub struct Text;

pub static MODULE: Text = Text;

fn manifest() -> String {
    let quote = |s: &str| {
        let mut out = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    };
    let commands: Vec<String> = COMMANDS.iter().map(|(name, read_only, usage)| format!("{{\"name\":{},\"usage\":{},\"read_only\":{read_only},\"man\":\"man/{name}.1\"}}", quote(name), quote(usage))).collect();
    format!(
        "{{\"manifest\":1,\"id\":\"dianash.text\",\"version\":\"{}\",\"abi\":\"1.0\",\"kind\":\"module\",\"services\":[],\"commands\":[{}],\"aliases\":{{\"hexdump\":\"xxd\"}}}}",
        env!("CARGO_PKG_VERSION"),
        commands.join(",")
    )
}

impl Module for Text {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(manifest)
    }

    fn run(&self, call: &Call) -> i32 {
        let args = call.args();
        let stdin = call.stdin();
        let answer = match call.command().as_str() {
            "cat" => files::cat(call, &args, stdin),
            "head" => files::head(call, &args, stdin, false),
            "tail" => files::head(call, &args, stdin, true),
            "wc" => files::wc(call, &args, stdin),
            "ls" => files::ls(call, &args),
            "find" => files::find(call, &args),
            "cp" => files::cp(call, &args),
            "mv" | "rm" => Ok(files::suspended(&call.command())),
            "touch" => files::touch(call, &args),
            "xxd" => files::xxd(call, &args, stdin),
            "sort" => Ok(lines::sort(&args, stdin)),
            "uniq" => Ok(lines::uniq(&args, stdin)),
            "grep" => lines::grep(&args, stdin),
            "sed" => lines::sed(&args, stdin),
            "awk" => lines::awk(call, &args, stdin),
            other => Err(format!("dianash.text has no command '{other}'")),
        };
        let answer = answer.unwrap_or_else(|words| Answer::fail(format!("Error: {words}")));
        if !answer.out.is_empty() {
            call.out(&answer.out);
        }
        if !answer.err.is_empty() {
            call.err(&answer.err);
        }
        answer.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_manifest_names_every_command_once() {
        let manifest = Text.manifest();
        for (name, _, _) in COMMANDS {
            assert!(manifest.contains(&format!("\"name\":\"{name}\"")), "{name}");
        }
        assert!(manifest.starts_with("{\"manifest\":1,\"id\":\"dianash.text\""));
    }
}
