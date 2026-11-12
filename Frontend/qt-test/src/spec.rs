//! The `spec 1` writer: a recorded state from the Qt item tree in the C# recordings' format, so a Qt screen is compared with the screen it replaces (EmuSen_InterfaceSpec.md §2, EmuSen_Kit.md §7.11).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use emusen_kit_core::audit::{Item, Kind, Rect, accepted, small_text_exception};
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;

use crate::harness::{Landing, Measures, Step};
use crate::tree::{Node, Tree};

/// The first line, a comment every reader skips.
pub const BANNER: &str =
    "# EmuSen interface specification, recorded from the Qt front end by emusen-qt-test - see EmuSen_InterfaceSpec.md.";

/// What a recording says of its state before the tree (InterfaceSpec §2.1).
pub struct Header<'a> {
    pub state: &'a str,
    pub case: &'a str,
    /// `sheet`, `menu`, `overlay` or `scene`.
    pub kind: &'a str,
    pub width: i32,
    pub height: i32,
    pub measures: Measures,
    pub palette: &'a [Colour],
}

/// A number to two places, as the format writes every number.
pub fn num(v: f64) -> String {
    let text = format!("{v:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

/// A value bare when it is plain printable ASCII with no quote, backslash or equals sign, else quoted with C escapes.
pub fn quote(v: &str) -> String {
    if !v.is_empty() && v.chars().all(|c| c > ' ' && c != '"' && c != '\\' && c != '=' && (c as u32) < 0x7F) {
        return v.to_owned();
    }
    let mut out = String::from("\"");
    for c in v.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn rect(r: &Rect) -> String {
    format!("{},{},{},{}", num(r.x), num(r.y), num(r.w), num(r.h))
}

fn rgba(c: Colour) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", c.r, c.g, c.b, c.a)
}

/// The words the format uses for a role, Avalonia's control types, from the role Qt's accessibility declares.
pub fn role(qt: i32) -> &'static str {
    match qt {
        0x02 => "menubar",
        0x03 => "scrollbar",
        0x09 | 0x12 => "window",
        0x0B => "menu",
        0x0C => "menuitem",
        0x0D => "tooltip",
        0x10 => "pane",
        0x14 => "group",
        0x15 => "separator",
        0x16 => "toolbar",
        0x17 => "statusbar",
        0x18 => "datagrid",
        0x1E => "hyperlink",
        0x21 => "list",
        0x22 => "listitem",
        0x23 => "tree",
        0x24 => "treeitem",
        0x25 => "tabitem",
        0x28 => "image",
        0x29 => "text",
        0x2A => "edit",
        0x2B => "button",
        0x2C => "checkbox",
        0x2D => "radiobutton",
        0x2E => "combobox",
        0x30 => "progressbar",
        0x33 => "slider",
        0x34 => "spinner",
        0x3C => "tab",
        _ => "none",
    }
}

/// The QML type a line names: a component's own name, or a Qt item's without the class prefix.
fn kind_of(node: &Node) -> &str {
    node.ty.strip_prefix("QQuick").unwrap_or(&node.ty)
}

fn direction(button: UiButton) -> &'static str {
    match button {
        UiButton::Up => "up",
        UiButton::Down => "down",
        UiButton::Left => "left",
        _ => "right",
    }
}

/// One written line: the item, its depth under the root, and its facts by key in the format's order.
struct Line<'a> {
    item: &'a Item,
    depth: usize,
    facts: BTreeMap<&'static str, String>,
}

struct Writer<'a> {
    tree: &'a Tree,
    frame: Rect,
    unit: f64,
    lines: Vec<Line<'a>>,
    ids: BTreeMap<u32, usize>,
}

impl<'a> Writer<'a> {
    /// Every shown item under the root in depth-first order; a backdrop is one line, as the rules read it.
    fn visit(&mut self, item: &'a Item, depth: usize, ancestors: &mut Vec<&'a Item>) {
        if !item.visible {
            return;
        }
        let at = self.lines.len();
        self.ids.insert(item.id.0, at);
        let facts = self.facts(item, ancestors);
        self.lines.push(Line { item, depth, facts });
        if self.tree.nodes[item.id.0 as usize].backdrop {
            return;
        }
        ancestors.push(item);
        for child in &item.children {
            self.visit(child, depth + 1, ancestors);
        }
        ancestors.pop();
    }

    fn id(&self, item: &Item) -> String {
        self.ids.get(&item.id.0).map(ToString::to_string).unwrap_or_default()
    }

    /// The facts of one item, the ones §2.4's rules read decided as kit-core's audit decides them (EmuSen_Kit.md §7.11).
    fn facts(&self, item: &Item, ancestors: &[&'a Item]) -> BTreeMap<&'static str, String> {
        let node = &self.tree.nodes[item.id.0 as usize];
        let mut f = BTreeMap::new();
        let flag = |f: &mut BTreeMap<&'static str, String>, key: &'static str| {
            f.insert(key, String::new());
        };
        f.insert("role", role(node.role).to_owned());
        f.insert("at", rect(&item.rect));
        let seen = ancestors.iter().filter(|a| a.clips).fold(item.rect, |r, a| r.overlap(&a.rect));
        if rect(&seen) != rect(&item.rect) {
            f.insert("seen", rect(&seen));
        }
        if !node.name.is_empty() {
            f.insert("name", node.name.clone());
        }
        if let Some(heard) = item.heard.as_ref().filter(|h| !h.is_empty()) {
            f.insert("heard", heard.clone());
        }
        if let Some(text) = node.text.as_ref().filter(|t| !t.is_empty()) {
            f.insert("text", text.clone());
        }
        if item.unseen {
            flag(&mut f, "unseen");
        } else if item.operable {
            flag(&mut f, "op");
        }
        if !node.enabled {
            flag(&mut f, "off");
        }
        if item.is_drawn() {
            flag(&mut f, "drawn");
        }
        if !item.hit_testable {
            flag(&mut f, "nohit");
        }
        if let Some(class) = small_text_exception(item, ancestors) {
            f.insert("allow", accepted(class).map_or("no ruling", |a| a.ruling).to_owned());
        }
        let in_popup = ancestors.iter().skip(1).any(|a| a.kind == Kind::Popup);
        if item.is_drawn() && !in_popup {
            match ancestors.iter().rev().find(|a| a.clips || a.scroll.vertical || a.scroll.horizontal) {
                Some(a) => {
                    f.insert("limit", rect(&a.rect));
                    f.insert("by", if a.scroll.vertical { "scroll" } else { "clip" }.to_owned());
                }
                None => {
                    f.insert("limit", rect(&self.frame));
                    f.insert("by", "panel".to_owned());
                }
            }
        }
        if item.kind == Kind::Words {
            if let Some(owner) = ancestors.last().filter(|o| matches!(o.kind, Kind::Control | Kind::Row)) {
                f.insert("owner", self.id(owner));
            }
        }
        let scrollers: Vec<String> = ancestors.iter().filter(|a| a.kind == Kind::Scroller).map(|a| self.id(a)).collect();
        if !scrollers.is_empty() {
            f.insert("in", scrollers.join(","));
        }
        if item.kind == Kind::Scroller {
            f.insert("overflow", num((node.ch - item.rect.h).max(0.0)));
            f.insert("viewport", rect(&item.rect));
            let fades: String = [(item.fades.top, 't'), (item.fades.bottom, 'b'), (item.fades.left, 'l'), (item.fades.right, 'r')]
                .iter()
                .filter(|(on, _)| *on)
                .map(|(_, c)| *c)
                .collect();
            if !fades.is_empty() {
                f.insert("fades", fades);
            }
            if !item.scroll.vertical {
                flag(&mut f, "vfixed");
            }
        }
        if let (Some(text), true) = (item.text, node.text.as_ref().is_some_and(|t| !t.is_empty())) {
            if text.ellipsis {
                flag(&mut f, "ellipsis");
            }
            f.insert("layout", format!("{},{}", num(text.width), num(text.height)));
            f.insert("box", format!("{},{}", num(item.rect.w), num(item.rect.h)));
            let unit = self.unit;
            f.insert("size", num(node.size * node.scale / unit));
            f.insert("caps", format!("{}", (text.cap_height / unit * 10_000.0).round() / 10_000.0));
            if let Some(face) = node.face.as_ref().filter(|face| !face.is_empty()) {
                f.insert("face", face.clone());
            }
        }
        let scene = item.scene || ancestors.iter().any(|a| a.scene);
        if !scene && !item.paints.is_empty() {
            let mut paints: Vec<String> = item.paints.iter().map(|c| rgba(*c)).collect();
            paints.sort();
            paints.dedup();
            f.insert("paints", paints.join(","));
        }
        f
    }
}

/// An item's path of child indices from the window's root, how the format names a control it did not record (InterfaceSpec §2.3).
fn path(tree: &Tree, index: u32) -> String {
    let mut steps = Vec::new();
    let mut at = index as usize;
    while let Some(parent) = tree.nodes.iter().position(|n| n.id == tree.nodes[at].parent) {
        let siblings = tree.nodes.iter().filter(|n| n.parent == tree.nodes[parent].id);
        steps.push(siblings.take_while(|n| n.id != tree.nodes[at].id).count());
        at = parent;
    }
    steps.iter().rev().map(ToString::to_string).collect::<Vec<_>>().join("/")
}

/// The state under `root` as a recording: the header, the tree, the focus and every press of the walk; a press that
/// landed on an item not recorded (a row built out of view) names it by its path.
pub fn write(tree: &Tree, root: &Item, header: &Header<'_>, focus: Option<u32>, walk: &[Step]) -> String {
    let frame = Rect::new(0.0, 0.0, f64::from(header.width), f64::from(header.height));
    let mut writer = Writer { tree, frame, unit: header.measures.unit, lines: Vec::new(), ids: BTreeMap::new() };
    writer.visit(root, 0, &mut Vec::new());
    let root_node = &tree.nodes[root.id.0 as usize];
    let root_name =
        if root_node.name.is_empty() { kind_of(root_node).to_owned() } else { format!("{}#{}", kind_of(root_node), root_node.name) };
    let mut palette: Vec<String> = header.palette.iter().map(|c| rgba(*c)).collect();
    palette.sort();
    palette.dedup();

    let mut out = String::new();
    let _ = writeln!(out, "{BANNER}");
    let _ = writeln!(out, "spec 1");
    let _ = writeln!(out, "state {}", header.state);
    let _ = writeln!(out, "size {} {}", header.width, header.height);
    let _ = writeln!(out, "kind {}", header.kind);
    let _ = writeln!(out, "root {root_name}");
    let _ = writeln!(out, "unit {}", num(header.measures.unit));
    let _ = writeln!(out, "slack {}", num(header.measures.slack));
    let _ = writeln!(out, "floor {}", num(header.measures.floor));
    let _ = writeln!(out, "frame {}", rect(&frame));
    let _ = writeln!(out, "case {}", header.case);
    let _ = writeln!(out, "palette {}", palette.join(" "));
    let _ = writeln!(out, "tree");
    for (id, line) in writer.lines.iter().enumerate() {
        let _ = write!(out, "{}{id} {}", "  ".repeat(line.depth + 1), kind_of(&tree.nodes[line.item.id.0 as usize]));
        for (key, value) in &line.facts {
            if value.is_empty() {
                let _ = write!(out, " {key}");
            } else {
                let _ = write!(out, " {key}={}", quote(value));
            }
        }
        out.push('\n');
    }
    let written = |index: u32| writer.ids.get(&index).map(ToString::to_string);
    let reference = |index: u32| written(index).unwrap_or_else(|| format!("path:{}", path(tree, index)));
    let _ = writeln!(out, "focus {}", focus.and_then(written).unwrap_or_else(|| "none".to_owned()));
    for step in walk {
        let from = reference(step.from);
        let to = match step.to {
            Landing::At(to) => reference(to),
            Landing::Same => "same".to_owned(),
            Landing::Lost => "none".to_owned(),
        };
        let _ = writeln!(out, "edge {} {} {}", quote(&from), direction(step.button), quote(&to));
    }
    out
}
