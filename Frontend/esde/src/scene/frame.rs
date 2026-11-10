//! The scene's output, the contract the painter binds to: one `Frame` per tick, every record in window pixels at the
//! scene's time (EmuSen_Qt.md §4.5.5). Plain data; the painter animates nothing of its own.

use crate::loader::Color;
use crate::marks::MarkId;

/// Everything on the screen at a time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    /// In drawing order: the leaving view's then the current view's during a slide, the overlays held still.
    pub elements: Vec<Placed>,
    pub primary: Option<Primary>,
    pub leaving_primary: Option<Primary>,
    /// Something moves now, so the painter asks for the next frame.
    pub moving: bool,
    /// While nothing moves, how long until something will (a marquee's delay, a container's pause, the clock's minute).
    pub next_change_ms: Option<u32>,
    /// The quick-scroll letters over a held list.
    pub overlay: Option<ScrollLetters>,
}

/// One drawn element.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    /// Stable for the element across frames: its view, kind and name.
    pub id: u64,
    pub kind: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub rotation: f64,
    /// Fractions of the box.
    pub rotation_origin: (f64, f64),
    pub opacity: f64,
    pub paint: Paint,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Paint {
    Picture(Picture),
    Text(Text),
    Rating(Rating),
    Badges(Badges),
    Help(Help),
    Info(Info),
    Status(Status),
    /// A video file played in the box (the media viewer's, EmuSen_Qt.md §4.10).
    Clip(Clip),
    /// A page of a PDF file drawn to the box (the media viewer's manual, EmuSen_Qt.md §4.13).
    Page(Page),
    /// The view's primary element, drawn from `Frame::primary` (or `leaving_primary` when true).
    Primary {
        leaving: bool,
    },
}

/// The media viewer's video under the player's settings: stretched to the box or fitted inside it, with its sound or
/// without, and its horizontal blur's Gaussian sigma in pixels, 0 for none (USERGUIDE.md, "Media viewer settings").
#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub source: String,
    pub stretch: bool,
    pub audio: bool,
    pub blur: f64,
}

/// A PDF file's page, counted from 0.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    pub source: String,
    pub page: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fit {
    Fill,
    #[default]
    Contain,
    Cover,
    Tile,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VAlign {
    Top,
    #[default]
    Center,
    Bottom,
}

/// A colour multiplied over a picture, or two graded across its box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    pub color: Color,
    pub end: Color,
    pub vertical: bool,
}

impl Tint {
    pub const WHITE: Tint = Tint { color: Color(0xFFFF_FFFF), end: Color(0xFFFF_FFFF), vertical: false };
}

#[derive(Clone, Debug, PartialEq)]
pub struct Picture {
    pub source: String,
    pub fit: Fit,
    /// Where a cover crop is centred, fractions of the source.
    pub crop_pos: (f64, f64),
    pub tile_size: (f64, f64),
    pub tile_align: (HAlign, VAlign),
    pub tint: Tint,
    pub saturation: f64,
    pub corner_radius: f64,
    pub nearest: bool,
    /// A GIF's frame at the time; None until the painter reports the file's frame delays.
    pub frame: Option<u32>,
}

/// A fill behind text or an indicator, grown outward from the box by its padding (left, top, right, bottom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Background {
    pub tint: Tint,
    pub radius: f64,
    pub padding: [f64; 4],
}

/// A self-scrolling text's state: its leftward offset and the distance to the copy that follows, or its upward offset and opacity.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Scroll {
    pub x: f64,
    pub repeat_at: Option<f64>,
    pub y: f64,
    pub opacity: f64,
}

/// A text's face: the theme's font file, else ES-DE's own face where the player has it (none: the painter's body face),
/// and whether it is ES-DE's own, which ES-DE draws at its size rounded up to a whole pixel (EmuSen_Qt.md §4.9.1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Face {
    pub file: Option<String>,
    pub own: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Text {
    /// Already in its letter case.
    pub text: String,
    pub font: Face,
    pub px: f64,
    pub color: Color,
    pub h_align: HAlign,
    pub v_align: VAlign,
    pub line_spacing: f64,
    pub wrap: bool,
    pub ellipsis: bool,
    pub background: Option<Background>,
    /// None before the painter has measured the text, and when it fits.
    pub scroll: Option<Scroll>,
}

/// A picture file of the theme's, or a built-in mark.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Icon {
    Path(String),
    Mark(MarkId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rating {
    /// 0 to 1.
    pub value: f64,
    /// The theme's star files, or the star mark, five across the box.
    pub filled: Icon,
    pub unfilled: Icon,
    pub tint: Color,
    /// The filled stars over the unfilled (true) or beside them.
    pub overlay: bool,
}

/// An icon in a box of the element's, and the colour it is multiplied by.
#[derive(Clone, Debug, PartialEq)]
pub struct Overlay {
    pub icon: Icon,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub tint: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Badge {
    /// The slot's cell in the element's box.
    pub cell: (f64, f64, f64, f64),
    /// The badge: a file fills the cell, a mark a square centred in it.
    pub icon: Overlay,
    pub controller: Option<Overlay>,
    /// The link over a linked folder; when both are marks the folder is cut where the link crosses it.
    pub link: Option<Overlay>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Badges {
    pub slots: Vec<Badge>,
}

/// A pad button by its place, as THEMES.md's help entries name them (a, b, x, y in Xbox positions).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadButton {
    DpadUpDown,
    DpadLeftRight,
    South,
    East,
    West,
    North,
    LeftShoulder,
    RightShoulder,
    LeftTrigger,
    RightTrigger,
    Select,
    Start,
    StickClick,
    /// The whole d-pad, its four directions as one (the viewer's pan).
    Dpad,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HelpEntry {
    /// In the element's letter case.
    pub label: String,
    pub button: PadButton,
    /// The button is a pair of directions (up/down, left/right).
    pub pair: bool,
    /// The theme's customButtonIcon file for the button, drawn in the glyph's place.
    pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Help {
    pub entries: Vec<HelpEntry>,
    pub font: Face,
    pub px: f64,
    pub label_color: Color,
    pub icon_color: Color,
    pub background: Option<Background>,
    pub entry_spacing: f64,
    pub icon_gap: f64,
    pub scale: f64,
    /// Letter-labelled filled discs, as ES-DE draws its help.
    pub filled_glyphs: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InfoItem {
    pub mark: MarkId,
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Info {
    pub items: Vec<InfoItem>,
    pub font: Face,
    pub px: f64,
    pub color: Color,
    pub background: Color,
    pub h_align: HAlign,
    pub v_align: VAlign,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatusItem {
    pub icon: Option<Icon>,
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub items: Vec<StatusItem>,
    /// The icons' height; a theme's icon file keeps its own width for that height.
    pub height: f64,
    pub font: Face,
    /// The percentage's size over the height.
    pub text_scale: f64,
    pub color: Color,
    pub background: Option<Background>,
    pub entry_spacing: f64,
}

/// A list the primary element shows: the systems, or one system's games.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rows {
    pub view_is_system: bool,
    pub system: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Marker {
    #[default]
    None,
    Star,
    Folder,
    FolderLink,
    Tick,
}

/// One item of a primary list.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub text: String,
    pub image: Option<String>,
    pub marker: Marker,
    pub folder: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Primary {
    /// The primary element's `Placed::id`.
    pub id: u64,
    pub rows: Rows,
    pub selected: usize,
    pub layout: Layout,
}

/// Text set in an item: font, size, colour and background.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemText {
    pub font: Face,
    pub px: f64,
    pub line_spacing: f64,
    pub color: Color,
    pub background: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Layout {
    Carousel(Carousel),
    TextList(TextList),
    Grid(Grid),
}

/// A carousel's item at the time: its box before scale and turn, both about their own points, in the element's box.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub row: usize,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub scale: f64,
    pub scale_origin: (f64, f64),
    pub rotation: f64,
    pub rotation_origin: (f64, f64),
    pub opacity: f64,
    /// Multiplies the colour of a picture or a text.
    pub dim: f64,
    pub saturation: f64,
    pub tint: Color,
    /// A horizontal row's mirror image under the fitted picture: its opacity at the picture's edge and how fast it fades.
    pub reflection: Option<(f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Carousel {
    /// Farthest from the selection first.
    pub slots: Vec<Slot>,
    pub background: Option<Tint>,
    pub fit: Fit,
    /// Where a contained picture sits in its item box.
    pub image_align: (HAlign, VAlign),
    pub text: ItemText,
}

/// A rectangle in the element's box and its colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextList {
    pub first_row: usize,
    pub row_h: f64,
    /// The height the text is centred in, from the row's top.
    pub band_h: f64,
    pub content_y: f64,
    pub selector: Option<Bar>,
    /// The selected row's background, fitted to its text and grown by margins (left, right).
    pub selected_background: Option<(Color, f64, (f64, f64))>,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub selected_color: Color,
    pub selected_secondary_color: Color,
    pub font: Face,
    pub px: f64,
    pub line_spacing: f64,
    pub margin: f64,
    pub h_align: HAlign,
    /// The marker's room before a name, in pixels.
    pub marker_w: f64,
    /// The selected row's scroll, once measured.
    pub marquee: Option<Scroll>,
}

/// A grid's layout: cell i sits at (margin + column × pitch, margin + row × pitch − content_y).
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub columns: usize,
    pub item: (f64, f64),
    pub pitch: (f64, f64),
    pub margin: (f64, f64),
    pub content_y: f64,
    pub item_scale: f64,
    /// The item the move left and its level then, the selection's level then, and the move's progress.
    pub focus_from: Option<usize>,
    pub from_level: f64,
    pub to_level: f64,
    pub progress: f64,
    pub unfocused_opacity: f64,
    pub unfocused_saturation: f64,
    pub unfocused_dim: f64,
    pub fit: Fit,
    pub crop_pos: (f64, f64),
    pub nearest: bool,
    pub image_scale: f64,
    pub image_radius: f64,
    pub tint: Tint,
    pub selected_tint: Option<Color>,
    pub saturation: f64,
    pub background: Option<Color>,
    pub background_image: Option<String>,
    pub background_scale: f64,
    pub background_radius: f64,
    pub selector: Option<Color>,
    pub selector_image: Option<String>,
    pub selector_scale: f64,
    pub selector_radius: f64,
    /// bottom, middle or top.
    pub selector_layer: String,
    pub text: ItemText,
    pub text_selected_color: Option<Color>,
    pub text_selected_background: Option<Color>,
    pub text_radius: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollLetters {
    pub letters: String,
    pub star: bool,
    pub px: f64,
    pub opacity: f64,
}
