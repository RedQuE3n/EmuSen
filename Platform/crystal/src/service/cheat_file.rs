//! A cheat list handed to CHEAT_IMPORT read as the C# reads it: `JsonSerializer.Deserialize<CheatFile>` under
//! `ConfigJson.Options` (comments skipped, trailing commas allowed, names in any case), its refusal of a value of the
//! wrong kind in its own words and position, and `CheatRegistry.LoadFrom`'s loop, which faults on a null where it
//! reads through one after loading the entries before it (EmuSen_Backend.md §29.24).

use emusen_cheats::registry::CheatRegistry;
use emusen_dotnet::json::Value;
use emusen_dotnet::text::equals_ignore_case;

/// What a property of the model holds.
#[derive(Clone, Copy)]
enum Shape {
    File,
    Entries,
    Entry,
    Writes,
    Write,
    Text,
    Flag,
    Count,
    MaybeCount,
}

impl Shape {
    /// The .NET type a refusal names.
    fn type_name(self) -> &'static str {
        match self {
            Shape::File => "EmuSen.Galaxia.Models.CheatFile",
            Shape::Entries => "System.Collections.Generic.List`1[EmuSen.Galaxia.Models.CheatFileEntry]",
            Shape::Entry => "EmuSen.Galaxia.Models.CheatFileEntry",
            Shape::Writes => "System.Collections.Generic.List`1[EmuSen.Galaxia.Models.CheatFileWrite]",
            Shape::Write => "EmuSen.Galaxia.Models.CheatFileWrite",
            Shape::Text => "System.String",
            Shape::Flag => "System.Boolean",
            Shape::Count => "System.Int32",
            Shape::MaybeCount => "System.Nullable`1[System.Int32]",
        }
    }

    /// A property's shape by its name in any case; none for a name the model has no setter for.
    fn member(self, name: &str) -> Option<Shape> {
        let fields: &[(&str, Shape)] = match self {
            Shape::File => &[("Cheats", Shape::Entries)],
            Shape::Entry => &[("Kind", Shape::Text), ("Writes", Shape::Writes), ("Compare", Shape::Text), ("Description", Shape::Text), ("Enabled", Shape::Flag), ("Space", Shape::Text), ("Address", Shape::Text), ("Value", Shape::Text)],
            Shape::Write => &[
                ("Space", Shape::Text),
                ("Address", Shape::Text),
                ("Value", Shape::Text),
                ("Width", Shape::Count),
                ("Type", Shape::Text),
                ("BitPosition", Shape::MaybeCount),
                ("BigEndian", Shape::Flag),
                ("RepeatCount", Shape::Count),
                ("RepeatAddAddress", Shape::Text),
                ("RepeatAddValue", Shape::Text),
            ],
            _ => &[],
        };
        fields.iter().find(|(n, _)| equals_ignore_case(n, name)).map(|(_, s)| *s)
    }
}

/// A reader over the text that knows where it is, as `Utf8JsonReader` counts lines and bytes.
struct Reader<'a> {
    b: &'a [u8],
    at: usize,
    line: usize,
    line_start: usize,
}

impl Reader<'_> {
    fn skip(&mut self) {
        loop {
            match self.b.get(self.at) {
                Some(b'\n') => {
                    self.at += 1;
                    self.line += 1;
                    self.line_start = self.at;
                }
                Some(b' ' | b'\t' | b'\r') => self.at += 1,
                Some(b'/') if self.b.get(self.at + 1) == Some(&b'/') => {
                    while self.b.get(self.at).is_some_and(|&c| c != b'\n') {
                        self.at += 1;
                    }
                }
                Some(b'/') if self.b.get(self.at + 1) == Some(&b'*') => {
                    self.at += 2;
                    while self.at < self.b.len() && !(self.b[self.at] == b'*' && self.b.get(self.at + 1) == Some(&b'/')) {
                        if self.b[self.at] == b'\n' {
                            self.line += 1;
                            self.line_start = self.at + 1;
                        }
                        self.at += 1;
                    }
                    self.at = (self.at + 2).min(self.b.len());
                }
                _ => return,
            }
        }
    }

    fn position(&self) -> String {
        format!("LineNumber: {} | BytePositionInLine: {}.", self.line, self.at - self.line_start)
    }

    /// A string token's end, its text read.
    fn string(&mut self) -> String {
        let start = self.at;
        self.at += 1;
        while self.at < self.b.len() && self.b[self.at] != b'"' {
            self.at += if self.b[self.at] == b'\\' { 2 } else { 1 };
        }
        self.at += 1;
        match emusen_dotnet::json::parse(std::str::from_utf8(&self.b[start..self.at]).unwrap_or("\"\"")) {
            Ok(Value::String(t)) => t.text,
            _ => String::new(),
        }
    }

    /// A scalar's token passed over, its text kept for a number.
    fn scalar(&mut self) -> &[u8] {
        let start = self.at;
        while self.at < self.b.len() && !matches!(self.b[self.at], b',' | b'}' | b']' | b' ' | b'\t' | b'\r' | b'\n' | b'/') {
            self.at += 1;
        }
        &self.b[start..self.at]
    }

    /// A value of `shape` read at `path`; the refusal as `JsonException` words it.
    fn value(&mut self, shape: Shape, path: &str) -> Result<(), String> {
        self.skip();
        let refuse = |r: &Reader| Err(format!("The JSON value could not be converted to {}. Path: {path} | {}", shape.type_name(), r.position()));
        match self.b.get(self.at) {
            Some(b'{') => {
                self.at += 1;
                if !matches!(shape, Shape::File | Shape::Entry | Shape::Write) {
                    return refuse(self);
                }
                loop {
                    self.skip();
                    if self.b.get(self.at) == Some(&b'}') {
                        self.at += 1;
                        return Ok(());
                    }
                    let name = self.string();
                    self.skip();
                    self.at += 1;
                    let child = format!("{path}.{name}");
                    match shape.member(&name) {
                        Some(s) => self.value(s, &child)?,
                        None => self.skip_value(),
                    }
                    self.skip();
                    if self.b.get(self.at) == Some(&b',') {
                        self.at += 1;
                    }
                }
            }
            Some(b'[') => {
                self.at += 1;
                let element = match shape {
                    Shape::Entries => Shape::Entry,
                    Shape::Writes => Shape::Write,
                    _ => return refuse(self),
                };
                let mut i = 0;
                loop {
                    self.skip();
                    if self.b.get(self.at) == Some(&b']') {
                        self.at += 1;
                        return Ok(());
                    }
                    self.value(element, &format!("{path}[{i}]"))?;
                    i += 1;
                    self.skip();
                    if self.b.get(self.at) == Some(&b',') {
                        self.at += 1;
                    }
                }
            }
            Some(b'"') => {
                self.string();
                if matches!(shape, Shape::Text) { Ok(()) } else { refuse(self) }
            }
            _ => {
                let token = self.scalar().to_vec();
                let fits = match &token[..] {
                    b"null" => !matches!(shape, Shape::Flag | Shape::Count),
                    b"true" | b"false" => matches!(shape, Shape::Flag),
                    number => matches!(shape, Shape::Count | Shape::MaybeCount) && std::str::from_utf8(number).ok().and_then(|n| n.parse::<i32>().ok()).is_some(),
                };
                if fits { Ok(()) } else { refuse(self) }
            }
        }
    }

    fn skip_value(&mut self) {
        self.skip();
        match self.b.get(self.at) {
            Some(b'{' | b'[') => {
                let mut depth = 0;
                while self.at < self.b.len() {
                    match self.b[self.at] {
                        b'{' | b'[' => depth += 1,
                        b'}' | b']' => {
                            depth -= 1;
                            if depth == 0 {
                                self.at += 1;
                                return;
                            }
                        }
                        b'"' => {
                            self.string();
                            continue;
                        }
                        b'\n' => {
                            self.line += 1;
                            self.line_start = self.at + 1;
                        }
                        _ => {}
                    }
                    self.at += 1;
                }
            }
            Some(b'"') => {
                self.string();
            }
            _ => {
                self.scalar();
            }
        }
    }
}

/// `Deserialize<CheatFile>`'s refusal of the text, none where it reads.
pub fn refusal(text: &str) -> Option<String> {
    let mut r = Reader { b: text.as_bytes(), at: 0, line: 0, line_start: 0 };
    if let Err(words) = r.value(Shape::File, "$") {
        return Some(words);
    }
    r.skip();
    let c = *r.b.get(r.at)?;
    let shown = if (0x20..0x7F).contains(&c) { (c as char).to_string() } else { format!("0x{c:02X}") };
    Some(format!("'{shown}' is invalid after a single JSON value. Expected end of data. Path: $ | {}", r.position()))
}

/// The words of a null the C# reads through.
pub const NULL_WORDS: &str = "Object reference not set to an instance of an object.";

/// `LoadFrom`: each entry loaded in turn; an entry the C# would read a null of stops it, those before it kept.
pub fn load(registry: &mut CheatRegistry, file: &Value) -> Result<usize, String> {
    let entries = match file.get("Cheats") {
        None => return Ok(0),
        Some(Value::Array(e)) => e,
        Some(_) => return Err(NULL_WORDS.to_string()),
    };
    let mut loaded = 0;
    for e in entries {
        if reads_a_null(e) {
            return Err(NULL_WORDS.to_string());
        }
        let one = Value::Object(vec![(emusen_dotnet::json::Text::new("Cheats"), Value::Array(vec![e.clone()]))]);
        loaded += registry.load(&one).0;
    }
    Ok(loaded)
}

/// Whether `TryReadEntry` reaches a null: the entry itself, its writes after a compare it reads, or a write before
/// which every write parsed.
fn reads_a_null(e: &Value) -> bool {
    if e.is_null() {
        return true;
    }
    let mut probe = CheatRegistry::new();
    let compare_only = Value::Object(vec![
        (emusen_dotnet::json::Text::new("Compare"), e.get("Compare").cloned().unwrap_or(Value::Null)),
        (emusen_dotnet::json::Text::new("Address"), Value::string("0")),
    ]);
    if probe.load(&Value::Object(vec![(emusen_dotnet::json::Text::new("Cheats"), Value::Array(vec![compare_only]))])).0 == 0 {
        return false;
    }
    let Some(writes) = e.get("Writes") else { return false };
    let Value::Array(list) = writes else { return writes.is_null() };
    let Some(first_null) = list.iter().position(Value::is_null) else { return false };
    if first_null == 0 {
        return true;
    }
    let mut prefix = e.clone();
    if let Value::Object(members) = &mut prefix {
        for (k, v) in members.iter_mut() {
            if k.text == "Writes" {
                *v = Value::Array(list[..first_null].to_vec());
            }
        }
    }
    CheatRegistry::new().load(&Value::Object(vec![(emusen_dotnet::json::Text::new("Cheats"), Value::Array(vec![prefix]))])).0 == 1
}
