//! One view's elements as the painter's records: each element's paint, its box once the painter's measurement is
//! applied, its scroll at the view's time, and the primary element's layout (EmuSen_Qt.md §4.5.5).

use std::collections::HashMap;

use crate::loader::Element;

use super::frame::*;
use super::motion::Time;
use super::paint::{self, Ctx, Sizing};
use super::primary;
use super::view::SceneView;

/// What the painter reported for an element: the content it measured (a key) and its extent.
pub type Measures = HashMap<u64, (u64, f64, f64)>;

/// FNV-1a over the parts, each ended by a zero byte.
pub fn key(parts: &[&str]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for b in part.bytes().chain([0]) {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

/// An element's id: its view, kind and name.
pub fn id_of(view: &str, e: &Element) -> u64 {
    key(&[view, e.kind(), &e.name])
}

/// One drawn element with the content key a measurement must match and how long until its scroll next moves.
pub struct Drawn {
    pub placed: Placed,
    pub content: u64,
    pub wait: Option<Time>,
}

fn is_primary(kind: &str) -> bool {
    matches!(kind, "carousel" | "textlist" | "grid")
}

impl SceneView {
    /// The rows the view's primary element lists.
    pub fn rows_of(&self) -> Vec<Row> {
        self.theme_view().primary().map(|e| primary::rows(e, &self.view, &self.data)).unwrap_or_default()
    }

    pub fn rows_id(&self) -> Rows {
        Rows { view_is_system: self.view == "system", system: self.data.system_index }
    }

    /// The view's drawn elements in drawing order and its primary element's layout, at the view's time.
    pub fn draw(&self, measures: &Measures, leaving: bool) -> (Vec<Drawn>, Option<Primary>) {
        let data = &self.data;
        let ctx = Ctx { view: &self.view, data, system: data.system(), game: None, w: data.width, h: data.height };
        let since = self.now() - self.selected_at();
        let elements = &self.theme_view().elements;
        let mut out = vec![];
        let mut primary_out = None;
        for s in self.frame().into_iter().filter(|s| s.skipped.is_none()) {
            let e = &elements[s.index];
            let ctx = Ctx { game: s.game.and_then(|i| ctx.system.any_game(i)), ..ctx };
            let id = id_of(&self.view, e);
            let mut placed = Placed {
                id,
                kind: s.kind.clone(),
                name: s.name.clone(),
                x: s.rect.x,
                y: s.rect.y,
                w: s.rect.width,
                h: s.rect.height,
                rotation: s.rotation as f64,
                rotation_origin: (s.rotation_origin.0 as f64, s.rotation_origin.1 as f64),
                opacity: s.opacity,
                paint: Paint::Primary { leaving },
            };
            if is_primary(&s.kind) {
                let (rect_w, rect_h) = (s.rect.width, s.rect.height);
                let (selected, count) = (self.index(), self.count());
                let (layout, content, wait) = match s.kind.as_str() {
                    "carousel" => (Layout::Carousel(primary::carousel(e, data, count, s.offset, rect_w, rect_h)), 0, None),
                    "grid" => match (self.grid(), s.grid) {
                        (Some(g), Some(focus)) => (Layout::Grid(primary::grid(e, &g, focus, data)), 0, None),
                        _ => continue,
                    },
                    _ => {
                        let mut list = primary::textlist(e, data, count, selected, rect_w, rect_h);
                        let row = primary::row(e, &self.view, data, selected);
                        let content = row.as_ref().map(|r| key(&[&r.text, &list.px.to_string()])).unwrap_or(0);
                        let mut wait = None;
                        if let (Some(rule), Some(row), Some(&(k, mw, _))) = (s.scroll, row, measures.get(&id)) {
                            let marker = if row.marker == Marker::None { 0.0 } else { list.marker_w };
                            let room = (rect_w - 2.0 * list.margin - marker).max(0.0);
                            if k == content && mw > room + 0.01 {
                                list.marquee = Some(Scroll {
                                    x: rule.loop_offset(since, mw),
                                    repeat_at: Some(mw + rule.gap.max(0.0)),
                                    y: 0.0,
                                    opacity: 1.0,
                                });
                                wait = rule.loop_wait(since, mw);
                            }
                        }
                        (Layout::TextList(list), content, wait)
                    }
                };
                primary_out = Some(Primary { id, rows: self.rows_id(), selected, layout });
                out.push(Drawn { placed, content, wait });
                continue;
            }
            let Some(draft) = paint::draft(e, s.text.clone(), s.scroll, &ctx) else { continue };
            // A picture's own size is its file's, so it holds across a resize; text and rows are measured at their size.
            let content = match (&draft.paint, draft.sizing) {
                (Paint::Picture(p), Sizing::Natural { .. } | Sizing::Aspect { .. }) => key(&["picture", &p.source]),
                (Paint::Video(Video { still: Some(p), .. }), Sizing::Natural { .. } | Sizing::Aspect { .. }) => key(&["picture", &p.source]),
                _ => key(&[&format!("{:?}{:?}", draft.paint, draft.size)]),
            };
            let measured = measures.get(&id).filter(|m| m.0 == content).map(|m| (m.1, m.2));
            let (mut bw, mut bh) = draft.size;
            match (draft.sizing, measured) {
                (Sizing::Natural { max: Some((xw, xh)) }, Some((mw, mh))) if mw > 0.0 && mh > 0.0 => {
                    let k = (xw / mw).min(xh / mh);
                    (bw, bh) = (mw * k, mh * k);
                }
                (Sizing::Natural { max: None }, Some((mw, mh))) => (bw, bh) = (mw, mh),
                (Sizing::Aspect { w, h }, Some((mw, mh))) if mw > 0.0 && mh > 0.0 => {
                    (bw, bh) = if w > 0.0 { (w, w * mh / mw) } else { (h * mw / mh, h) };
                }
                (Sizing::Content { auto_w, auto_h }, Some((mw, mh))) => {
                    if auto_w {
                        bw = mw;
                    }
                    if auto_h {
                        bh = mh;
                    }
                }
                _ => {}
            }
            let pos = e.pair("pos").unwrap_or_default();
            let origin = e.pair("origin").unwrap_or_default();
            placed.x = pos.x as f64 * data.width - origin.x as f64 * bw;
            placed.y = pos.y as f64 * data.height - origin.y as f64 * bh;
            (placed.w, placed.h) = (bw, bh);
            // The help is measured as ES-DE's grid, its trailing spacing in; its box is the strip seen (EmuSen_Qt.md §4.12.4).
            if let Paint::Help(h) = &draft.paint {
                (placed.x, placed.y) = super::help::top_left(e.pair("pos"), origin, h.entry_spacing, (bw, bh), data.width, data.height);
                placed.w = (bw - h.entry_spacing).max(0.0);
            }
            let mut wait = None;
            let mut paint = draft.paint;
            if let (Paint::Text(t), Some((rule, vertical)), Some((mw, mh))) = (&mut paint, draft.scroll, measured) {
                if vertical {
                    let travel = mh - bh;
                    if travel > 0.0 {
                        let (y, opacity) = rule.run_at(since, travel);
                        t.scroll = Some(Scroll { x: 0.0, repeat_at: None, y, opacity });
                        wait = rule.run_wait(since, travel);
                    }
                } else if mw > bw + 0.01 {
                    t.scroll =
                        Some(Scroll { x: rule.loop_offset(since, mw), repeat_at: Some(mw + rule.gap.max(0.0)), y: 0.0, opacity: 1.0 });
                    wait = rule.loop_wait(since, mw);
                }
            }
            // A video starts once its delay has passed since the game was selected; until then the scene waits on it.
            if let Paint::Video(v) = &mut paint {
                v.anchor = (v.anchor.0 - placed.x, v.anchor.1 - placed.y);
                let delay = super::motion::ms(v.delay);
                v.playing = since >= delay;
                if !v.playing {
                    wait = Some(delay - since);
                }
            }
            placed.paint = paint;
            out.push(Drawn { placed, content, wait });
        }
        (out, primary_out)
    }

    /// The quick-scroll letters while a held list has faded its metadata out (§29).
    pub fn scroll_letters(&self) -> Option<ScrollLetters> {
        let data = &self.data;
        let on = data.scroll_overlay
            && self.view == "gamelist"
            && self.theme_view().primary().is_some_and(|p| p.kind() == "textlist")
            && self.quick_scrolling();
        let game = data.game().filter(|_| on)?;
        Some(ScrollLetters {
            letters: game.name.chars().take(2).collect(),
            star: game.favorite && data.system().favorites_on_top,
            px: data.height * 0.15,
            opacity: 1.0,
        })
    }
}
