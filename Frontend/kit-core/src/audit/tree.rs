use crate::input::ElementId;
use crate::tokens::Colour;

/// A rectangle in a screen's design pixels.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }

    pub fn centre(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn is_empty(&self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Whether `inner` lies within this rectangle side to side, within the slack.
    pub fn holds_across(&self, inner: &Rect, slack: f64) -> bool {
        inner.x >= self.x - slack && inner.right() <= self.right() + slack
    }

    /// Whether `inner` lies within this rectangle top to bottom, within the slack.
    pub fn holds_down(&self, inner: &Rect, slack: f64) -> bool {
        inner.y >= self.y - slack && inner.bottom() <= self.bottom() + slack
    }

    pub fn holds(&self, inner: &Rect, slack: f64) -> bool {
        self.holds_across(inner, slack) && self.holds_down(inner, slack)
    }

    /// The common part of two rectangles; empty where they do not meet.
    pub fn overlap(&self, other: &Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let w = (self.right().min(other.right()) - x).max(0.0);
        let h = (self.bottom().min(other.bottom()) - y).max(0.0);
        Rect { x, y, w, h }
    }
}

/// The axes an area scrolls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Axes {
    pub vertical: bool,
    pub horizontal: bool,
}

/// The edges of a scrolling area that fade what they cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Edges {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

/// What an item is to the rules: how it is drawn and what a player does with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Layout only: nothing drawn, nothing pressed.
    Panel,
    Words,
    Picture,
    /// A button, toggle, tab, dropdown, field, slider, switch or meter.
    Control,
    /// A row of a list or a menu.
    Row,
    /// An area that scrolls its children.
    Scroller,
    Title,
    Footer,
    MenuBar,
    StatusBar,
    /// Opens over the window on purpose; the fit audit skips it.
    Popup,
}

/// The words an item draws, as measured: their capitals' height and the drawn text's size against the item's box.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Text {
    pub cap_height: f64,
    pub width: f64,
    pub height: f64,
    pub ellipsis: bool,
    /// The words are the host's data (a title, a name, a line built from them), which may elide; a fixed word may not.
    pub data: bool,
    /// The item's own box in the scene's units, unturned, where a turn makes its scene box another shape; None where the
    /// scene box is its own. The words are measured along the item's own axes, so they are held to this box.
    pub own: Option<(f64, f64)>,
}

/// What words are drawn in and what lies behind them, for the contrast rule (EmuSen_Kit.md §7.10).
#[derive(Clone, Debug, PartialEq)]
pub struct Legibility {
    /// The words' colour, the item's opacity folded into its alpha.
    pub ink: Colour,
    /// The type's size in the item's own units, design pixels on a big screen.
    pub size: f64,
    pub bold: bool,
    /// What is behind the words, sampled across their box with no words drawn.
    pub behind: Vec<Colour>,
}

/// An item's place in a stack a kit layout declares (EmuSen_Kit.md §7.33): the stack and the member it lies in, as the
/// harness numbers them, the member's place in the stacking order (higher draws over lower), whether the member holds the
/// focus, and whether this item is the member itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stacking {
    pub stack: u64,
    pub member: u64,
    pub layer: f64,
    pub focused: bool,
    pub is_member: bool,
}

/// One item of a screen's tree, as a recording or a harness describes it; the rules read nothing else.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: ElementId,
    pub name: String,
    pub kind: Kind,
    pub rect: Rect,
    pub visible: bool,
    pub hit_testable: bool,
    pub operable: bool,
    pub unseen: bool,
    pub clips: bool,
    pub scroll: Axes,
    pub fades: Edges,
    pub text: Option<Text>,
    pub legibility: Option<Legibility>,
    pub heard: Option<String>,
    pub paints: Vec<Colour>,
    pub scene: bool,
    pub click_call: Option<String>,
    pub accept_call: Option<String>,
    pub wheel_scrolls: bool,
    /// A turned item's four corners in the scene, in order round its edge; None where its scene box is its shape.
    pub corners: Option<[(f64, f64); 4]>,
    pub stacking: Option<Stacking>,
    pub children: Vec<Item>,
}

impl Item {
    /// A visible item of a kind at a place; controls and rows are hit-testable, a scroller clips and scrolls up and down.
    pub fn new(id: u32, kind: Kind, rect: Rect) -> Item {
        let pressed = matches!(kind, Kind::Control | Kind::Row);
        Item {
            id: ElementId(id),
            name: format!("item{id}"),
            kind,
            rect,
            visible: true,
            hit_testable: pressed,
            operable: false,
            unseen: false,
            clips: kind == Kind::Scroller,
            scroll: Axes { vertical: kind == Kind::Scroller, horizontal: false },
            fades: Edges::default(),
            text: None,
            legibility: None,
            heard: None,
            paints: Vec::new(),
            scene: false,
            click_call: None,
            accept_call: None,
            wheel_scrolls: false,
            corners: None,
            stacking: None,
            children: Vec::new(),
        }
    }

    pub fn named(mut self, name: &str) -> Item {
        self.name = name.to_owned();
        self
    }

    pub fn child(mut self, item: Item) -> Item {
        self.children.push(item);
        self
    }

    pub fn hidden(mut self) -> Item {
        self.visible = false;
        self
    }

    /// The pad can act on it; such an item is also hit-testable.
    pub fn operable(mut self) -> Item {
        self.operable = true;
        self.hit_testable = true;
        self
    }

    pub fn hit_testable(mut self) -> Item {
        self.hit_testable = true;
        self
    }

    /// A row kept built out of view, which the pad is not required to reach.
    pub fn unseen(mut self) -> Item {
        self.unseen = true;
        self
    }

    pub fn clipping(mut self) -> Item {
        self.clips = true;
        self
    }

    pub fn scrolling(mut self, axes: Axes) -> Item {
        self.scroll = axes;
        self.clips = true;
        self
    }

    pub fn fading(mut self, edges: Edges) -> Item {
        self.fades = edges;
        self
    }

    /// Turned: its shape is these four corners, not its scene box.
    pub fn turned(mut self, corners: [(f64, f64); 4]) -> Item {
        self.corners = Some(corners);
        self
    }

    pub fn stacked(mut self, stacking: Stacking) -> Item {
        self.stacking = Some(stacking);
        self
    }

    /// The item's shape: its corners when turned, else its scene box's.
    pub fn shape(&self) -> [(f64, f64); 4] {
        let r = self.rect;
        self.corners.unwrap_or([(r.x, r.y), (r.right(), r.y), (r.right(), r.bottom()), (r.x, r.bottom())])
    }

    pub fn words(mut self, text: Text) -> Item {
        self.text = Some(text);
        self
    }

    pub fn legible(mut self, legibility: Legibility) -> Item {
        self.legibility = Some(legibility);
        self
    }

    pub fn heard(mut self, name: &str) -> Item {
        self.heard = Some(name.to_owned());
        self
    }

    pub fn paints(mut self, colours: &[Colour]) -> Item {
        self.paints = colours.to_vec();
        self
    }

    /// A theme's scene: its colours are its author's.
    pub fn scene(mut self) -> Item {
        self.scene = true;
        self
    }

    /// What a click and the pad's A each call on the item.
    pub fn calls(mut self, click: &str, accept: &str) -> Item {
        self.click_call = Some(click.to_owned());
        self.accept_call = Some(accept.to_owned());
        self
    }

    pub fn wheel_scrolls(mut self) -> Item {
        self.wheel_scrolls = true;
        self
    }

    /// Whether the item puts ink on the screen.
    pub fn is_drawn(&self) -> bool {
        self.visible && !matches!(self.kind, Kind::Panel | Kind::Scroller | Kind::Popup)
    }

    /// Whether a player reads or presses it: words, pictures and anything hit-testable.
    pub fn is_read_or_pressed(&self) -> bool {
        self.is_drawn() && (matches!(self.kind, Kind::Words | Kind::Picture) || self.hit_testable)
    }

    /// Marks every item's words as data unless `fixed` knows them as one of the host's fixed words, by what a reader hears.
    pub fn mark_data(&mut self, fixed: &dyn Fn(&str) -> bool) {
        if let Some(text) = self.text.as_mut() {
            text.data = !fixed(self.heard.as_deref().unwrap_or_default());
        }
        for child in &mut self.children {
            child.mark_data(fixed);
        }
    }

    /// Calls `f` on every visible item with its ancestors, nearest last; an invisible item hides its subtree.
    pub fn visit<'a>(&'a self, f: &mut impl FnMut(&'a Item, &[&'a Item])) {
        let mut ancestors = Vec::new();
        self.visit_from(&mut ancestors, f);
    }

    fn visit_from<'a>(&'a self, ancestors: &mut Vec<&'a Item>, f: &mut impl FnMut(&'a Item, &[&'a Item])) {
        if !self.visible {
            return;
        }
        f(self, ancestors);
        ancestors.push(self);
        for child in &self.children {
            child.visit_from(ancestors, f);
        }
        ancestors.pop();
    }

    pub fn find(&self, id: ElementId) -> Option<&Item> {
        let mut found = None;
        self.visit(&mut |item, _| {
            if item.id == id {
                found = Some(item);
            }
        });
        found
    }

    /// Whether `id` is this item or one of its descendants.
    pub fn encloses(&self, id: ElementId) -> bool {
        self.find(id).is_some()
    }

    /// The hit-testable item drawn last under a point: later siblings over earlier ones, children over their parents.
    pub fn topmost_at(&self, x: f64, y: f64) -> Option<&Item> {
        let mut top = None;
        self.visit(&mut |item, _| {
            if item.hit_testable && item.rect.contains_point(x, y) {
                top = Some(item);
            }
        });
        top
    }
}
