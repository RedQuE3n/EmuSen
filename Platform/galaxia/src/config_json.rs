//! The JSON of the files a person may edit: read by jsonc-parser with comments and a trailing comma allowed and nothing
//! looser, member names matched to a model's fields without regard to case and the last of two winning; written by
//! serde_json, indented by two. The one place those rules live (EmuSen_RustPlatform.md §47.19).

use serde::de::{self, DeserializeOwned, DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::Serialize;
use serde_json::{Error, Map, Value};

/// Comments and a trailing comma, as a hand-edited file may hold; every other JSON5 liberty refused.
fn options() -> jsonc_parser::ParseOptions {
    jsonc_parser::ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
        allow_bare_decimal_point_numbers: false,
        allow_non_finite_numbers: false,
        allow_extended_string_escapes: false,
    }
}

/// Text parsed as any type serde reads, by the hand-edited file's rules; text with no value in it, only spaces or
/// comments, is refused, where the serde reader alone would take it for `null`.
pub fn parse<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    if jsonc_parser::parse_to_value(text, &options()).map_err(|error| sentence(error.to_string()))?.is_none() {
        return Err("The file holds no JSON.".to_string());
    }
    jsonc_parser::parse_to_serde_value(text, &options()).map_err(|error| sentence(error.to_string()))
}

/// Words ended as a sentence, since a diagnostic follows them with one of its own; a question ends one too.
fn sentence(mut words: String) -> String {
    if !words.ends_with(['.', '?']) {
        words.push('.');
    }
    words
}

/// Text read as a model: parsed, then each object's member names matched to the fields its type declares.
pub fn read<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    let value: Value = parse(text)?;
    T::deserialize(Folded(value)).map_err(|error| sentence(error.to_string()))
}

/// A model as its file's text: two spaces, a line feed between lines, none at the end.
pub fn write<T: Serialize>(model: &T) -> Result<String, String> {
    serde_json::to_string_pretty(model).map_err(|error| error.to_string())
}

/// The words of a refusal inside a member or an item, the path to it in front: `$.Cheats[0].Writes`.
fn within(step: &str, error: Error) -> Error {
    let words = error.to_string();
    match words.strip_prefix('$') {
        Some(rest) => de::Error::custom(format!("${step}{rest}")),
        None => de::Error::custom(format!("${step}: {words}")),
    }
}

/// What a value is, for the words of a refusal.
fn unexpected(value: &Value) -> de::Unexpected<'_> {
    match value {
        Value::Null => de::Unexpected::Unit,
        Value::Bool(b) => de::Unexpected::Bool(*b),
        Value::Number(n) => n.as_f64().map_or(de::Unexpected::Other("a number"), de::Unexpected::Float),
        Value::String(s) => de::Unexpected::Str(s),
        Value::Array(_) => de::Unexpected::Seq,
        Value::Object(_) => de::Unexpected::Map,
    }
}

/// A parsed value on its way into a model.
struct Folded(Value);

impl<'de> Deserializer<'de> for Folded {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Array(items) => visitor.visit_seq(Items { items: items.into_iter(), index: 0 }),
            Value::Object(members) => visitor.visit_map(Members { members: members.into_iter(), pending: None }),
            other => other.deserialize_any(visitor),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        if self.0.is_null() { visitor.visit_none() } else { visitor.visit_some(self) }
    }

    fn deserialize_struct<V: Visitor<'de>>(self, _: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        let members = match self.0 {
            Value::Object(members) => members,
            other => return Err(de::Error::invalid_type(unexpected(&other), &visitor)),
        };
        let mut folded = Map::new();
        for (name, value) in members {
            let name = fields.iter().find(|field| field.eq_ignore_ascii_case(&name)).map_or(name, |field| field.to_string());
            folded.insert(name, value);
        }
        visitor.visit_map(Members { members: folded.into_iter(), pending: None })
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _: &'static str, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Error> {
        self.0.deserialize_enum(name, variants, visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map identifier ignored_any
    }
}

struct Items {
    items: std::vec::IntoIter<Value>,
    index: usize,
}

impl<'de> SeqAccess<'de> for Items {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Error> {
        let Some(item) = self.items.next() else { return Ok(None) };
        self.index += 1;
        seed.deserialize(Folded(item)).map(Some).map_err(|error| within(&format!("[{}]", self.index - 1), error))
    }
}

struct Members {
    members: serde_json::map::IntoIter,
    pending: Option<(String, Value)>,
}

impl<'de> MapAccess<'de> for Members {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Error> {
        let Some((name, value)) = self.members.next() else { return Ok(None) };
        let key = seed.deserialize(IntoDeserializer::<Error>::into_deserializer(name.clone()))?;
        self.pending = Some((name, value));
        Ok(Some(key))
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let (name, value) = self.pending.take().ok_or_else(|| <Error as de::Error>::custom("a value was asked for before its name"))?;
        seed.deserialize(Folded(value)).map_err(|error| within(&format!(".{name}"), error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
    #[serde(rename_all = "PascalCase", default)]
    struct Inner {
        word: Option<String>,
        count: i32,
    }

    #[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
    #[serde(rename_all = "PascalCase", default)]
    struct Outer {
        inner: Option<Inner>,
        list: Option<Vec<Inner>>,
        map: Option<indexmap::IndexMap<String, Option<i32>>>,
        volume: f64,
    }

    #[test]
    fn names_fold_at_every_depth_and_the_last_of_two_wins() {
        let read: Outer = read(r#"{"INNER": {"word": "a", "COUNT": 2}, "list": [{"count": 3}], "Volume": 0.5, "volume": 0.7, /* by hand */ "map": {"a": 1, "A": 2, "a": null},}"#).unwrap();
        assert_eq!(read.inner, Some(Inner { word: Some("a".to_string()), count: 2 }));
        assert_eq!(read.list, Some(vec![Inner { word: None, count: 3 }]));
        assert_eq!(read.volume, 0.7);
        assert_eq!(read.map.unwrap().into_iter().collect::<Vec<_>>(), vec![("a".to_string(), None), ("A".to_string(), Some(2))]);
    }

    #[test]
    fn a_refusal_names_where_it_is_and_looser_json_is_refused() {
        assert_eq!(read::<Outer>(r#"{"List": [{}, {"Count": 1.5}]}"#).unwrap_err(), "$.List[1].Count: invalid type: floating point `1.5`, expected i32.");
        for text in ["{'Volume': 1}", "{Volume: 1}", "{\"Volume\": 0x10}", "{\"Volume\": +1}", "{\"Volume\": .5}", "{\"Volume\": NaN}", "{\"Volume\": 1 \"Map\": null}", "", "  ", "// only a comment", "/* */", "{} {}"] {
            assert!(read::<Outer>(text).is_err(), "{text}");
        }
        assert_eq!(read::<Option<Outer>>("null").unwrap(), None);
    }

    #[test]
    fn a_model_is_written_indented_by_two() {
        assert_eq!(write(&Inner { word: None, count: 1 }).unwrap(), "{\n  \"Word\": null,\n  \"Count\": 1\n}");
    }
}
