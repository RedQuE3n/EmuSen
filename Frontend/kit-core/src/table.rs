//! A table's columns kept inside its width (`FitsWidth`, LunaP §204): content columns at their words' width, fill columns
//! sharing the rest by weight, nothing ever past the table's right edge, so no table scrolls sideways (EmuSen_Kit.md §7.39).

/// How a column takes its width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnWidth {
    /// Its widest words, as the view measured them; trimmed only where a fill column would fall under its least.
    Content(f64),
    /// Its `least`, then a share by weight of what the content columns and every fill's least leave; where the room
    /// cannot give every fill its least, the leasts are scaled into it.
    Fill { weight: f64, least: f64 },
}

/// The table's own margins: the mark column at the left, the gap between columns, the room after the last.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableMeasures {
    pub lead: f64,
    pub gap: f64,
    pub trail: f64,
}

impl TableMeasures {
    /// Mistress's big-screen table, as the Cores recording places its cells (EmuSen_Interface.md §15.2).
    pub const BIG_SCREEN: TableMeasures = TableMeasures { lead: 24.0, gap: 24.0, trail: 24.0 };
}

/// One column's place from the table's left edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Column {
    pub x: f64,
    pub width: f64,
}

/// The room the columns share at `width`: the table less its margins and gaps, never below nothing.
pub fn room(width: f64, count: usize, m: &TableMeasures) -> f64 {
    (width - m.lead - m.trail - m.gap * count.saturating_sub(1) as f64).max(0.0)
}

/// Every column's place at `width`. Content columns take their words' width (whole pixels, rounded up); fill columns
/// share the rest by weight. Where the fill columns would fall under their least, the content columns give up the
/// difference in proportion to their widths; where even that cannot fit, every column is scaled into the room. The
/// columns' widths never sum past the room, so the last column ends at or before `width - trail`.
pub fn fit_columns(width: f64, columns: &[ColumnWidth], m: &TableMeasures) -> Vec<Column> {
    let room = room(width, columns.len(), m);
    let content: f64 = columns.iter().map(|c| if let ColumnWidth::Content(w) = c { w.max(0.0).ceil() } else { 0.0 }).sum();
    let weight: f64 = columns.iter().map(|c| if let ColumnWidth::Fill { weight, .. } = c { weight.max(0.0) } else { 0.0 }).sum();
    let least: f64 = columns.iter().map(|c| if let ColumnWidth::Fill { least, .. } = c { least.max(0.0) } else { 0.0 }).sum();
    let fills = columns.iter().any(|c| matches!(c, ColumnWidth::Fill { .. }));
    // What the content columns keep, as a share of their words' width.
    let kept = if content <= 0.0 {
        1.0
    } else if fills {
        ((room - least.min(room)) / content).min(1.0)
    } else {
        (room / content).min(1.0)
    };
    let left = (room - content * kept).max(0.0);
    let widths: Vec<f64> = columns
        .iter()
        .map(|c| match *c {
            ColumnWidth::Content(w) => w.max(0.0).ceil() * kept,
            ColumnWidth::Fill { weight: w, least: l } => {
                if weight > 0.0 && left >= least {
                    l.max(0.0) + (left - least) * w.max(0.0) / weight
                } else if least > 0.0 {
                    l.max(0.0) * left / least
                } else {
                    left / columns.iter().filter(|c| matches!(c, ColumnWidth::Fill { .. })).count() as f64
                }
            }
        })
        .collect();
    let mut x = m.lead;
    widths
        .into_iter()
        .map(|width| {
            let column = Column { x, width };
            x += width + m.gap;
            column
        })
        .collect()
}
