use super::accepted::Class;
use super::tree::{Item, Kind, Rect};
use super::{Fault, Side};

/// Inter's capitals as a share of its size, measured from its H: the desktop face the floor is set by (Settings Reference §4.83.7).
pub const DESKTOP_CAPS: f64 = 0.7273;

/// The smallest words on a big screen, as a size of the desktop face, in design pixels.
pub const SMALLEST_TEXT: f64 = 16.0;

/// The audit's slack on the capitals' height: 0.05 of a pixel of the desktop face.
pub const CAPS_SLACK: f64 = 0.05 * DESKTOP_CAPS;

/// A scrolling area must show two rows of 42 design pixels.
pub const TWO_ROWS: f64 = 84.0;

/// Words the capitals' floor does not hold: the scene they belong to (an ancestor's name), their item's name, and the
/// accepted class that lets them pass (EmuSen_Kit.md §7.102).
pub const SMALL_TEXT_EXCEPTIONS: [(&str, &str, Class); 1] = [("moonrise", "moonriseCaption", Class::MoonriseCaption)];

/// The class of `SMALL_TEXT_EXCEPTIONS`' entry that names an item in its scene, if one does; the same name elsewhere is held.
pub fn small_text_exception(item: &Item, ancestors: &[&Item]) -> Option<Class> {
    SMALL_TEXT_EXCEPTIONS
        .iter()
        .find(|(scene, name, _)| item.name == *name && ancestors.iter().any(|a| a.name == *scene))
        .map(|(_, _, class)| *class)
}

/// The floor on a big screen: the capitals Inter draws at 16 design pixels, 11.64.
pub fn smallest_capitals() -> f64 {
    SMALLEST_TEXT * DESKTOP_CAPS
}

/// The fit audit of Settings Reference §4.83.2 over one screen: `panel` is its frame, `slack` a pixel (a pixel and a half above scale 1), `smallest` the capitals' floor or 0 for a desktop window.
pub fn fit(root: &Item, panel: Rect, slack: f64, smallest: f64) -> Vec<Fault> {
    let mut faults = Vec::new();
    let mut read_or_pressed: Vec<(&Item, Vec<&Item>)> = Vec::new();
    root.visit(&mut |item, ancestors| {
        if ancestors.iter().any(|a| a.kind == Kind::Popup) || item.kind == Kind::Popup {
            return;
        }
        // A scroller or a clip is held to its limit too, though it draws nothing: what it shows lies inside it (decided 2026-11-08).
        let bounds = item.visible && (item.clips || item.kind == Kind::Scroller);
        if item.is_drawn() || bounds {
            past_its_limit(item, ancestors, panel, slack, &mut faults);
        }
        if item.is_drawn() {
            cut_at_an_edge(item, ancestors, slack, &mut faults);
            if item.kind == Kind::Words {
                words(item, ancestors, slack, smallest, &mut faults);
            }
        }
        if item.kind == Kind::Scroller && item.scroll.vertical && item.rect.h < TWO_ROWS - slack {
            faults.push(Fault::ScrollsInTooLittleRoom { item: item.id, height: item.rect.h });
        }
        if item.is_read_or_pressed() {
            read_or_pressed.push((item, ancestors.to_vec()));
        }
    });
    overlaps(&read_or_pressed, slack, &mut faults);
    faults
}

fn limit_of<'a>(ancestors: &[&'a Item]) -> Option<&'a Item> {
    ancestors.iter().rev().find(|a| a.clips || a.scroll.vertical || a.scroll.horizontal).copied()
}

fn past_its_limit(item: &Item, ancestors: &[&Item], panel: Rect, slack: f64, faults: &mut Vec<Fault>) {
    let (limit, scrolls_across, scrolls_down, limit_id) = match limit_of(ancestors) {
        Some(a) => (a.rect, a.scroll.horizontal, a.scroll.vertical, Some(a.id)),
        None => (panel, false, false, None),
    };
    // A scroller excuses what lies beyond it along the axis it scrolls; what crosses its edge is the edge rule's.
    let across = scrolls_across || limit.holds_across(&item.rect, slack);
    let down = scrolls_down || limit.holds_down(&item.rect, slack);
    if across && down {
        return;
    }
    faults.push(match item.kind {
        Kind::Title => Fault::TitleCut { item: item.id },
        Kind::Footer => Fault::FooterCut { item: item.id },
        _ => Fault::PastItsLimit { item: item.id, limit: limit_id },
    });
}

fn cut_at_an_edge(item: &Item, ancestors: &[&Item], slack: f64, faults: &mut Vec<Fault>) {
    let Some(area) = nearest_scroller(ancestors) else { return };
    for (side, fades) in crossed_edges(item, area, slack) {
        if !fades {
            faults.push(Fault::CutAtEdge { item: item.id, area: area.id, side });
        }
    }
}

fn nearest_scroller<'a>(ancestors: &[&'a Item]) -> Option<&'a Item> {
    ancestors.iter().rev().find(|a| a.scroll.vertical || a.scroll.horizontal).copied()
}

/// The edges of a scroller, along the axis it scrolls, that an item crosses, each with whether it fades.
fn crossed_edges(item: &Item, area: &Item, slack: f64) -> Vec<(Side, bool)> {
    let r = item.rect;
    let a = area.rect;
    let crosses = |near: f64, far: f64, edge: f64| near < edge - slack && far > edge + slack;
    let mut sides = Vec::new();
    if area.scroll.vertical {
        if crosses(r.y, r.bottom(), a.y) {
            sides.push((Side::Top, area.fades.top));
        }
        if crosses(r.y, r.bottom(), a.bottom()) {
            sides.push((Side::Bottom, area.fades.bottom));
        }
    } else {
        if crosses(r.x, r.right(), a.x) {
            sides.push((Side::Left, area.fades.left));
        }
        if crosses(r.x, r.right(), a.right()) {
            sides.push((Side::Right, area.fades.right));
        }
    }
    sides
}

/// Whether an item crosses a faded edge of its nearest scroller, and so is drawn fading out there (EmuSen_Kit.md §7.35).
pub(crate) fn fading_out(item: &Item, ancestors: &[&Item]) -> bool {
    nearest_scroller(ancestors).is_some_and(|area| crossed_edges(item, area, 0.0).iter().any(|(_, fades)| *fades))
}

fn words(item: &Item, ancestors: &[&Item], slack: f64, smallest: f64, faults: &mut Vec<Fault>) {
    if let Some(owner) = ancestors.last()
        && matches!(owner.kind, Kind::Control | Kind::Row)
        && !owner.rect.holds(&item.rect, slack)
    {
        faults.push(Fault::WordsPastOwner { words: item.id, owner: owner.id });
    }
    let Some(text) = item.text else { return };
    if text.ellipsis && !text.data {
        faults.push(Fault::Ellipsis { item: item.id });
    }
    let (own_w, own_h) = text.own.unwrap_or((item.rect.w, item.rect.h));
    if text.width > own_w + slack {
        faults.push(Fault::RunsPastItsBox { item: item.id });
    }
    if text.height > own_h + slack {
        faults.push(Fault::CutAtFoot { item: item.id });
    }
    if smallest > 0.0 && text.cap_height < smallest - CAPS_SLACK && small_text_exception(item, ancestors).is_none() {
        faults.push(Fault::TooSmall { item: item.id, capitals: text.cap_height });
    }
}

fn overlaps(items: &[(&Item, Vec<&Item>)], slack: f64, faults: &mut Vec<Fault>) {
    for (n, (a, a_ancestors)) in items.iter().enumerate() {
        for (b, b_ancestors) in &items[n + 1..] {
            if a_ancestors.iter().any(|x| x.id == b.id) || b_ancestors.iter().any(|x| x.id == a.id) {
                continue;
            }
            if a.rect.holds(&b.rect, slack) || b.rect.holds(&a.rect, slack) {
                continue;
            }
            if !shapes_overlap(a, b, slack) {
                continue;
            }
            // Members of one declared stack may lie over each other in their order; words or a focused member under another may not.
            if let (Some(sa), Some(sb)) = (a.stacking, b.stacking)
                && sa.stack == sb.stack
                && sa.member != sb.member
            {
                let (low, low_stacking, high) = if sa.layer < sb.layer { (a, sa, b) } else { (b, sb, a) };
                // Words are covered when their drawn glyphs are, not their layout box, which a card's title fills.
                let glyphs_covered = || match low.inked {
                    Some(inked) => shapes_meet(inked, high.shape(), slack),
                    None => true,
                };
                let hidden = (low.kind == Kind::Words && glyphs_covered()) || (low_stacking.is_member && low_stacking.focused);
                if hidden {
                    faults.push(Fault::Overlap { a: low.id, b: high.id });
                }
                continue;
            }
            faults.push(Fault::Overlap { a: a.id, b: b.id });
        }
    }
}

/// Whether two items' shapes share more than `slack` along every separating axis: their scene boxes when neither is turned,
/// else the turned corners by the separating axis theorem, so two turned boxes whose scene boxes cross but whose shapes do
/// not are apart.
fn shapes_overlap(a: &Item, b: &Item, slack: f64) -> bool {
    if a.corners.is_none() && b.corners.is_none() {
        let shared = a.rect.overlap(&b.rect);
        return shared.w > slack && shared.h > slack;
    }
    shapes_meet(a.shape(), b.shape(), slack)
}

/// Whether two four-cornered shapes share more than `slack` along every separating axis.
fn shapes_meet(pa: [(f64, f64); 4], pb: [(f64, f64); 4], slack: f64) -> bool {
    let axes = [pa, pb].into_iter().flat_map(|p| [(p[1].0 - p[0].0, p[1].1 - p[0].1), (p[2].0 - p[1].0, p[2].1 - p[1].1)]);
    let span = |p: &[(f64, f64); 4], (ax, ay): (f64, f64)| {
        p.iter().map(|(x, y)| x * ax + y * ay).fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| (lo.min(v), hi.max(v)))
    };
    axes.filter(|(x, y)| x.hypot(*y) > 1e-9).all(|(x, y)| {
        let len = x.hypot(y);
        let axis = (x / len, y / len);
        let ((alo, ahi), (blo, bhi)) = (span(&pa, axis), span(&pb, axis));
        ahi.min(bhi) - alo.max(blo) > slack
    })
}
