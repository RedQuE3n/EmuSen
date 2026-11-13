//! One view that moves: the selection, its glide, the held direction's repeats, the metadata fade and the grid's focus
//! and rows, and each element's state at the time the host steps to (EmuSen_BigPicture.md §14.6, §14.7, §16).

use crate::loader::{Element, View};

use super::data::{Data, Game, System};
use super::grid::GridGeometry;
use super::mapping;
use super::motion::{Easing, Glide, Motion, Repeat, RepeatRule, TextScroll, Time};
use super::selector;
use super::words;

/// A rectangle in screen pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A grid's focus at a time: the item the move left and its level then, the selection's level then, and the move's progress.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GridFocus {
    pub scroll_row: f64,
    pub from: Option<usize>,
    pub from_level: f64,
    pub to_level: f64,
    pub progress: f64,
}

/// One element at a time: where, how opaque, how far its list has moved, and what it says.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementState {
    /// The element's index in the resolved view, which is its drawing order.
    pub index: usize,
    pub kind: String,
    pub name: String,
    /// Why the element draws nothing; None when it is drawn.
    pub skipped: Option<String>,
    /// pos, size and origin in pixels: x and y are the box's top left.
    pub rect: Rect,
    pub rotation: f32,
    /// rotationOrigin, a fraction of the box.
    pub rotation_origin: (f32, f32),
    pub opacity: f64,
    /// A carousel's or list's position in items, not wrapped; a grid's first row shown.
    pub offset: f64,
    /// Time since the selection, which a self-scrolling text reads.
    pub since_selected: Time,
    /// The rule a self-scrolling text follows, its speed and gap in pixels.
    pub scroll: Option<TextScroll>,
    pub grid: Option<GridFocus>,
    /// What a text, datetime or clock says, in its letter case.
    pub text: Option<String>,
    /// The game the element draws, an index into the system's games: the selection in a gamelist, a gameselector's pick in the system view.
    pub game: Option<usize>,
}

/// Whether an element shows a game's media or metadata, which in the system view a gameselector gives it.
fn uses_game(e: &Element) -> bool {
    match e.kind() {
        "image" | "video" => !e.list("imageType").is_empty(),
        "text" | "datetime" => e.string("metadata").is_some(),
        "rating" => true,
        _ => false,
    }
}

/// A move of the selection, for the host's sounds: its signed size, whether it was held, and its time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub delta: i32,
    pub held: bool,
    pub at: Time,
}

pub struct SceneView {
    pub data: Data,
    pub view: String,
    pub motion: Motion,
    now: Time,
    position: Glide,
    selected_at: Time,
    changed: bool,
    metadata: Glide,
    repeat: Repeat,
    vertical: bool,
    scroll: Glide,
    focus: Glide,
    focus_from: Option<usize>,
    from_level: f64,
    to_level: f64,
    steps: Vec<Step>,
    /// Moves of the selection so far, which vary the gameselectors' random picks.
    navigations: u64,
}

impl SceneView {
    pub fn new(data: Data, view: &str, now: Time) -> Self {
        let motion = data.motion;
        let mut v = SceneView {
            data,
            view: view.to_owned(),
            motion,
            now,
            position: Glide::at(0.0),
            selected_at: now,
            changed: false,
            metadata: Glide::at(1.0),
            repeat: Repeat::default(),
            vertical: false,
            scroll: Glide::at(0.0),
            focus: Glide::at(1.0),
            focus_from: None,
            from_level: 1.0,
            to_level: 0.0,
            steps: vec![],
            navigations: 0,
        };
        v.position = Glide::at(v.index() as f64);
        v.scroll = Glide::at(v.grid().map(|g| g.scroll_for(v.index())).unwrap_or(0.0));
        v
    }

    pub fn now(&self) -> Time {
        self.now
    }

    pub fn theme_view(&self) -> &View {
        self.data.system().theme.view(&self.view)
    }

    fn is_system_view(&self) -> bool {
        self.view == "system"
    }

    fn primary_kind(&self) -> Option<String> {
        self.theme_view().primary().map(|p| p.kind().to_owned())
    }

    /// How many items the view lists, and which is selected.
    pub fn count(&self) -> usize {
        if self.is_system_view() {
            self.data.systems.len()
        } else {
            self.data.system().games.len()
        }
    }

    pub fn index(&self) -> usize {
        if self.is_system_view() {
            self.data.system_index
        } else {
            self.data.game_index
        }
    }

    /// The primary element's position now, in items, not wrapped; and where it is going.
    pub fn position(&self) -> f64 {
        self.position.value_at(self.now)
    }

    pub fn target(&self) -> f64 {
        self.position.to
    }

    /// The moves since the host last asked.
    pub fn take_steps(&mut self) -> Vec<Step> {
        std::mem::take(&mut self.steps)
    }

    /// The grid's layout, when the view's primary element is a grid.
    pub fn grid(&self) -> Option<GridGeometry> {
        let p = self.theme_view().primary().filter(|p| p.kind() == "grid")?;
        Some(GridGeometry::of(p, self.data.width, self.data.height, self.count()))
    }

    // Whether the primary element slides between items: a carousel's itemTransitions.
    fn slides(&self) -> bool {
        self.theme_view().primary().is_some_and(|p| p.kind() == "carousel" && p.string("itemTransitions") != Some("instant"))
    }

    fn select(&mut self, target: usize) {
        if self.is_system_view() {
            self.data.system_index = target;
        } else {
            self.data.game_index = target;
        }
    }

    /// One step of the selection, as a tap gives it: a carousel and a list both wrap at their ends.
    pub fn step(&mut self, delta: i32, now: Time) {
        self.move_by(delta, now, false, false);
    }

    /// A move by several items that stops at a list's ends; false when nothing moved.
    pub fn jump(&mut self, delta: i32, now: Time) -> bool {
        self.move_by(delta, now, true, false)
    }

    fn move_by(&mut self, delta: i32, now: Time, held: bool, vertical: bool) -> bool {
        if self.primary_kind().as_deref() == Some("grid") {
            return self.move_grid(delta, now, held, vertical);
        }
        self.now = now;
        let count = self.count() as i64;
        if count == 0 || delta == 0 {
            return false;
        }
        let index = self.index() as i64;
        let wraps = self.primary_kind().as_deref() == Some("carousel") || !held;
        let target = if wraps { (index + delta as i64).rem_euclid(count) } else { (index + delta as i64).clamp(0, count - 1) };
        if target == index {
            return false;
        }
        let to = self.position.to + if wraps { delta as f64 } else { (target - index) as f64 };
        self.select(target as usize);
        self.position = if self.slides() {
            self.position.toward(to, now, self.motion.carousel_step, self.motion.carousel_easing)
        } else {
            Glide::at(to)
        };
        self.selected_at = now;
        self.changed = true;
        self.steps.push(Step { delta, held, at: now });
        self.navigations += 1;
        true
    }

    // An item's focus at a time: the selection rising from its level, the item left falling from its own, the rest none.
    fn focus_of(&self, index: usize, at: Time) -> f64 {
        let p = self.focus.value_at(at);
        if index == self.index() {
            return if self.focus_from == Some(index) { 1.0 } else { self.to_level + (1.0 - self.to_level) * p };
        }
        if Some(index) == self.focus_from {
            self.from_level * (1.0 - p)
        } else {
            0.0
        }
    }

    // ES-DE's grid, measured: across a row's end to the next row; a tap wraps at the ends and a hold stops; up and down stop at the first and last rows (§16).
    fn move_grid(&mut self, delta: i32, now: Time, held: bool, vertical: bool) -> bool {
        self.now = now;
        let count = self.count() as i64;
        let Some(g) = self.grid().filter(|_| count > 0 && delta != 0) else { return false };
        let index = self.index() as i64;
        let target = if vertical {
            let t = index + delta.signum() as i64 * g.columns() as i64;
            if t < 0 {
                return false;
            }
            if t >= count {
                if g.row_of(index as usize) >= g.row_of((count - 1) as usize) {
                    return false;
                }
                count - 1
            } else {
                t
            }
        } else if held {
            (index + delta as i64).clamp(0, count - 1)
        } else {
            (index + delta as i64).rem_euclid(count)
        };
        if target == index {
            return false;
        }
        let (target, index) = (target as usize, index as usize);
        (self.from_level, self.to_level) = (self.focus_of(index, now), self.focus_of(target, now));
        self.focus_from = Some(index);
        let primary = self.theme_view().primary().expect("a grid");
        let instant_items = primary.string("itemTransitions") == Some("instant");
        let instant_rows = primary.string("rowTransitions") == Some("instant");
        self.focus = if instant_items { Glide::at(1.0) } else { Glide::new(0.0, 1.0, now, self.motion.grid_step, self.motion.grid_easing) };
        let scroll = g.scroll_for(target);
        self.scroll =
            if instant_rows { Glide::at(scroll) } else { self.scroll.toward(scroll, now, self.motion.grid_step, self.motion.grid_easing) };
        self.select(target);
        self.selected_at = now;
        self.changed = true;
        self.steps.push(Step { delta, held, at: now });
        self.navigations += 1;
        true
    }

    fn repeat_rule(&self) -> RepeatRule {
        match self.primary_kind().as_deref() {
            Some("grid") => self.motion.grid_repeat,
            Some("carousel") if self.theme_view().primary().and_then(|p| p.bool("fastScrolling")) == Some(true) => {
                self.motion.carousel_fast_repeat
            }
            Some("carousel") => self.motion.carousel_repeat,
            _ => self.motion.list_repeat,
        }
    }

    /// A direction held from a time on; a grid's vertical direction moves by rows.
    pub fn press(&mut self, direction: i32, now: Time, vertical: bool) {
        self.vertical = vertical && self.primary_kind().as_deref() == Some("grid");
        self.move_by(direction, now, false, self.vertical);
        let rule = self.repeat_rule();
        self.repeat.press(direction, now, rule);
    }

    pub fn release(&mut self, now: Time) {
        self.advance(now);
        self.repeat.release();
        self.fade_metadata_in(now);
    }

    fn fade_metadata_in(&mut self, now: Time) {
        if self.metadata.to < 1.0 {
            self.metadata = self.metadata.toward(1.0, now, self.motion.metadata_fade_in, Easing::Linear);
        }
    }

    /// Moves the view's clock on: the repeats that fall due, then the time every state is read at.
    pub fn advance(&mut self, now: Time) {
        while let Some((delta, at)) = self.repeat.next_due(now) {
            if self.is_system_view() {
                self.move_by(delta, at, true, self.vertical);
            } else if self.move_by(delta, at, true, self.vertical) {
                if self.metadata.to > 0.0 {
                    self.metadata = self.metadata.toward(0.0, at, self.motion.metadata_fade_out, Easing::Linear);
                }
            } else {
                self.fade_metadata_in(at);
            }
        }
        self.now = now;
    }

    /// Whether anything still moves, so a host can stop asking for frames; a text that may scroll counts, since its loop never ends.
    pub fn is_moving(&self) -> bool {
        self.is_animating() || self.frame().iter().any(|s| s.skipped.is_none() && s.scroll.is_some_and(|r| r.speed > 0.0))
    }

    /// Whether a glide, a hold, a fade or a scrollFadeIn runs; a self-scrolling text is the frame's to judge, once measured.
    pub fn is_animating(&self) -> bool {
        let now = self.now;
        let view = self.theme_view();
        !self.position.is_settled_at(now)
            || !self.scroll.is_settled_at(now)
            || !self.focus.is_settled_at(now)
            || self.repeat.held()
            || !self.metadata.is_settled_at(now)
            || (self.changed && now < self.selected_at + self.motion.scroll_fade_in && view.elements.iter().any(|e| self.fades_in(e)))
    }

    pub fn selected_at(&self) -> Time {
        self.selected_at
    }

    /// ES-DE's quick-scroll overlay is up: a direction is held and its repeats have faded the metadata out.
    pub fn quick_scrolling(&self) -> bool {
        self.repeat.held() && self.metadata.to <= 0.0
    }

    // A game's metadata and media, which ES-DE fades out while the list scrolls fast.
    fn is_game_metadata(&self, e: &Element) -> bool {
        !self.is_system_view()
            && (e.bool("metadataElement") == Some(true)
                || match e.kind() {
                    "text" => e
                        .string("metadata")
                        .is_some_and(|m| !matches!(m, "systemName" | "systemFullname" | "sourceSystemName" | "sourceSystemFullname")),
                    "datetime" | "rating" | "badges" => true,
                    // A video with `path` is static, metadata only by metadataElement (THEMES.md, video).
                    "video" => !e.explicit.contains_key("path"),
                    "image" => !e.list("imageType").is_empty(),
                    _ => false,
                })
    }

    // An element fading in on a change of game (scrollFadeIn), in the gamelist only.
    fn fades_in(&self, e: &Element) -> bool {
        !self.is_system_view() && e.bool("scrollFadeIn") == Some(true)
    }

    // The reasons an element draws nothing, before what it would show is asked.
    fn skip(&self, e: &Element, game: Option<&Game>) -> Option<String> {
        if e.kind() == "gameselector" {
            return Some("a gameselector picks games and draws nothing".into());
        }
        if !mapping::is_drawn(e.kind()) {
            return Some(format!("{} is not drawn", e.kind()));
        }
        if e.bool("visible") == Some(false) {
            return Some("visible is false".into());
        }
        if e.bool("metadataElement") == Some(true) && self.data.hide_metadata {
            return Some("metadata elements hidden".into());
        }
        if self.view == "gamelist" && game.is_some_and(|g| g.hide_metadata) && hidden_with_metadata(e) {
            return Some("the entry hides its metadata".into());
        }
        // The help, clock and status by their scope against a menu over the view (ES-DE 3.4.1's `Window::render`).
        match e.string("scope") {
            Some("none") => return Some("scope none".into()),
            Some("menu") if !self.data.menu_open => return Some("scope menu, and no menu is open".into()),
            Some("view") if self.data.menu_open => return Some("scope view, and a menu is open".into()),
            _ => {}
        }
        if e.float("opacity") == Some(0.0) {
            return Some("opacity is 0".into());
        }
        if e.kind() == "clock" && !self.data.show_clock {
            return Some("the clock is turned off".into());
        }
        if e.kind() == "helpsystem" && !self.data.show_help {
            return Some("the help is turned off".into());
        }
        None
    }

    /// Every element of the view at the view's time, in drawing order.
    pub fn frame(&self) -> Vec<ElementState> {
        let now = self.now;
        let (w, h) = (self.data.width, self.data.height);
        let system = self.data.system();
        // In the system view a game comes from a gameselector's pick (selector.rs), drawn again on each move.
        let view = self.theme_view();
        let seed = self.data.selection_seed ^ self.navigations;
        let selecting = self.is_system_view() && view.of_kind("gameselector").next().is_some();
        let fade_in = if self.motion.scroll_fade_in > 0 && self.changed {
            Glide::new(self.motion.scroll_fade_in_from, 1.0, self.selected_at, self.motion.scroll_fade_in, Easing::Linear).value_at(now)
        } else {
            1.0
        };
        let metadata = self.metadata.value_at(now);
        let grid = self.grid();
        view.elements
            .iter()
            .enumerate()
            .map(|(index, e)| {
                let pos = e.pair("pos").unwrap_or_default();
                let size = e.pair("size").unwrap_or_default();
                let origin = e.pair("origin").unwrap_or_default();
                let (bw, bh) = (size.x.max(0.0) as f64 * w, size.y.max(0.0) as f64 * h);
                let rect =
                    Rect { x: pos.x as f64 * w - origin.x as f64 * bw, y: pos.y as f64 * h - origin.y as f64 * bh, width: bw, height: bh };
                let rotation_origin = e.pair("rotationOrigin").map(|p| (p.x, p.y)).unwrap_or((0.5, 0.5));
                let game_index = if !self.is_system_view() {
                    self.data.game().map(|_| self.data.game_index.min(system.games.len() - 1))
                } else if selecting {
                    selector::game_for(e, view, system, seed)
                } else {
                    None
                };
                let game = game_index.and_then(|i| system.any_game(i));
                let mut skipped = self.skip(e, game);
                if skipped.is_none() && selecting && game.is_none() && uses_game(e) {
                    skipped = Some("the gameselector's entry is empty".into());
                }
                let text = if skipped.is_none() { self.text_of(e, system, game) } else { None };
                if skipped.is_none() && matches!(e.kind(), "text" | "datetime") && text.is_none() {
                    skipped = Some("nothing to show".into());
                }
                let opacity_of = if e.kind() == "helpsystem" { super::help::now_of(e, "opacity", self.data.menu_open) } else { "opacity".into() };
                let mut opacity = e.float(&opacity_of).unwrap_or(1.0) as f64;
                if self.fades_in(e) {
                    opacity *= fade_in;
                }
                if self.is_game_metadata(e) {
                    opacity *= metadata;
                }
                let (offset, grid_focus) = match e.kind() {
                    "carousel" | "textlist" => (self.position.value_at(now), None),
                    "grid" => {
                        let scroll = self.scroll.value_at(now);
                        let focus = GridFocus {
                            scroll_row: scroll,
                            from: self.focus_from,
                            from_level: self.from_level,
                            to_level: self.to_level,
                            progress: self.focus.value_at(now),
                        };
                        (scroll, grid.map(|_| focus))
                    }
                    _ => (0.0, None),
                };
                ElementState {
                    index,
                    kind: e.kind().to_owned(),
                    name: e.name.clone(),
                    skipped,
                    rect,
                    rotation: e.float("rotation").unwrap_or(0.0),
                    rotation_origin,
                    opacity,
                    offset,
                    since_selected: now - self.selected_at,
                    scroll: self.scroll_of(e, h),
                    grid: grid_focus,
                    text,
                    game: game_index,
                }
            })
            .collect()
    }

    fn text_of(&self, e: &Element, system: &System, game: Option<&Game>) -> Option<String> {
        let said = match e.kind() {
            "text" => words::text(e, &self.view, system, game),
            "datetime" => words::date_time(e, &self.view, game, &self.data.now),
            "clock" => Some(words::clock(e, &self.data.now)),
            _ => None,
        }?;
        Some(words::cased(&said, e.string("letterCase")))
    }

    // A textlist's, carousel's or grid's selected name and a text container scroll by the theme's delays, speed and gap
    // and ES-DE's measured rates (§14.7); an item's name at its font's size, as ES-DE 3.4.1's `TextComponent` sets its
    // speed from the font, which textRelativeScale leaves alone.
    fn scroll_of(&self, e: &Element, h: f64) -> Option<TextScroll> {
        let m = &self.motion;
        let seconds = |s: f32| (s as f64 * 1_000_000.0).round() as Time;
        match e.kind() {
            // On by default in a textlist, off in a carousel or grid (THEMES.md).
            "textlist" | "carousel" | "grid" if e.bool("textHorizontalScrolling").unwrap_or(e.kind() == "textlist") => {
                let font = if e.kind() == "textlist" { e.float("fontSize").unwrap_or(0.045) as f64 } else { super::primary::item_font(e) } * h;
                Some(TextScroll {
                    delay: seconds(e.float("textHorizontalScrollDelay").unwrap_or(3.0)),
                    speed: m.marquee_speed_per_em * font * e.float("textHorizontalScrollSpeed").unwrap_or(1.0) as f64,
                    gap: m.marquee_gap_seconds * e.float("textHorizontalScrollGap").unwrap_or(1.5) as f64 * m.marquee_speed_per_em * font,
                    end_pause: 0,
                    fade_in: 0,
                    whole_pixels: false,
                })
            }
            "text" if e.bool("container") == Some(true) => {
                let font = e.float("fontSize").unwrap_or(0.045) as f64 * h;
                let horizontal = e.string("containerType") == Some("horizontal");
                let speed = e.float("containerScrollSpeed").unwrap_or(1.0) as f64;
                let delay = seconds(e.float("containerStartDelay").unwrap_or(if horizontal { 1.5 } else { 4.5 }));
                let unit = m.horizontal_container_speed_per_em * font;
                Some(if horizontal {
                    TextScroll {
                        delay,
                        speed: unit * speed,
                        gap: m.horizontal_container_gap_seconds * e.float("containerScrollGap").unwrap_or(1.5) as f64 * unit,
                        end_pause: 0,
                        fade_in: 0,
                        whole_pixels: false,
                    }
                } else {
                    TextScroll {
                        delay,
                        speed: m.vertical_container_speed_per_em * font * speed,
                        gap: 0.0,
                        end_pause: seconds(e.float("containerResetDelay").unwrap_or(7.0)),
                        fade_in: m.vertical_container_fade_in,
                        whole_pixels: true,
                    }
                })
            }
            _ => None,
        }
    }
}

// THEMES.md's metadataElement: text metadata fields, ratings and badges, and elements the theme marks; the description stays (§22).
fn hidden_with_metadata(e: &Element) -> bool {
    e.bool("metadataElement") == Some(true)
        || match e.kind() {
            "rating" | "badges" | "datetime" => true,
            "text" => e.string("metadata").is_some_and(|m| {
                !matches!(m, "description" | "name" | "systemName" | "systemFullname" | "sourceSystemName" | "sourceSystemFullname")
            }),
            _ => false,
        }
}
