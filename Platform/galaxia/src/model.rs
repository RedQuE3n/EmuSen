//! A config model as data: its fields, their types and their defaults, and the two things done with them, binding a
//! document that was read and writing one out.
//!
//! Binding reproduces how System.Text.Json fills a C# class under `ConfigJson.Options`; writing reproduces what it
//! serializes. A bound document is a `Value` holding every field of its schema in the schema's order, so that the
//! writer, the C# facade and a later reader all see the same shape. See EmuSen_RustPlatform.md §11.2.

use crate::dotnet_path::equals_ignore_case;
use crate::json::{self, Text, Value};
use std::fmt;

/// What a field holds, by the C# type it mirrors. Everything C# can leave null is nullable here too.
#[derive(Clone, Copy, Debug)]
pub enum Type {
    /// `string`: a string or null.
    String,
    Bool,
    /// `bool?`.
    NullableBool,
    /// `int`: an integer token in range.
    Int32,
    /// `int?`.
    NullableInt32,
    /// `long`.
    Int64,
    /// `double`: any number token.
    Double,
    /// `float`: any number token, held at a float's width.
    Single,
    /// A class: an object bound to its own schema, or null.
    Object(&'static Schema),
    /// `List<T>`: an array, or null.
    List(&'static Type),
    /// `Dictionary<string, T>`: an object whose names are keys, or null.
    Map(&'static Type),
}

/// A field's value in a new instance.
#[derive(Clone, Copy, Debug)]
pub enum Initial {
    Null,
    Bool(bool),
    /// A number by its token.
    Number(&'static str),
    Str(&'static str),
    /// A list of strings; empty for an empty list of anything.
    Strings(&'static [&'static str]),
    EmptyMap,
    /// A new instance of the field's own class.
    New,
}

/// A value computed from the object it belongs to, written and never read: a C# property with a getter alone.
pub type Derive = fn(&Value) -> Result<Value, String>;

#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub name: &'static str,
    pub ty: Type,
    pub initial: Initial,
    /// Left out of the file while null.
    pub skip_null: bool,
    pub derived: Option<Derive>,
}

impl Field {
    pub const fn new(name: &'static str, ty: Type, initial: Initial) -> Field {
        Field { name, ty, initial, skip_null: false, derived: None }
    }

    pub const fn skipped_when_null(mut self) -> Field {
        self.skip_null = true;
        self
    }

    pub const fn derived(name: &'static str, ty: Type, derive: Derive) -> Field {
        Field { name, ty, initial: Initial::Null, skip_null: false, derived: Some(derive) }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub name: &'static str,
    pub fields: &'static [Field],
}

/// A value the schema does not allow, and where it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindError {
    pub path: String,
    pub what: &'static str,
}

impl fmt::Display for BindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} is {}.", self.path, self.what)
    }
}

/// A refusal with no place yet: each level it passes on the way out puts its own part of the path in front, so a document that binds builds no path at all.
fn refuse<T>(what: &'static str) -> Result<T, BindError> {
    Err(BindError { path: String::new(), what })
}

impl BindError {
    fn under(mut self, part: impl FnOnce() -> String) -> BindError {
        self.path.insert_str(0, &part());
        self
    }
}

/// An integer token: digits with an optional sign and nothing after them, which is all .NET takes for an integer.
fn integer(token: &str) -> Option<i128> {
    let digits = token.strip_prefix('-').unwrap_or(token);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    token.parse().ok()
}

impl Schema {
    /// A new instance: every field at its initial value.
    pub fn new_instance(&self) -> Value {
        Value::Object(self.fields.iter().filter(|field| field.derived.is_none()).map(|field| (Text::new(field.name), initial(field))).collect())
    }

    /// The field a JSON name means: .NET matches names without regard to case.
    fn field(&self, name: &str) -> Option<&Field> {
        // A name in its own case is the usual one and is found without casing anything; no two fields differ by case alone.
        self.fields.iter().find(|field| field.name == name).or_else(|| self.fields.iter().find(|field| equals_ignore_case(field.name, name)))
    }

    /// Binds an object read from a file: each member replaces its field whole, an unknown member is ignored, and a field with no member keeps its initial value.
    fn bind_object(&'static self, members: &[(Text, Value)]) -> Result<Value, BindError> {
        let Value::Object(mut bound) = self.new_instance() else { unreachable!("a new instance is an object") };
        for (name, value) in members {
            if name.broken {
                return refuse("an object with a name that is not valid text");
            }
            let Some(field) = self.field(&name.text) else { continue };
            // A value given for a property that cannot be set is skipped, whatever it is.
            if field.derived.is_some() {
                continue;
            }
            let slot = bound.iter_mut().find(|(key, _)| key.text == field.name).expect("every settable field has a slot");
            slot.1 = bind(&field.ty, value).map_err(|error| error.under(|| format!(".{}", field.name)))?;
        }
        Ok(Value::Object(bound))
    }

    /// Binds a whole document. A document that is `null` is no document, which is not an error.
    pub fn bind(&'static self, document: &Value) -> Result<Option<Value>, BindError> {
        let bound = match document {
            Value::Null => Ok(None),
            Value::Object(members) => self.bind_object(members).map(Some),
            _ => refuse("not an object"),
        };
        bound.map_err(|error| error.under(|| "$".to_string()))
    }
}

fn initial(field: &Field) -> Value {
    match (field.initial, &field.ty) {
        (Initial::Null, _) => Value::Null,
        (Initial::Bool(value), _) => Value::Bool(value),
        (Initial::Number(token), _) => Value::number(token),
        (Initial::Str(text), _) => Value::string(text),
        (Initial::Strings(items), _) => Value::Array(items.iter().map(|item| Value::string(*item)).collect()),
        (Initial::EmptyMap, _) => Value::Object(Vec::new()),
        (Initial::New, Type::Object(schema)) => schema.new_instance(),
        (Initial::New, _) => Value::Null,
    }
}

/// One value against one type, by .NET's rules for that C# type.
fn bind(ty: &Type, value: &Value) -> Result<Value, BindError> {
    match (ty, value) {
        (Type::String, Value::String(text)) if text.broken => refuse("a string with an escape that is not valid text"),
        (Type::String, Value::String(_) | Value::Null) => Ok(value.clone()),
        (Type::String, _) => refuse("not a string"),
        (Type::Bool | Type::NullableBool, Value::Bool(_)) | (Type::NullableBool, Value::Null) => Ok(value.clone()),
        (Type::Bool | Type::NullableBool, _) => refuse("neither true nor false"),
        (Type::Int32 | Type::NullableInt32, Value::Number(token)) => match integer(token) {
            Some(number) if i32::try_from(number).is_ok() => Ok(value.clone()),
            _ => refuse("not a whole number a 32-bit integer holds"),
        },
        (Type::NullableInt32, Value::Null) => Ok(Value::Null),
        (Type::Int64, Value::Number(token)) => match integer(token) {
            Some(number) if i64::try_from(number).is_ok() => Ok(value.clone()),
            _ => refuse("not a whole number a 64-bit integer holds"),
        },
        (Type::Int32 | Type::NullableInt32 | Type::Int64, _) => refuse("not a whole number"),
        (Type::Double | Type::Single, Value::Number(_)) => Ok(value.clone()),
        (Type::Double | Type::Single, _) => refuse("not a number"),
        (Type::Object(_) | Type::List(_) | Type::Map(_), Value::Null) => Ok(Value::Null),
        (Type::Object(schema), Value::Object(members)) => schema.bind_object(members),
        (Type::Object(_), _) => refuse("not an object"),
        (Type::List(item), Value::Array(items)) => {
            let bound: Result<Vec<Value>, BindError> = items.iter().enumerate().map(|(i, element)| bind(item, element).map_err(|error| error.under(|| format!("[{i}]")))).collect();
            bound.map(Value::Array)
        }
        (Type::List(_), _) => refuse("not a list"),
        (Type::Map(item), Value::Object(members)) => {
            // A key written twice keeps its first place and takes its last value, as a C# dictionary's indexer leaves it.
            let mut bound: Vec<(Text, Value)> = Vec::with_capacity(members.len());
            for (key, member) in members {
                if key.broken {
                    return refuse("a map with a key that is not valid text");
                }
                let member = bind(item, member).map_err(|error| error.under(|| format!(".{}", key.text)))?;
                match bound.iter_mut().find(|(existing, _)| existing.text == key.text) {
                    Some(slot) => slot.1 = member,
                    None => bound.push((key.clone(), member)),
                }
            }
            Ok(Value::Object(bound))
        }
        (Type::Map(_), _) => refuse("not an object"),
    }
}

/// The platform's line ending, which .NET's indented writer uses.
pub const NEW_LINE: &str = if cfg!(windows) { "\r\n" } else { "\n" };

/// One step of the way to the value being written, kept so that a value that cannot be written is named and one that can costs no text.
enum Step<'a> {
    Field(&'static str),
    Index(usize),
    Key(&'a str),
}

struct Writer<'a> {
    out: String,
    steps: Vec<Step<'a>>,
}

impl<'a> Writer<'a> {
    fn line(&mut self, depth: usize) {
        self.out.push_str(NEW_LINE);
        self.out.push_str(&"  ".repeat(depth));
    }

    fn path(&self) -> String {
        let mut path = String::from("$");
        for step in &self.steps {
            match step {
                Step::Field(name) => path.push_str(&format!(".{name}")),
                Step::Index(i) => path.push_str(&format!("[{i}]")),
                Step::Key(key) => path.push_str(&format!(".{key}")),
            }
        }
        path
    }

    fn value(&mut self, ty: &Type, value: &'a Value, depth: usize) -> Result<(), String> {
        match (ty, value) {
            (_, Value::Null) => self.out.push_str("null"),
            (Type::String, Value::String(text)) => self.out.push_str(&json::quoted(&text.text)),
            (Type::Bool | Type::NullableBool, Value::Bool(flag)) => self.out.push_str(if *flag { "true" } else { "false" }),
            (Type::Int32 | Type::NullableInt32 | Type::Int64, Value::Number(token)) => match integer(token) {
                Some(number) => self.out.push_str(&number.to_string()),
                None => return Err(self.wrong()),
            },
            (Type::Double, Value::Number(token)) => match token.parse().ok().map(json::format_f64) {
                Some(Some(number)) => self.out.push_str(&number),
                Some(None) => return Err(format!("{} is not a number JSON can hold.", self.path())),
                None => return Err(self.wrong()),
            },
            (Type::Single, Value::Number(token)) => match token.parse().ok().map(json::format_f32) {
                Some(Some(number)) => self.out.push_str(&number),
                Some(None) => return Err(format!("{} is not a number JSON can hold.", self.path())),
                None => return Err(self.wrong()),
            },
            (Type::Object(schema), Value::Object(_)) => self.object(schema, value, depth)?,
            (Type::List(item), Value::Array(items)) => {
                if items.is_empty() {
                    self.out.push_str("[]");
                    return Ok(());
                }
                self.out.push('[');
                for (i, element) in items.iter().enumerate() {
                    if i > 0 {
                        self.out.push(',');
                    }
                    self.line(depth + 1);
                    self.steps.push(Step::Index(i));
                    self.value(item, element, depth + 1)?;
                    self.steps.pop();
                }
                self.line(depth);
                self.out.push(']');
            }
            (Type::Map(item), Value::Object(members)) => {
                if members.is_empty() {
                    self.out.push_str("{}");
                    return Ok(());
                }
                self.out.push('{');
                for (i, (key, member)) in members.iter().enumerate() {
                    if i > 0 {
                        self.out.push(',');
                    }
                    self.line(depth + 1);
                    self.out.push_str(&json::quoted(&key.text));
                    self.out.push_str(": ");
                    self.steps.push(Step::Key(&key.text));
                    self.value(item, member, depth + 1)?;
                    self.steps.pop();
                }
                self.line(depth);
                self.out.push('}');
            }
            _ => return Err(self.wrong()),
        }
        Ok(())
    }

    fn wrong(&self) -> String {
        format!("{} does not hold what its type writes.", self.path())
    }

    fn object(&mut self, schema: &'static Schema, object: &'a Value, depth: usize) -> Result<(), String> {
        let mut written = 0;
        self.out.push('{');
        for field in schema.fields {
            self.steps.push(Step::Field(field.name));
            let value = match field.derived {
                None => object.get(field.name).ok_or_else(|| format!("{} is missing.", self.path()))?,
                Some(derive) => {
                    // What is computed is written from a copy of its own, since nothing else holds it.
                    let computed = derive(object)?;
                    if !(field.skip_null && computed.is_null()) {
                        self.member(field, &mut written, depth);
                        Writer { out: std::mem::take(&mut self.out), steps: Vec::new() }.finish(&field.ty, &computed, depth + 1).map(|out| self.out = out).map_err(|words| words.replacen('$', &self.path(), 1))?;
                    }
                    self.steps.pop();
                    continue;
                }
            };
            if !(field.skip_null && value.is_null()) {
                self.member(field, &mut written, depth);
                self.value(&field.ty, value, depth + 1)?;
            }
            self.steps.pop();
        }
        if written > 0 {
            self.line(depth);
        }
        self.out.push('}');
        Ok(())
    }

    /// A member's name on its own line, after a comma when it is not the first.
    fn member(&mut self, field: &Field, written: &mut usize, depth: usize) {
        if *written > 0 {
            self.out.push(',');
        }
        *written += 1;
        self.line(depth + 1);
        self.out.push_str(&json::quoted(field.name));
        self.out.push_str(": ");
    }

    fn finish(mut self, ty: &Type, value: &'a Value, depth: usize) -> Result<String, String> {
        self.value(ty, value, depth)?;
        Ok(self.out)
    }
}

/// A bound document as the bytes .NET writes for it: indented by two, fields in the schema's order, numbers at their fields' widths.
pub fn format(schema: &'static Schema, bound: &Value) -> Result<String, String> {
    let mut writer = Writer { out: String::new(), steps: Vec::new() };
    writer.object(schema, bound, 0)?;
    Ok(writer.out)
}

/// A bound document as compact JSON for a reader that binds it again: every settable field, numbers by their tokens.
pub fn transport(bound: &Value) -> String {
    let mut out = String::new();
    json::write_compact(&mut out, bound);
    out
}

/// One line for each field, `Class.Name<TAB>type` and then what is unusual about it, each class once and the classes it holds after it: what a host compares its own model against.
pub fn describe(schema: &'static Schema) -> String {
    fn name(ty: &Type) -> String {
        match ty {
            Type::String => "string".to_string(),
            Type::Bool => "bool".to_string(),
            Type::NullableBool => "bool?".to_string(),
            Type::Int32 => "int".to_string(),
            Type::NullableInt32 => "int?".to_string(),
            Type::Int64 => "long".to_string(),
            Type::Double => "double".to_string(),
            Type::Single => "float".to_string(),
            Type::Object(schema) => schema.name.to_string(),
            Type::List(item) => format!("list<{}>", name(item)),
            Type::Map(item) => format!("map<{}>", name(item)),
        }
    }
    fn class_of(ty: &Type) -> Option<&'static Schema> {
        match ty {
            Type::Object(schema) => Some(schema),
            Type::List(item) | Type::Map(item) => class_of(item),
            _ => None,
        }
    }
    fn walk(schema: &'static Schema, seen: &mut Vec<&'static str>, out: &mut String) {
        if seen.contains(&schema.name) {
            return;
        }
        seen.push(schema.name);
        for field in schema.fields {
            let kind = if field.derived.is_some() { "\tderived" } else if field.skip_null { "\tskipped when null" } else { "" };
            out.push_str(&format!("{}.{}\t{}{kind}\n", schema.name, field.name, name(&field.ty)));
        }
        for field in schema.fields {
            if let Some(inner) = class_of(&field.ty) {
                walk(inner, seen, out);
            }
        }
    }
    let mut out = String::new();
    walk(schema, &mut Vec::new(), &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    static INNER: Schema = Schema { name: "Inner", fields: &[Field::new("S", Type::String, Initial::Str("d")), Field::new("I", Type::Int32, Initial::Number("7"))] };
    static STRINGS: Type = Type::String;
    static LONGS: Type = Type::Int64;
    static NULLABLE_INTS: Type = Type::NullableInt32;
    fn twice(object: &Value) -> Result<Value, String> {
        match object.get("I") {
            Some(Value::Number(token)) => Ok(Value::number(token.parse::<i64>().map_err(|e| e.to_string())? * 2)),
            _ => Err("no I".to_string()),
        }
    }
    static M: Schema = Schema {
        name: "M",
        fields: &[
            Field::new("S", Type::String, Initial::Null),
            Field::new("I", Type::Int32, Initial::Number("0")),
            Field::new("L", Type::Int64, Initial::Number("0")),
            Field::new("Db", Type::Double, Initial::Number("0")),
            Field::new("F", Type::Single, Initial::Number("0")),
            Field::new("B", Type::Bool, Initial::Bool(false)),
            Field::new("NB", Type::NullableBool, Initial::Null).skipped_when_null(),
            Field::new("NI", Type::NullableInt32, Initial::Null),
            Field::new("O", Type::Object(&INNER), Initial::New),
            Field::new("List", Type::List(&LONGS), Initial::Strings(&[])),
            Field::new("SL", Type::List(&STRINGS), Initial::Strings(&["x"])),
            Field::new("D", Type::Map(&NULLABLE_INTS), Initial::EmptyMap),
            Field::derived("Twice", Type::Int64, twice),
        ],
    };

    fn bound(text: &str) -> Result<Option<String>, BindError> {
        M.bind(&json::parse(text).expect("the text is JSON")).map(|document| document.map(|document| transport(&document)))
    }

    fn reads(text: &str) -> String {
        bound(text).unwrap().unwrap()
    }

    const NEW: &str = r#"{"S":null,"I":0,"L":0,"Db":0,"F":0,"B":false,"NB":null,"NI":null,"O":{"S":"d","I":7},"List":[],"SL":["x"],"D":{}}"#;

    #[test]
    fn a_field_with_no_member_keeps_its_initial_value_and_an_unknown_member_is_ignored() {
        assert_eq!(reads("{}"), NEW);
        assert_eq!(reads(r#"{"Unknown":[1,{"deep":"\ud800"}],"Twice":"anything"}"#), NEW);
        assert_eq!(bound("null"), Ok(None));
        assert!(bound("[]").is_err());
        assert!(bound("5").is_err());
    }

    #[test]
    fn names_match_without_case_and_the_last_member_wins_whole() {
        assert_eq!(reads(r#"{"i":1,"I":2}"#), NEW.replace(r#""I":0"#, r#""I":2"#));
        assert_eq!(reads(r#"{"s":"a","S":null}"#), NEW);
        assert_eq!(reads(r#"{"O":{"S":"a"},"o":{"I":5}}"#), NEW.replace(r#"{"S":"d","I":7}"#, r#"{"S":"d","I":5}"#));
        assert_eq!(reads(r#"{"O":{},"O":null}"#), NEW.replace(r#"{"S":"d","I":7}"#, "null"));
        assert_eq!(reads(r#"{"List":[1],"LIST":[2,3]}"#), NEW.replace(r#""List":[]"#, r#""List":[2,3]"#));
        assert_eq!(reads("{\"\u{17F}\":\"long s\"}"), NEW);
    }

    #[test]
    fn a_map_keeps_a_repeated_keys_first_place_and_last_value_and_its_keys_keep_their_case() {
        assert_eq!(reads(r#"{"D":{"a":1,"b":2,"a":3,"A":4}}"#), NEW.replace(r#""D":{}"#, r#""D":{"a":3,"b":2,"A":4}"#));
        assert_eq!(reads(r#"{"D":{"a":1},"d":{"b":null}}"#), NEW.replace(r#""D":{}"#, r#""D":{"b":null}"#));
        assert_eq!(reads(r#"{"D":null}"#), NEW.replace(r#""D":{}"#, r#""D":null"#));
    }

    #[test]
    fn each_type_takes_what_dotnet_takes_for_it() {
        for text in [
            r#"{"I":-0}"#, r#"{"I":2147483647}"#, r#"{"I":-2147483648}"#, r#"{"L":9223372036854775807}"#, r#"{"L":-9223372036854775808}"#, r#"{"Db":1e999}"#, r#"{"F":1e39}"#,
            r#"{"Db":1E5}"#, r#"{"NB":null}"#, r#"{"NI":null}"#, r#"{"NI":5}"#, r#"{"S":null}"#, r#"{"O":null}"#, r#"{"List":null}"#, r#"{"SL":[null,"a"]}"#, r#"{"D":{"a":null}}"#,
            r#"{"S":"\ud83d\ude00"}"#,
        ] {
            assert!(bound(text).is_ok(), "{text}");
        }
        for text in [
            r#"{"I":1e2}"#, r#"{"I":1.0}"#, r#"{"I":2147483648}"#, r#"{"I":"1"}"#, r#"{"I":null}"#, r#"{"I":true}"#, r#"{"L":9223372036854775808}"#, r#"{"L":1.0}"#, r#"{"L":null}"#,
            r#"{"Db":null}"#, r#"{"F":"1"}"#, r#"{"F":null}"#, r#"{"B":null}"#, r#"{"B":1}"#, r#"{"B":"true"}"#, r#"{"NB":0}"#, r#"{"NI":1.0}"#, r#"{"NI":"1"}"#, r#"{"S":1}"#,
            r#"{"O":[]}"#, r#"{"List":{}}"#, r#"{"List":[null]}"#, r#"{"D":[]}"#, r#"{"S":"\ud800"}"#, r#"{"\ud800":1}"#, r#"{"D":{"\ud800":1}}"#, r#"{"O":{"i":"bad"}}"#,
        ] {
            assert!(bound(text).is_err(), "{text}");
        }
        assert_eq!(bound(r#"{"O":{"i":"bad"}}"#).unwrap_err().to_string(), "$.O.I is not a whole number.");
    }

    #[test]
    fn a_document_is_written_as_dotnet_writes_it() {
        let document = M.bind(&json::parse(r#"{"S":"é","I":-0,"Db":1.50,"F":0.1,"NB":true,"List":[1,2],"SL":[],"D":{"z":1,"a":null},"O":null}"#).unwrap()).unwrap().unwrap();
        let expected = "{\n  \"S\": \"\\u00E9\",\n  \"I\": 0,\n  \"L\": 0,\n  \"Db\": 1.5,\n  \"F\": 0.1,\n  \"B\": false,\n  \"NB\": true,\n  \"NI\": null,\n  \"O\": null,\n  \"List\": [\n    1,\n    2\n  ],\n  \"SL\": [],\n  \"D\": {\n    \"z\": 1,\n    \"a\": null\n  },\n  \"Twice\": 0\n}";
        assert_eq!(format(&M, &document).unwrap(), expected.replace('\n', NEW_LINE));
    }

    #[test]
    fn a_null_that_is_skipped_leaves_no_line_and_an_infinite_number_cannot_be_written() {
        let new = M.new_instance();
        let text = format(&M, &new).unwrap();
        assert!(!text.contains("NB"));
        assert!(text.contains("\"O\": {\n    \"S\": \"d\",\n    \"I\": 7\n  }".replace('\n', NEW_LINE).as_str()));
        let infinite = M.bind(&json::parse(r#"{"Db":1e999}"#).unwrap()).unwrap().unwrap();
        assert_eq!(format(&M, &infinite), Err("$.Db is not a number JSON can hold.".to_string()));
        static EMPTY: Schema = Schema { name: "Empty", fields: &[] };
        assert_eq!(format(&EMPTY, &EMPTY.new_instance()).unwrap(), "{}");
    }

    #[test]
    fn a_schema_lists_itself_and_the_classes_it_holds() {
        let listing = describe(&M);
        assert!(listing.starts_with("M.S\tstring\nM.I\tint\n"));
        assert!(listing.contains("M.NB\tbool?\tskipped when null\n"));
        assert!(listing.contains("M.D\tmap<int?>\n"));
        assert!(listing.contains("M.Twice\tlong\tderived\n"));
        assert!(listing.ends_with("Inner.S\tstring\nInner.I\tint\n"));
    }
}
