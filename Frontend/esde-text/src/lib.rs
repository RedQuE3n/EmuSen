//! ES-DE's text rules over a face (EmuSen_Qt.md §4.9): ES-DE's own face at its size rounded up to a whole pixel, a theme's at its own, each glyph at a
//! whole-pixel pen advanced by its shaped advance rounded, lines broken at spaces within the box's width, a pitch of the
//! whole size times `lineSpacing`, whole lines only in a box of fixed height, and an ellipsis where the box holds one line.
//! Plain data in and out; the painter draws the glyphs where the layout puts them.

use harfrust::{shape, Buffer, Font, ShapeOptions, ShaperFont};

/// The shaper's units in a pixel: fine enough that an advance's rounding is the exact value's (§4.12.1).
const FINE: f64 = 65536.0;

/// The ellipsis THEMES.md names, "(...)": three full stops.
pub const ELLIPSIS: &str = "...";

/// A font's face, read from its file's bytes by read-fonts and shaped by HarfRust (EmuSen_Qt.md §4.9.2).
pub struct Face {
    font: Font,
}

/// A face's ascent and descent at a whole size, in pixels, both positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub ascent: f64,
    pub descent: f64,
}

impl Face {
    /// The face of a font file's bytes, or none when they hold no face.
    pub fn new(data: Vec<u8>) -> Option<Face> {
        let font = Font::new(data, 0)?;
        font.metrics().hhea_line?;
        Some(Face { font })
    }

    /// The face's own line at a size: its ascender and descender scaled, the typographic ones where the face asks for them.
    pub fn metrics(&self, size: f64) -> Metrics {
        let m = self.font.metrics();
        let line = if m.use_typo_metrics { m.typo_line.or(m.hhea_line) } else { m.hhea_line }.expect("checked in new");
        let scale = size / f64::from(m.units_per_em);
        Metrics { ascent: line.ascender.to_f64() * scale, descent: -line.descender.to_f64() * scale }
    }

    /// Each cluster of `text` shaped at `size` with the face's kerning, and its advance: the sum of its glyphs' advances,
    /// each rounded to a whole pixel from its exact value, not from 64ths of a pixel, which round 6.4944 up as 6.5
    /// (§4.9.1, 2; §4.12.1); in ES-DE's own face (`own`) a space's is rounded down (§4.9.1, 3).
    pub fn shape(&self, text: &str, size: f64, own: bool) -> Vec<Piece> {
        if text.is_empty() {
            return vec![];
        }
        let shaper = ShaperFont::new(&self.font).with_scale((size * FINE).round() as i32);
        let mut buffer = Buffer::new();
        buffer.push_str(text);
        buffer.guess_segment_properties();
        shape(&shaper, &mut buffer, ShapeOptions::new()).expect("a buffer with its segment's properties shapes");
        let mut pieces: Vec<(usize, i32)> = vec![];
        for (info, pos) in buffer.glyph_infos().iter().zip(buffer.glyph_positions()) {
            let start = info.cluster as usize;
            let exact = f64::from(pos.x_advance) / FINE;
            let space = text[start..].starts_with(' ');
            let advance = if own && space { exact.floor() } else { exact.round() } as i32;
            match pieces.last_mut() {
                Some(last) if last.0 == start => last.1 += advance,
                _ => pieces.push((start, advance)),
            }
        }
        pieces.sort_by_key(|p| p.0);
        let ends: Vec<usize> = pieces.iter().skip(1).map(|p| p.0).chain([text.len()]).collect();
        pieces.iter().zip(ends).map(|(&(start, advance), end)| Piece { text: text[start..end].to_owned(), advance }).collect()
    }
}

/// One shaped cluster: its characters and its whole-pixel advance.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub text: String,
    pub advance: i32,
}

impl Piece {
    fn is_space(&self) -> bool {
        self.text.chars().all(char::is_whitespace)
    }
}

/// The size ES-DE draws its own face at: THEMES.md's size in FreeType's unit, a 64th of a pixel, the fraction dropped,
/// then rounded up to a whole pixel (§4.9.1, 1); never under one. A theme's own face is drawn at its size as given. A theme's decimal times the screen's height lands a hair off a whole size (0.02916667 ×
/// 1200 is 35.000004), which the 64ths leave whole.
pub fn whole(px: f64) -> u32 {
    ((px * 64.0).floor() / 64.0).ceil().max(1.0) as u32
}

/// Where a line stands in its box's width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}

/// What to lay out: the words, the theme's size and whether the face is ES-DE's own (the theme named none), the box (0 for no bound on either side), `lineSpacing` (0 for the
/// face's own line), whether the words wrap, whether a line cut by the box ends in an ellipsis, and the lines' alignment.
#[derive(Clone, Copy, Debug)]
pub struct Spec<'a> {
    pub text: &'a str,
    pub px: f64,
    pub own_face: bool,
    pub width: f64,
    pub height: f64,
    pub line_spacing: f64,
    pub wrap: bool,
    pub ellipsis: bool,
    pub align: Align,
}

/// A glyph's characters and its pen in its line.
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub text: String,
    pub x: i32,
}

/// A line: its glyphs, its width (the pen past the last glyph, trailing spaces left out), its place in the box's width
/// and its top, both whole pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub glyphs: Vec<Glyph>,
    pub width: i32,
    pub x: i32,
    pub y: i32,
}

/// The laid-out words: the size drawn, the pitch, the face's line at that size, the lines, and the widest line's width.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub size: f64,
    pub pitch: f64,
    pub metrics: Metrics,
    pub lines: Vec<Line>,
    pub width: i32,
}

impl Layout {
    /// The lines' height: the pitch for each line, one for no words.
    pub fn height(&self) -> f64 {
        self.lines.len().max(1) as f64 * self.pitch
    }

    /// A line's baseline below its top: the face's own line centred in the pitch (§4.9.1, 3).
    pub fn baseline(&self) -> f64 {
        (self.pitch - self.metrics.ascent - self.metrics.descent) / 2.0 + self.metrics.ascent
    }
}

// The width of a run of pieces with its trailing spaces left out.
fn width(pieces: &[Piece]) -> i32 {
    let end = pieces.iter().rposition(|p| !p.is_space()).map_or(0, |i| i + 1);
    pieces[..end].iter().map(|p| p.advance).sum()
}

// A paragraph's pieces broken into lines no wider than `max`: at the last space that fits, else before the piece that
// does not, a long word broken where it overflows; the space at a break starts no line.
fn wrapped(pieces: Vec<Piece>, max: f64) -> Vec<Vec<Piece>> {
    let mut lines = vec![];
    let mut line: Vec<Piece> = vec![];
    for piece in pieces {
        if line.is_empty() && piece.is_space() && !lines.is_empty() {
            continue;
        }
        line.push(piece);
        if f64::from(width(&line)) <= max || line.len() == 1 {
            continue;
        }
        let last = line.pop().expect("pushed");
        match line.iter().rposition(Piece::is_space) {
            Some(space) if space > 0 => {
                let rest = line.split_off(space + 1);
                lines.push(std::mem::replace(&mut line, rest));
            }
            _ => lines.push(std::mem::take(&mut line)),
        }
        if !(last.is_space() && line.is_empty()) {
            line.push(last);
        }
    }
    lines.push(line);
    lines
}

// A line cut to `max` with the ellipsis after its last piece that fits, trailing spaces dropped before it.
fn ellipsized(face: &Face, mut line: Vec<Piece>, max: f64, size: f64, own: bool) -> Vec<Piece> {
    let dots = face.shape(ELLIPSIS, size, own);
    let room = max - f64::from(dots.iter().map(|p| p.advance).sum::<i32>());
    while !line.is_empty() && (f64::from(width(&line)) > room || line.last().is_some_and(Piece::is_space)) {
        line.pop();
    }
    line.extend(dots);
    line
}

/// The words laid out by ES-DE's rules (§4.9.1).
pub fn layout(face: &Face, spec: &Spec<'_>) -> Layout {
    let size = if spec.own_face { f64::from(whole(spec.px)) } else { spec.px };
    let metrics = face.metrics(size);
    let pitch = if spec.line_spacing > 0.0 { size * spec.line_spacing } else { metrics.ascent + metrics.descent };
    let fits = if spec.height > 0.0 { ((spec.height + 1e-6) / pitch).floor().max(1.0) as usize } else { usize::MAX };
    let bounded = spec.width > 0.0;
    // A box one line high holds the words on one line, cut with an ellipsis (THEMES.md, size `w h`).
    let one = fits == 1 || !spec.wrap;
    let paragraphs: Vec<String> = if fits == 1 { vec![spec.text.replace('\n', " ")] } else { spec.text.split('\n').map(str::to_owned).collect() };
    let mut runs: Vec<Vec<Piece>> = vec![];
    for paragraph in &paragraphs {
        let pieces = face.shape(paragraph, size, spec.own_face);
        if !one && bounded {
            runs.extend(wrapped(pieces, spec.width));
        } else if spec.ellipsis && bounded && f64::from(width(&pieces)) > spec.width {
            runs.push(ellipsized(face, pieces, spec.width, size, spec.own_face));
        } else {
            runs.push(pieces);
        }
    }
    runs.truncate(fits);
    let lines: Vec<Line> = runs
        .into_iter()
        .enumerate()
        .map(|(i, run)| {
            let mut pen = 0;
            let glyphs = run
                .iter()
                .map(|p| {
                    let g = Glyph { text: p.text.clone(), x: pen };
                    pen += p.advance;
                    g
                })
                .collect();
            let w = width(&run);
            let room = if bounded { spec.width - f64::from(w) } else { 0.0 };
            let x = match spec.align {
                Align::Start => 0.0,
                Align::Center => room / 2.0,
                Align::End => room,
            };
            Line { glyphs, width: w, x: x.round() as i32, y: (i as f64 * pitch).round() as i32 }
        })
        .collect();
    let width = lines.iter().map(|l| l.width).max().unwrap_or(0);
    Layout { size, pitch, metrics, lines, width }
}
