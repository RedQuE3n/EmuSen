//! The window's item tree, as the C++ dumps it, read into kit-core's audit `Item`s, so the rules are kit-core's and nothing is re-implemented (EmuSen_Qt.md §8.1, EmuSen_Kit.md §7.9).

use std::collections::HashMap;

use emusen_kit_core::audit::{Axes, Edges, Item, Kind, Legibility, Rect, Stacking, Text};
use emusen_kit_core::tokens::Colour;
use serde::Deserialize;

/// One item as `harness_tree` writes it.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Node {
    pub id: String,
    pub parent: String,
    pub ty: String,
    pub cls: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub visible: bool,
    pub opacity: f64,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// Scene pixels to one of the item's own units.
    #[serde(default = "one")]
    pub scale: f64,
    pub clip: bool,
    pub hit: bool,
    pub operable: bool,
    pub control: bool,
    pub focus: bool,
    pub popup: bool,
    pub menubar: bool,
    pub picture: bool,
    #[serde(default)]
    pub backdrop: bool,
    /// The words' colour with the item's opacity in its alpha, empty for a disabled control's.
    #[serde(default)]
    pub ink: Option<String>,
    #[serde(default)]
    pub size: f64,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub face: Option<String>,
    /// Where the words are drawn: x, y, width, height in the scene.
    #[serde(default)]
    pub glyphs: Option<[f64; 4]>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub elided: bool,
    /// A turned item's four corners in the scene, x and y in turn.
    #[serde(default)]
    pub corners: Option<[f64; 8]>,
    /// The words' drawn glyphs' four corners in the scene, x and y in turn.
    #[serde(default)]
    pub inked: Option<[f64; 8]>,
    /// The stack a kit layout declares that the item lies in, its member and the member's layer.
    #[serde(default)]
    pub stack: Option<String>,
    #[serde(default)]
    pub member: Option<String>,
    #[serde(default)]
    pub layer: f64,
    #[serde(default, rename = "memberFocus")]
    pub member_focus: bool,
    #[serde(default, rename = "isMember")]
    pub is_member: bool,
    /// A turned item's own width and height in the scene's units, unturned.
    #[serde(default)]
    pub own: Option<[f64; 2]>,
    #[serde(default)]
    pub tw: f64,
    #[serde(default)]
    pub th: f64,
    #[serde(default)]
    pub cap: f64,
    #[serde(default)]
    pub sv: bool,
    #[serde(default)]
    pub sh: bool,
    #[serde(default)]
    pub wheel: bool,
    /// A flickable's content height in the scene.
    #[serde(default)]
    pub ch: f64,
    #[serde(default, rename = "fadeTop")]
    pub fade_top: bool,
    #[serde(default, rename = "fadeBottom")]
    pub fade_bottom: bool,
    #[serde(default, rename = "fadeLeft")]
    pub fade_left: bool,
    #[serde(default, rename = "fadeRight")]
    pub fade_right: bool,
    #[serde(default)]
    pub heard: String,
    #[serde(default)]
    pub role: i32,
    #[serde(default)]
    pub colors: Vec<String>,
    /// What a press may move: Qt Quick's own selection, value and scroll properties, by name (EmuSen_Kit.md §7.11).
    #[serde(default)]
    pub kept: std::collections::BTreeMap<String, f64>,
    /// A focus scope's remembered item, by its id, empty for none: what a press may move too.
    #[serde(default)]
    pub scoped: String,
}

fn yes() -> bool {
    true
}

fn one() -> f64 {
    1.0
}

/// The kit's drawn world: its colours are hues by design, so palette reach treats it as a scene.
pub const SCENES: [&str; 4] = ["Body", "Sky", "Moonrise", "Scene"];

/// The kit's rows and Qt's.
const ROWS: [&str; 4] = ["MenuRow", "WholeRow", "QQuickItemDelegate", "QQuickMenuItem"];

/// A colour as the dump writes it, `#AARRGGBB`.
pub fn colour(text: &str) -> Option<Colour> {
    let hex = text.strip_prefix('#')?;
    if hex.len() != 8 {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some(Colour::rgba(byte(2)?, byte(4)?, byte(6)?, byte(0)?))
}

fn kind(node: &Node) -> Kind {
    if node.popup {
        Kind::Popup
    } else if node.menubar {
        Kind::MenuBar
    } else if node.name == "statusBar" {
        Kind::StatusBar
    } else if node.sv || node.sh {
        Kind::Scroller
    } else if node.text.is_some() {
        Kind::Words
    } else if node.picture {
        Kind::Picture
    } else if ROWS.contains(&node.ty.as_str()) || ROWS.contains(&node.cls.as_str()) {
        Kind::Row
    } else if node.operable || node.control {
        Kind::Control
    } else if node.name.to_ascii_lowercase().contains("title") {
        Kind::Title
    } else if node.name.to_ascii_lowercase().contains("footer") {
        Kind::Footer
    } else {
        Kind::Panel
    }
}

fn item(index: u32, node: &Node) -> Item {
    let rect = Rect::new(node.x, node.y, node.w, node.h);
    let kind = kind(node);
    let label = if node.name.is_empty() { node.ty.clone() } else { node.name.clone() };
    let mut item = Item::new(index, kind, rect).named(&label);
    if !node.visible {
        item = item.hidden();
    }
    if node.hit {
        item = item.hit_testable();
    }
    if node.operable {
        item = item.operable().calls("activate", "activate");
    }
    if node.clip {
        item = item.clipping();
    }
    if node.sv || node.sh {
        item = item.scrolling(Axes { vertical: node.sv, horizontal: node.sh });
    }
    if node.wheel {
        item = item.wheel_scrolls();
    }
    if node.fade_top || node.fade_bottom || node.fade_left || node.fade_right {
        item = item.fading(Edges { top: node.fade_top, bottom: node.fade_bottom, left: node.fade_left, right: node.fade_right });
    }
    if let Some(c) = node.inked {
        item = item.inked([(c[0], c[1]), (c[2], c[3]), (c[4], c[5]), (c[6], c[7])]);
    }
    if let Some(c) = node.corners {
        item = item.turned([(c[0], c[1]), (c[2], c[3]), (c[4], c[5]), (c[6], c[7])]);
    }
    if let (Some(stack), Some(member)) = (&node.stack, &node.member) {
        let number = |s: &str| s.parse::<u64>().unwrap_or(0);
        item = item.stacked(Stacking {
            stack: number(stack),
            member: number(member),
            layer: node.layer,
            focused: node.member_focus,
            is_member: node.is_member,
        });
    }
    if node.text.is_some() {
        item = item.words(Text { cap_height: node.cap, width: node.tw, height: node.th, ellipsis: node.elided, data: false, own: node.own.map(|o| (o[0], o[1])) });
    }
    let heard = if node.heard.is_empty() { node.text.clone().unwrap_or_default() } else { node.heard.clone() };
    if !heard.is_empty() {
        item = item.heard(&heard);
    }
    let paints: Vec<Colour> = node.colors.iter().filter_map(|c| colour(c)).collect();
    if !paints.is_empty() {
        item = item.paints(&paints);
    }
    if SCENES.contains(&node.ty.as_str()) {
        item = item.scene();
    }
    item
}

/// The tree with its ids: the root `Item` and, by the items' numbers, the Qt pointers the harness focuses by.
pub struct Tree {
    pub root: Item,
    pub ids: Vec<u64>,
    pub nodes: Vec<Node>,
}

/// Points spread over a box, a few pixels apart, for sampling what is behind words.
fn spread(glyphs: [f64; 4], width: f64, height: f64) -> Vec<(f64, f64)> {
    let [x, y, w, h] = glyphs;
    let (columns, rows) = ((w / 4.0).clamp(2.0, 16.0) as usize, (h / 4.0).clamp(2.0, 6.0) as usize);
    let mut points = Vec::with_capacity(columns * rows);
    for row in 0..rows {
        for column in 0..columns {
            let px = x + w * (column as f64 + 0.5) / columns as f64;
            let py = y + h * (row as f64 + 0.5) / rows as f64;
            if px >= 0.0 && py >= 0.0 && px < width && py < height {
                points.push((px, py));
            }
        }
    }
    points
}

/// The item of this number under an item, to be changed.
fn find_mut(item: &mut Item, id: u32) -> Option<&mut Item> {
    if item.id.0 == id {
        return Some(item);
    }
    item.children.iter_mut().find_map(|child| find_mut(child, id))
}

impl Tree {
    /// Where to sample behind each visible text that has an ink: its number and its points.
    pub fn behind_points(&self, width: f64, height: f64) -> Vec<(u32, Vec<(f64, f64)>)> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.visible && n.ink.as_deref().is_some_and(|ink| !ink.is_empty()))
            .filter_map(|(i, n)| n.glyphs.map(|g| (i as u32, spread(g, width, height))))
            .filter(|(_, points)| !points.is_empty())
            .collect()
    }

    /// Gives each sampled text its legibility: its ink, its type and what was found behind it.
    pub fn set_behind(&mut self, index: u32, behind: Vec<Colour>) {
        let node = &self.nodes[index as usize];
        let Some(ink) = node.ink.as_deref().and_then(colour) else { return };
        let legibility = Legibility { ink, size: node.size, bold: node.bold, behind };
        if let Some(item) = find_mut(&mut self.root, index) {
            item.legibility = Some(legibility);
        }
    }

    pub fn from_json(json: &str) -> Result<Tree, String> {
        let nodes: Vec<Node> = serde_json::from_str(json).map_err(|e| format!("the tree's JSON: {e}"))?;
        let mut children: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            children.entry(node.parent.as_str()).or_default().push(i);
        }
        let ids: Vec<u64> = nodes.iter().map(|n| n.id.parse().unwrap_or(0)).collect();
        let root_index = nodes.iter().position(|n| n.parent == "0").ok_or("no root")?;
        let root = build(&nodes, &children, root_index, None);
        Ok(Tree { root, ids, nodes })
    }

    /// The item number of a Qt pointer, as `harness_focus` reports one.
    pub fn index_of(&self, id: u64) -> Option<u32> {
        (id != 0).then(|| self.ids.iter().position(|i| *i == id)).flatten().map(|i| i as u32)
    }

    pub fn id_of(&self, index: u32) -> u64 {
        self.ids.get(index as usize).copied().unwrap_or(0)
    }

    pub fn find_named(&self, name: &str) -> Option<u32> {
        self.nodes.iter().position(|n| n.name == name).map(|i| i as u32)
    }

    pub fn find_type(&self, ty: &str) -> Vec<u32> {
        self.nodes.iter().enumerate().filter(|(_, n)| n.ty == ty).map(|(i, _)| i as u32).collect()
    }

    /// One line per item an audit may name: its number, type, objectName, words and box.
    pub fn legend(&self) -> String {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.visible && (n.operable || n.text.is_some() || n.sv || n.sh || !n.name.is_empty()))
            .map(|(i, n)| {
                format!("{i}: {} {} {:?} at {:.0},{:.0} {:.0}x{:.0}", n.ty, n.name, n.text.clone().unwrap_or_default(), n.x, n.y, n.w, n.h)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The nearest operable item at or above this one: where a focus that landed inside an element (a carousel's current card) stands in the walk.
    pub fn walkable_of(&self, index: u32) -> u32 {
        let mut at = index as usize;
        while !self.nodes[at].operable {
            match self.nodes.iter().position(|n| n.id == self.nodes[at].parent) {
                Some(parent) => at = parent,
                None => return index,
            }
        }
        at as u32
    }

    /// The numbers of every operable, visible item.
    pub fn operable(&self) -> Vec<u32> {
        self.nodes.iter().enumerate().filter(|(_, n)| n.operable && n.visible).map(|(i, _)| i as u32).collect()
    }
}

/// The first words under an item: a control named by what it holds, as Avalonia's automation peers resolved it (InterfaceSpec §4.3).
fn words_under(nodes: &[Node], children: &HashMap<&str, Vec<usize>>, index: usize) -> Option<String> {
    for &child in children.get(nodes[index].id.as_str()).map(Vec::as_slice).unwrap_or(&[]) {
        let node = &nodes[child];
        if !node.heard.is_empty() {
            return Some(node.heard.clone());
        }
        if let Some(text) = node.text.as_ref().filter(|t| !t.is_empty()) {
            return Some(text.clone());
        }
        if let Some(found) = words_under(nodes, children, child) {
            return Some(found);
        }
    }
    None
}

/// The nearest clipping ancestor's box, and whether it scrolls.
#[derive(Clone, Copy)]
struct Clip {
    rect: Rect,
    scrolls: bool,
}

/// Builds an item and its children: wholly outside a clip that scrolls it is not shown, outside one that does not an operable item is `unseen` (EmuSen_Kit.md §7.10).
fn build(nodes: &[Node], children: &HashMap<&str, Vec<usize>>, index: usize, clip: Option<Clip>) -> Item {
    let node = &nodes[index];
    if node.backdrop {
        return backdrop_item(nodes, children, index);
    }
    let mut it = item(index as u32, node);
    if node.operable && it.heard.is_none() {
        if let Some(words) = words_under(nodes, children, index) {
            it = it.heard(&words);
        }
    }
    if let Some(clip) = clip {
        if clip.rect.overlap(&it.rect).is_empty() {
            if clip.scrolls {
                it = it.hidden();
            } else if node.operable {
                it = it.unseen();
            }
        }
    }
    let own_clip = if node.clip { Some(Clip { rect: it.rect, scrolls: node.sv || node.sh }) } else { clip };
    if let Some(kids) = children.get(node.id.as_str()) {
        for &child in kids {
            it = it.child(build(nodes, children, child, own_clip));
        }
    }
    it
}

/// A backdrop (an item declaring `backdrop`, as the sky's paint does) and everything under it: drawn behind the screen, so a panel to the rules.
fn backdrop_item(nodes: &[Node], children: &HashMap<&str, Vec<usize>>, index: usize) -> Item {
    let node = &nodes[index];
    let mut it = Item::new(index as u32, Kind::Panel, Rect::new(node.x, node.y, node.w, node.h)).scene();
    if !node.visible {
        it = it.hidden();
    }
    for &child in children.get(node.id.as_str()).map(Vec::as_slice).unwrap_or(&[]) {
        it = it.child(backdrop_item(nodes, children, child));
    }
    it
}
