//! Core options: declared by the core in any of libretro's three forms, kept as one list, given to the core by
//! `GET_VARIABLE` and to EmuSen as the settings schema (EmuSen_LibretroHost.md §4.7).

use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char};

use emusen_libretro_sys as sys;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declared {
    pub key: String,
    pub label: String,
    pub info: String,
    pub category: Option<String>,
    /// (value, label), in the core's order; the label is the value where the core gives none.
    pub values: Vec<(String, String)>,
    pub default: String,
}

#[derive(Default)]
pub struct Options {
    pub declared: Vec<Declared>,
    /// Category key to its description.
    pub categories: HashMap<String, String>,
    values: HashMap<String, CString>,
    /// A value changed since the core last asked `GET_VARIABLE_UPDATE`.
    pub updated: bool,
    /// Keys the core has hidden for the current values.
    pub hidden: HashMap<String, bool>,
    /// Keys whose value the core changed itself, for the notes.
    pub changed_by_core: Vec<String>,
    pub update_display: sys::retro_core_options_update_display_callback_t,
    /// Values held over the player's while the console's own picture is asked for, by key (`console_picture`).
    forced: HashMap<String, CString>,
    /// Defaults the host gives in place of the core's, (key, value), from the core's exact defaults.
    pub exact: Vec<(String, String)>,
}

/// Text of a C string the core owns, or empty.
pub fn text(p: *const c_char) -> String {
    if p.is_null() { String::new() } else { unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned() }
}

fn values_of(list: &[sys::retro_core_option_value; sys::NUM_CORE_OPTION_VALUES_MAX]) -> Vec<(String, String)> {
    list.iter()
        .take_while(|v| !v.value.is_null())
        .map(|v| {
            let value = text(v.value);
            let label = if v.label.is_null() { value.clone() } else { text(v.label) };
            (value, label)
        })
        .collect()
}

fn with_default(values: &[(String, String)], default: *const c_char) -> String {
    let d = text(default);
    if values.iter().any(|(v, _)| *v == d) { d } else { values.first().map(|v| v.0.clone()).unwrap_or_default() }
}

impl Options {
    /// The options of `SET_CORE_OPTIONS_V2`, replacing any declared before.
    pub unsafe fn declare_v2(&mut self, options: *const sys::retro_core_options_v2) {
        let Some(o) = (unsafe { options.as_ref() }) else { return };
        self.categories.clear();
        let mut c = o.categories;
        while !c.is_null() && !unsafe { (*c).key }.is_null() {
            let cat = unsafe { &*c };
            self.categories.insert(text(cat.key), text(cat.desc));
            c = unsafe { c.add(1) };
        }
        let mut list = Vec::new();
        let mut d = o.definitions;
        while !d.is_null() && !unsafe { (*d).key }.is_null() {
            let def = unsafe { &*d };
            let values = values_of(&def.values);
            let category = Some(text(def.category_key)).filter(|k| !k.is_empty() && self.categories.contains_key(k));
            let label = if category.is_some() && !def.desc_categorized.is_null() { text(def.desc_categorized) } else { text(def.desc) };
            let info = if category.is_some() && !def.info_categorized.is_null() { text(def.info_categorized) } else { text(def.info) };
            list.push(Declared { key: text(def.key), label, info, category, default: with_default(&values, def.default_value), values });
            d = unsafe { d.add(1) };
        }
        self.set_declared(list);
    }

    /// The options of `SET_CORE_OPTIONS`, version 1.
    pub unsafe fn declare_v1(&mut self, mut d: *const sys::retro_core_option_definition) {
        let mut list = Vec::new();
        while !d.is_null() && !unsafe { (*d).key }.is_null() {
            let def = unsafe { &*d };
            let values = values_of(&def.values);
            list.push(Declared { key: text(def.key), label: text(def.desc), info: text(def.info), category: None, default: with_default(&values, def.default_value), values });
            d = unsafe { d.add(1) };
        }
        self.categories.clear();
        self.set_declared(list);
    }

    /// The oldest form, `SET_VARIABLES`: "Description; first|second|third", the first the default.
    pub unsafe fn declare_legacy(&mut self, mut v: *const sys::retro_variable) {
        let mut list = Vec::new();
        while !v.is_null() && !unsafe { (*v).key }.is_null() {
            let var = unsafe { &*v };
            let spec = text(var.value);
            let (label, rest) = spec.split_once(';').unwrap_or((spec.as_str(), ""));
            let values: Vec<(String, String)> = rest.trim_start().split('|').filter(|s| !s.is_empty()).map(|s| (s.to_owned(), s.to_owned())).collect();
            let default = values.first().map(|x| x.0.clone()).unwrap_or_default();
            list.push(Declared { key: text(var.key), label: label.trim().to_owned(), info: String::new(), category: None, default, values });
            v = unsafe { v.add(1) };
        }
        self.categories.clear();
        self.set_declared(list);
    }

    fn set_declared(&mut self, mut list: Vec<Declared>) {
        for d in &mut list {
            if let Some((_, v)) = self.exact.iter().find(|(k, v)| *k == d.key && d.values.iter().any(|x| x.0 == *v)) {
                d.default = v.clone();
            }
        }
        self.values.retain(|k, _| list.iter().any(|d| d.key == *k));
        for d in &list {
            self.values.entry(d.key.clone()).or_insert_with(|| CString::new(d.default.as_str()).unwrap_or_default());
        }
        self.declared = list;
    }

    /// The value the core reads, valid until the value changes: a held value over the player's.
    pub fn get(&self, key: &str) -> Option<*const c_char> {
        self.forced.get(key).or_else(|| self.values.get(key)).map(|v| v.as_ptr())
    }

    fn effective(&self) -> Vec<(String, String)> {
        self.declared.iter().map(|d| (d.key.clone(), self.get(&d.key).map(text).unwrap_or_default())).collect()
    }

    /// Holds these values over the player's, or with none gives the player's back; the keys held, those declared with
    /// the value among their choices.
    pub fn hold(&mut self, held: &[(String, String)]) -> Vec<String> {
        let before = self.effective();
        self.forced = held
            .iter()
            .filter(|(k, v)| self.declared.iter().any(|d| d.key == *k && d.values.iter().any(|c| c.0 == *v)))
            .map(|(k, v)| (k.clone(), CString::new(v.as_str()).unwrap_or_default()))
            .collect();
        if self.effective() != before {
            self.updated = true;
        }
        self.declared.iter().map(|d| d.key.clone()).filter(|k| self.forced.contains_key(k)).collect()
    }

    /// Whether a key's value is held over the player's.
    pub fn held(&self, key: &str) -> bool {
        self.forced.contains_key(key)
    }

    /// A value from EmuSen's settings; false for a key not declared or a value outside its choices.
    pub fn set(&mut self, key: &str, value: &str) -> bool {
        let Some(d) = self.declared.iter().find(|d| d.key == key) else { return false };
        if !d.values.iter().any(|(v, _)| v == value) {
            return false;
        }
        if self.values.get(key).is_none_or(|old| old.to_bytes() != value.as_bytes()) {
            self.values.insert(key.to_owned(), CString::new(value).unwrap_or_default());
            self.updated |= !self.forced.contains_key(key);
        }
        true
    }

    /// `SET_VARIABLE`: the core changing its own option, which EmuSen is told of.
    pub fn set_by_core(&mut self, key: &str, value: &str) -> bool {
        let known = self.declared.iter().any(|d| d.key == key && d.values.iter().any(|(v, _)| v == value));
        if known {
            self.values.insert(key.to_owned(), CString::new(value).unwrap_or_default());
            if !self.changed_by_core.iter().any(|k| k == key) {
                self.changed_by_core.push(key.to_owned());
            }
        }
        known
    }

    /// The current value of every option, as `key=value` lines.
    pub fn current(&self) -> Vec<(String, String)> {
        self.declared.iter().map(|d| (d.key.clone(), self.values.get(&d.key).map(|v| v.to_string_lossy().into_owned()).unwrap_or_default())).collect()
    }

    /// The declarations as lines a later load reads back: `category`, `option` and `value` records, tab-separated.
    pub fn to_text(&self) -> String {
        let clean = |s: &str| s.replace(['\t', '\n', '\r'], " ");
        let mut out = String::new();
        for (k, d) in &self.categories {
            out += &format!("category\t{}\t{}\n", clean(k), clean(d));
        }
        for o in &self.declared {
            out += &format!("option\t{}\t{}\t{}\t{}\t{}\n", clean(&o.key), o.category.as_deref().map_or("-".into(), clean), clean(&o.default), clean(&o.label), clean(&o.info));
            for (v, l) in &o.values {
                out += &format!("value\t{}\t{}\t{}\n", clean(&o.key), clean(v), clean(l));
            }
        }
        out
    }

    /// Declarations a previous load recorded, added where the core has not declared the key itself.
    pub fn merge_text(&mut self, text: &str) {
        let mut added: Vec<Declared> = Vec::new();
        for line in text.lines() {
            let f: Vec<&str> = line.split('\t').collect();
            match f.as_slice() {
                ["category", k, d] => {
                    self.categories.entry(k.to_string()).or_insert_with(|| d.to_string());
                }
                ["option", k, c, default, label, info] if !self.declared.iter().any(|o| o.key == *k) => added.push(Declared {
                    key: k.to_string(),
                    label: label.to_string(),
                    info: info.to_string(),
                    category: (*c != "-").then(|| c.to_string()),
                    values: Vec::new(),
                    default: default.to_string(),
                }),
                ["value", k, v, l] => {
                    if let Some(o) = added.iter_mut().find(|o| o.key == *k) {
                        o.values.push((v.to_string(), l.to_string()));
                    }
                }
                _ => {}
            }
        }
        let mut list = std::mem::take(&mut self.declared);
        list.extend(added.into_iter().filter(|o| o.values.iter().any(|v| v.0 == o.default)));
        self.set_declared(list);
    }

    /// Whether every key is one the settings schema can carry.
    pub fn bad_key(&self) -> Option<&str> {
        self.declared.iter().map(|d| d.key.as_str()).find(|k| k.is_empty() || !k.bytes().all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_oldest_form_reads_its_description_and_choices() {
        let vars = [
            sys::retro_variable { key: c"a_key".as_ptr(), value: c"A setting; one|two|three".as_ptr() },
            sys::retro_variable { key: std::ptr::null(), value: std::ptr::null() },
        ];
        let mut o = Options::default();
        unsafe { o.declare_legacy(vars.as_ptr()) };
        assert_eq!(o.declared[0].label, "A setting");
        assert_eq!(o.declared[0].values.iter().map(|v| v.0.as_str()).collect::<Vec<_>>(), ["one", "two", "three"]);
        assert_eq!(text(o.get("a_key").unwrap()), "one");
        assert!(o.set("a_key", "two") && o.updated);
        assert!(!o.set("a_key", "four"));
        assert!(!o.set("other", "one"));
    }

    #[test]
    fn declarations_recorded_by_one_load_join_the_next_ones_list() {
        let mut first = Options::default();
        first.set_declared(vec![
            Declared { key: "early".into(), label: "Early".into(), info: String::new(), category: None, values: vec![("a".into(), "A".into())], default: "a".into() },
            Declared { key: "late".into(), label: "Late\tone".into(), info: "help".into(), category: Some("cat".into()), values: vec![("x".into(), "X".into()), ("y".into(), "Y".into())], default: "y".into() },
        ]);
        first.categories.insert("cat".into(), "Category".into());
        let mut next = Options::default();
        next.set_declared(vec![Declared { key: "early".into(), label: "Early, again".into(), info: String::new(), category: None, values: vec![("a".into(), "A".into())], default: "a".into() }]);
        next.merge_text(&first.to_text());
        assert_eq!(next.declared.iter().map(|o| o.key.as_str()).collect::<Vec<_>>(), ["early", "late"]);
        assert_eq!(next.declared[0].label, "Early, again", "the core's own declaration stands");
        assert_eq!((next.declared[1].label.as_str(), next.declared[1].values.len(), text(next.get("late").unwrap())), ("Late one", 2, "y".to_owned()));
        assert_eq!(next.categories.get("cat").map(String::as_str), Some("Category"));
    }

    #[test]
    fn a_held_value_is_read_over_the_players_and_released_back_to_it() {
        let vars = [
            sys::retro_variable { key: c"scale".as_ptr(), value: c"Scale; 1|2".as_ptr() },
            sys::retro_variable { key: c"other".as_ptr(), value: c"Other; a|b".as_ptr() },
            sys::retro_variable { key: std::ptr::null(), value: std::ptr::null() },
        ];
        let mut o = Options::default();
        unsafe { o.declare_legacy(vars.as_ptr()) };
        assert!(o.set("scale", "2"));
        o.updated = false;
        let held = o.hold(&[("scale".into(), "1".into()), ("other".into(), "z".into()), ("absent".into(), "1".into())]);
        assert_eq!((held, text(o.get("scale").unwrap()), o.updated), (vec!["scale".to_owned()], "1".to_owned(), true));
        o.updated = false;
        assert!(o.set("scale", "2") && !o.updated, "the player's value changes nothing the core reads while held");
        assert!(o.hold(&[]).is_empty());
        assert_eq!((text(o.get("scale").unwrap()), o.updated), ("2".to_owned(), true));
    }

    #[test]
    fn an_exact_default_replaces_the_cores_and_a_value_it_lacks_does_not() {
        let mut o = Options { exact: vec![("threads".into(), "off".into()), ("scale".into(), "9".into())], ..Options::default() };
        let choice = |key: &str, default: &str| Declared { key: key.into(), label: String::new(), info: String::new(), category: None, values: vec![("on".into(), "on".into()), ("off".into(), "off".into()), ("1".into(), "1".into())], default: default.into() };
        o.set_declared(vec![choice("threads", "on"), choice("scale", "1")]);
        assert_eq!(o.current(), [("threads".to_owned(), "off".to_owned()), ("scale".to_owned(), "1".to_owned())]);
        assert_eq!(o.declared[0].default, "off");
        o.set("threads", "on");
        o.set_declared(vec![choice("threads", "on"), choice("scale", "1")]);
        assert_eq!(o.current()[0].1, "on", "the player's choice of the core's default stands");
    }

    #[test]
    fn a_key_outside_the_schemas_characters_is_found() {
        let mut o = Options::default();
        o.set_declared(vec![Declared { key: "bad key".into(), label: String::new(), info: String::new(), category: None, values: vec![("x".into(), "x".into())], default: "x".into() }]);
        assert_eq!(o.bad_key(), Some("bad key"));
    }
}
