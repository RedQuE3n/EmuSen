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

    fn set_declared(&mut self, list: Vec<Declared>) {
        self.values.retain(|k, _| list.iter().any(|d| d.key == *k));
        for d in &list {
            self.values.entry(d.key.clone()).or_insert_with(|| CString::new(d.default.as_str()).unwrap_or_default());
        }
        self.declared = list;
    }

    /// The value the core reads, valid until the value changes.
    pub fn get(&self, key: &str) -> Option<*const c_char> {
        self.values.get(key).map(|v| v.as_ptr())
    }

    /// A value from EmuSen's settings; false for a key not declared or a value outside its choices.
    pub fn set(&mut self, key: &str, value: &str) -> bool {
        let Some(d) = self.declared.iter().find(|d| d.key == key) else { return false };
        if !d.values.iter().any(|(v, _)| v == value) {
            return false;
        }
        if self.values.get(key).is_none_or(|old| old.to_bytes() != value.as_bytes()) {
            self.values.insert(key.to_owned(), CString::new(value).unwrap_or_default());
            self.updated = true;
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
    fn a_key_outside_the_schemas_characters_is_found() {
        let mut o = Options::default();
        o.set_declared(vec![Declared { key: "bad key".into(), label: String::new(), info: String::new(), category: None, values: vec![("x".into(), "x".into())], default: "x".into() }]);
        assert_eq!(o.bad_key(), Some("bad key"));
    }
}
