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
        if item.is_drawn() {
            past_its_limit(item, ancestors, panel, slack, &mut faults);
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
    let (limit, scrolls_down, limit_id) = match limit_of(ancestors) {
        Some(a) => (a.rect, a.scroll.vertical, Some(a.id)),
        None => (panel, false, None),
    };
    let across = limit.holds_across(&item.rect, slack);
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
    let Some(area) = ancestors.iter().rev().find(|a| a.scroll.vertical || a.scroll.horizontal) else { return };
    let r = item.rect;
    let a = area.rect;
    let crosses = |near: f64, far: f64, edge: f64| near < edge - slack && far > edge + slack;
    let mut cut = |side: Side, fades: bool| {
        if !fades {
            faults.push(Fault::CutAtEdge { item: item.id, area: area.id, side });
        }
    };
    if area.scroll.vertical {
        if crosses(r.y, r.bottom(), a.y) {
            cut(Side::Top, area.fades.top);
        }
        if crosses(r.y, r.bottom(), a.bottom()) {
            cut(Side::Bottom, area.fades.bottom);
        }
    } else {
        if crosses(r.x, r.right(), a.x) {
            cut(Side::Left, area.fades.left);
        }
        if crosses(r.x, r.right(), a.right()) {
            cut(Side::Right, area.fades.right);
        }
    }
}

fn words(item: &Item, ancestors: &[&Item], slack: f64, smallest: f64, faults: &mut Vec<Fault>) {
    if let Some(owner) = ancestors.last()
        && matches!(owner.kind, Kind::Control | Kind::Row)
        && !owner.rect.holds(&item.rect, slack)
    {
        faults.push(Fault::WordsPastOwner { words: item.id, owner: owner.id });
    }
    let Some(text) = item.text else { return };
    if text.ellipsis {
        faults.push(Fault::Ellipsis { item: item.id });
    }
    if text.width > item.rect.w + slack {
        faults.push(Fault::RunsPastItsBox { item: item.id });
    }
    if text.height > item.rect.h + slack {
        faults.push(Fault::CutAtFoot { item: item.id });
    }
    if smallest > 0.0 && text.cap_height < smallest - CAPS_SLACK {
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
            let shared = a.rect.overlap(&b.rect);
            if shared.w > slack && shared.h > slack {
                faults.push(Fault::Overlap { a: a.id, b: b.id });
            }
        }
    }
}
