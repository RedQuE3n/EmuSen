//! One pass over a theme's files for one variant, in THEMES.md's parsing order: includes, variables and selection
//! blocks, with each view's element definitions handed to its builder (EmuSen_BigPicture.md §12.2, §31, §35).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::capabilities::{self, Capabilities, FONT_SIZE_ORDER};
use super::catalog::{catalog, Shape};
use super::choices::{Selection, ThemeSystem};
use super::diagnostics::{Code, Diagnostic, Diagnostics, Severity};
use super::resolved::View;
use super::values::{resolve_path, split};
use super::view::{Raw, ViewBuilder};
use super::xml::{self, Node};

pub const MAX_INCLUDE_DEPTH: usize = 64;
const VIEW_NAMES: [&str; 3] = ["system", "gamelist", "all"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Block {
    Theme,
    Variant,
    AspectRatio,
    Language,
}

fn allowed(kind: Block) -> &'static [&'static str] {
    match kind {
        Block::Theme => &["variables", "colorScheme", "fontSize", "language", "include", "view", "variant", "aspectRatio"],
        Block::Variant => &["transitions", "variables", "colorScheme", "fontSize", "language", "include", "view", "aspectRatio"],
        Block::AspectRatio => &["variables", "colorScheme", "fontSize", "language", "include", "view"],
        Block::Language => &["variables", "include"],
    }
}

pub struct ParseRun<'a> {
    capabilities: &'a Capabilities,
    pub selection: &'a Selection,
    variant: Option<&'a str>,
    pub system: &'a ThemeSystem,
    variables: HashMap<String, String>,
    views: Vec<ViewBuilder>,
    include_stack: Vec<String>,
    warned_names: HashSet<String>,
    order: usize,
    variant_depth: usize,
    pub diagnostics: Diagnostics,
    pub files_read: Vec<String>,
    pub variant_transitions: Option<String>,
}

impl<'a> ParseRun<'a> {
    pub fn run(
        capabilities: &'a Capabilities,
        selection: &'a Selection,
        variant: Option<&'a str>,
        system: &'a ThemeSystem,
        entry: &str,
    ) -> Self {
        let mut run = ParseRun {
            capabilities,
            selection,
            variant,
            system,
            variables: system.variables().into_iter().collect(),
            views: VIEW_NAMES.iter().map(|v| ViewBuilder::new(v)).collect(),
            include_stack: vec![],
            warned_names: HashSet::new(),
            order: 0,
            variant_depth: 0,
            diagnostics: Diagnostics::default(),
            files_read: vec![],
            variant_transitions: None,
        };
        run.process_file(entry, 0);
        run
    }

    /// A view resolved, its new diagnostics added to `out` unless already seen.
    pub fn finish(&self, view: &str, out: &mut Diagnostics, seen: &mut HashSet<Diagnostic>) -> View {
        let mut local = Diagnostics::default();
        let builder = self.views.iter().find(|v| v.name == view).expect("a known view");
        let resolved = builder.resolve(self, &mut local);
        for d in local.items {
            if seen.insert(d.clone()) {
                out.items.push(d);
            }
        }
        resolved
    }

    fn error(&mut self, code: Code, file: &str, line: u32, message: impl Into<String>) {
        self.diagnostics.add(Severity::Error, code, file, line, message);
    }
    fn warn(&mut self, code: Code, file: &str, line: u32, message: impl Into<String>) {
        self.diagnostics.add(Severity::Warning, code, file, line, message);
    }
    fn debug(&mut self, code: Code, file: &str, line: u32, message: impl Into<String>) {
        self.diagnostics.add(Severity::Debug, code, file, line, message);
    }

    fn process_file(&mut self, file: &str, line: u32) {
        if self.include_stack.iter().any(|f| f == file) || self.include_stack.len() >= MAX_INCLUDE_DEPTH {
            // ES-DE does not check for loops and hangs; the loader refuses the include instead.
            let from = self.include_stack.last().cloned().unwrap_or_else(|| file.to_owned());
            self.error(Code::IncludeLoop, &from, line, format!("including \"{file}\" again would loop"));
            return;
        }
        let text = match std::fs::read(file) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            Err(e) => {
                self.error(Code::MalformedXml, file, 0, e.to_string());
                return;
            }
        };
        let root = match xml::parse(&text) {
            Ok((root, leniency)) => {
                if let Some(l) = leniency {
                    self.warn(Code::LenientXml, file, 0, format!("not well-formed XML, read as ES-DE reads it: {l}"));
                }
                root
            }
            Err(e) => {
                self.error(Code::MalformedXml, file, e.line, e.message);
                return;
            }
        };
        self.files_read.push(file.to_owned());
        if root.name != "theme" {
            self.error(Code::WrongRoot, file, root.line, format!("root is <{}>, not <theme>", root.name));
            return;
        }
        self.include_stack.push(file.to_owned());
        self.process_block(&root, file, Block::Theme);
        self.include_stack.pop();
    }

    // Steps 1-9 of "Configuration parsing order", each in file order; an include runs all nine at its place.
    fn process_block(&mut self, block: &Node, file: &str, kind: Block) {
        let mut children: Vec<&Node> = vec![];
        for child in &block.children {
            let tag = child.name.as_str();
            if allowed(kind).contains(&tag) {
                children.push(child);
                continue;
            }
            if tag == "transitions" && kind == Block::Theme {
                if self.variant_depth > 0 {
                    children.push(child);
                } else {
                    self.warn(Code::IgnoredTag, file, child.line, "<transitions> outside any <variant> is ignored, as ES-DE ignores it");
                }
                continue;
            }
            let known = allowed(Block::Theme).contains(&tag) || tag == "transitions";
            if known {
                self.error(Code::MisplacedTag, file, child.line, format!("<{tag}> is not allowed inside <{}>", block.name));
            } else {
                self.diagnostics.add_about(Severity::Error, Code::UnknownTag, file, child.line, tag, format!("<{tag}> is not a theme tag"));
            }
        }
        let of = |name: &'static str| children.iter().copied().filter(move |c| c.name == name);

        for t in of("transitions") {
            self.transitions(t, file);
        }
        for v in of("variables") {
            self.define_variables(v);
        }
        let selection = self.selection;
        for c in of("colorScheme") {
            let caps = self.capabilities;
            if self.selected(
                c,
                file,
                Some("colour scheme"),
                &|n| caps.color_schemes.iter().any(|s| s.name == n),
                selection.color_scheme.as_deref(),
            ) {
                self.variables_only(c, file);
            }
        }
        for f in of("fontSize") {
            if self.selected(f, file, Some("font size"), &|n| FONT_SIZE_ORDER.contains(&n), selection.font_size.as_deref()) {
                self.variables_only(f, file);
            }
        }
        for l in of("language") {
            if self.selected(l, file, None, &|_| true, selection.language.as_deref()) {
                self.process_block(l, file, Block::Language);
            }
        }
        for i in of("include") {
            self.include(i, file);
        }
        for v in of("view") {
            self.view(v, file);
        }
        for v in of("variant") {
            if self.variant_applies(v, file) {
                self.variant_depth += 1;
                self.process_block(v, file, Block::Variant);
                self.variant_depth -= 1;
            }
        }
        for a in of("aspectRatio") {
            if self.selected(a, file, Some("aspect ratio"), &|n| capabilities::ratio(n).is_some(), selection.aspect_ratio.as_deref()) {
                self.process_block(a, file, Block::AspectRatio);
            }
        }
    }

    fn transitions(&mut self, element: &Node, file: &str) {
        let name = element.value.trim();
        if self.capabilities.transitions.iter().any(|t| t.name == name) {
            self.variant_transitions = Some(name.to_owned());
        } else {
            self.warn(Code::Undeclared, file, element.line, format!("transitions profile \"{name}\" is not declared in capabilities.xml"));
        }
    }

    // Variables live in one namespace, and a later definition replaces an earlier one.
    fn define_variables(&mut self, variables: &Node) {
        for variable in &variables.children {
            let value = self.substitute(&variable.value, false).0;
            self.variables.insert(variable.name.clone(), value);
        }
    }

    // Colour schemes and font sizes hold variables only; a bare child is taken as a variable, as THEMES.md's own example writes it.
    fn variables_only(&mut self, block: &Node, file: &str) {
        for child in &block.children {
            if child.name == "variables" {
                self.define_variables(child);
                continue;
            }
            self.warn(
                Code::MisplacedTag,
                file,
                child.line,
                format!("<{}> directly inside <{}> is taken as a variable", child.name, block.name),
            );
            let value = self.substitute(&child.value, false).0;
            self.variables.insert(child.name.clone(), value);
        }
    }

    fn selected(&mut self, block: &Node, file: &str, what: Option<&str>, known: &dyn Fn(&str) -> bool, chosen: Option<&str>) -> bool {
        let names = self.names(block, file);
        if let Some(what) = what {
            for name in names.iter().filter(|n| !known(n)) {
                if self.warned_names.insert(format!("{what}:{name}")) {
                    self.warn(Code::Undeclared, file, block.line, format!("{what} \"{name}\" is not declared"));
                }
            }
        }
        chosen.is_some_and(|c| names.iter().any(|n| n == c))
    }

    fn variant_applies(&mut self, block: &Node, file: &str) -> bool {
        let names = self.names(block, file);
        for name in names.iter().filter(|n| *n != "all" && !self.capabilities.is_declared_variant(n)) {
            if self.warned_names.insert(format!("variant:{name}")) {
                self.diagnostics.add(
                    Severity::Warning,
                    Code::Undeclared,
                    file,
                    block.line,
                    format!("variant \"{name}\" is not declared in capabilities.xml"),
                );
            }
        }
        names.iter().any(|n| n == "all") || self.variant.is_some_and(|v| names.iter().any(|n| n == v))
    }

    fn names(&mut self, block: &Node, file: &str) -> Vec<String> {
        let names = split(block.attribute("name").unwrap_or(""));
        if names.is_empty() {
            self.error(Code::MissingName, file, block.line, format!("<{}> has no name attribute", block.name));
        }
        names
    }

    // Explicit paths that are missing are errors, read untrimmed as ES-DE reads them; paths built from variables are skipped silently (§35.2).
    fn include(&mut self, include: &Node, file: &str) {
        let written = include.value.as_str();
        let from_variable = written.contains("${");
        let (text, undefined) = self.substitute(written, false);
        if !undefined.is_empty() {
            let names: Vec<String> = undefined.iter().map(|u| format!("${{{u}}}")).collect();
            self.debug(Code::IncludeSkipped, file, include.line, format!("include \"{written}\" skipped: {} undefined", names.join(", ")));
            return;
        }
        if text.trim().is_empty() {
            if from_variable {
                self.debug(Code::IncludeSkipped, file, include.line, format!("include \"{written}\" skipped: empty"));
            } else {
                self.error(Code::NoValue, file, include.line, "<include> has no path");
            }
            return;
        }
        let resolved = resolve_path(&text, file);
        if !Path::new(&resolved).is_file() {
            if from_variable {
                self.debug(Code::IncludeSkipped, file, include.line, format!("include \"{written}\" skipped: {resolved} not found"));
            } else {
                self.error(Code::IncludeMissing, file, include.line, format!("include \"{written}\" not found (resolved to {resolved})"));
            }
            return;
        }
        self.process_file(&resolved, include.line);
    }

    fn view(&mut self, view: &Node, file: &str) {
        let names = self.names(view, file);
        for name in names.iter().filter(|n| !VIEW_NAMES.contains(&n.as_str())) {
            self.error(Code::UnknownView, file, view.line, format!("view \"{name}\" is not system, gamelist or all"));
        }
        let mut targets: Vec<usize> = vec![];
        for name in &names {
            if let Some(i) = VIEW_NAMES.iter().position(|v| v == name) {
                if !targets.contains(&i) {
                    targets.push(i);
                }
            }
        }
        for element in &view.children {
            let kind = element.name.as_str();
            let Some(spec) = catalog().find(kind) else {
                if allowed(Block::Theme).contains(&kind) || kind == "transitions" {
                    self.error(Code::MisplacedTag, file, element.line, format!("<{kind}> is not allowed inside <view>"));
                } else {
                    self.diagnostics.add_about(
                        Severity::Error,
                        Code::UnknownElement,
                        file,
                        element.line,
                        kind,
                        format!("<{kind}> is not an element type"),
                    );
                }
                continue;
            };
            if element.attribute("extra").is_some() {
                self.error(Code::LegacySyntax, file, element.line, format!("<{kind}> carries the legacy extra attribute"));
            }
            for (attribute, _) in element.attributes.iter().filter(|(n, _)| n != "name" && n != "extra") {
                self.warn(Code::UnusedAttribute, file, element.line, format!("attribute {attribute} on <{kind}> is ignored"));
            }
            let element_names = self.names(element, file);
            let properties = self.properties(element, spec, file);
            for &target in &targets {
                for element_name in &element_names {
                    self.views[target].merge(spec, element_name, &mut self.order, &properties);
                }
            }
        }
    }

    fn properties(&mut self, element: &Node, spec: &'static super::catalog::ElementSpec, file: &str) -> Vec<(String, Raw, Option<String>)> {
        let mut properties = vec![];
        for property in &element.children {
            let name = property.name.as_str();
            let Some(property_spec) = spec.find(name) else {
                let subject = format!("{}.{name}", spec.kind);
                self.diagnostics.add_about(
                    Severity::Error,
                    Code::UnknownProperty,
                    file,
                    property.line,
                    &subject,
                    format!("<{name}> is not a property of <{}>", spec.kind),
                );
                continue;
            };
            if !property.children.is_empty() {
                self.error(Code::BadFormat, file, property.line, format!("<{name}> holds elements, not a value"));
                continue;
            }
            let mut key = None;
            if property_spec.shape == Shape::Keyed {
                let attribute = property_spec.key_attribute.as_deref().unwrap_or("");
                let k = property.attribute(attribute).map(str::trim).unwrap_or("");
                if k.is_empty() {
                    self.error(Code::MissingName, file, property.line, format!("<{name}> needs a {attribute} attribute"));
                    continue;
                }
                if !property_spec.values.iter().any(|v| v == k) {
                    self.warn(Code::InvalidValue, file, property.line, format!("<{name} {attribute}=\"{k}\"> names no known {attribute}"));
                    continue;
                }
                key = Some(k.to_owned());
            }
            let written = property.value.as_str();
            if written.trim().is_empty() {
                self.error(
                    Code::NoValue,
                    file,
                    property.line,
                    format!("property \"{name}\" for element \"{}\" has no value defined", spec.kind),
                );
                continue;
            }
            let (text, undefined) = self.substitute(written, true);
            if !undefined.is_empty() {
                let names: Vec<String> = undefined.iter().map(|u| format!("${{{u}}}")).collect();
                self.warn(
                    Code::UndefinedVariable,
                    file,
                    property.line,
                    format!("<{name}> uses undefined {}, read as empty, as ES-DE reads it", names.join(", ")),
                );
            }
            let from_variable = written.contains("${");
            if text.is_empty() && !undefined.is_empty() {
                self.error(
                    Code::NoValue,
                    file,
                    property.line,
                    format!("property \"{name}\" for element \"{}\" has no value defined once its undefined variables are read as empty; ES-DE refuses it too", spec.kind),
                );
                continue;
            }
            if text.trim().is_empty() {
                self.warn(
                    Code::NoValue,
                    file,
                    property.line,
                    format!("<{name}> is empty once its variables are substituted, and is ignored"),
                );
                continue;
            }
            properties.push((name.to_owned(), Raw { text, file: file.to_owned(), line: property.line, from_variable }, key));
        }
        properties
    }

    // One pass: each ${name} becomes the variable's value, itself substituted when it was defined; an undefined one, in a property, nothing (§39).
    fn substitute(&self, text: &str, blank: bool) -> (String, Vec<String>) {
        let mut out = String::with_capacity(text.len());
        let mut missing = vec![];
        let mut rest = text;
        while let Some(start) = rest.find("${") {
            let Some(end) = rest[start + 2..].find('}') else { break };
            out.push_str(&rest[..start]);
            let name = &rest[start + 2..start + 2 + end];
            match self.variables.get(name) {
                Some(value) => out.push_str(value),
                None => {
                    missing.push(name.to_owned());
                    if !blank {
                        out.push_str(&rest[start..start + 3 + end]);
                    }
                }
            }
            rest = &rest[start + 3 + end..];
        }
        out.push_str(rest);
        (out, missing)
    }
}
