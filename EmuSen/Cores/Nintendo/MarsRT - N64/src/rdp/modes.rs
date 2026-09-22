//! Other modes, the combiner's selectors and the colour registers: C#'s `Rdp.Modes.cs`.

use super::{Color, Rdp, sign_extend};

pub const SET_KEY_GREEN_BLUE: u32 = 0x2A;
pub const SET_KEY_RED: u32 = 0x2B;
pub const SET_CONVERT: u32 = 0x2C;
pub const SET_PRIMITIVE_DEPTH: u32 = 0x2E;
pub const SET_FOG_COLOR: u32 = 0x38;
pub const SET_BLEND_COLOR: u32 = 0x39;
pub const SET_PRIMITIVE_COLOR: u32 = 0x3A;
pub const SET_ENVIRONMENT_COLOR: u32 = 0x3B;
pub const SET_COMBINE: u32 = 0x3C;
pub const SET_MASK_IMAGE: u32 = 0x3E;

/// Which colour and alpha a blender cycle weighs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BlendSelectors {
    pub first_color: i32,
    pub first_alpha: i32,
    pub second_color: i32,
    pub second_alpha: i32,
}

/// A combiner cycle's eight inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CombinerSelectors {
    pub color_a: i32,
    pub color_b: i32,
    pub color_c: i32,
    pub color_d: i32,
    pub alpha_a: i32,
    pub alpha_b: i32,
    pub alpha_c: i32,
    pub alpha_d: i32,
}

/// The fields C# marks `[SkipInState]` and decodes from `_otherModes` and `_combine`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Modes {
    pub cycle_type: i32,
    pub perspective: bool,
    pub detail_enabled: bool,
    pub sharpen_enabled: bool,
    pub lod_enabled: bool,
    pub palette_enabled: bool,
    pub palette_intensity_alpha: bool,
    pub sample_four: bool,
    pub mid_texel: bool,
    pub bilinear_first_cycle: bool,
    pub rgb_dither: i32,
    pub alpha_dither: i32,
    pub dither_table: usize,
    pub key_enabled: bool,
    pub bilinear_second_cycle: bool,
    pub convert_one: bool,
    /// The one-cycle mode's blender reads this cycle; its fields are `_blendFirstColor` to `_blendSecondAlpha`.
    pub first_blend_cycle: BlendSelectors,
    pub second_blend_cycle: BlendSelectors,
    pub force_blend: bool,
    pub alpha_from_coverage: bool,
    pub coverage_times_alpha: bool,
    pub depth_mode: i32,
    pub coverage_destination: i32,
    pub color_on_coverage: bool,
    pub image_read: bool,
    pub depth_update: bool,
    pub depth_compare: bool,
    pub antialias: bool,
    pub primitive_depth: bool,
    pub dither_alpha: bool,
    pub alpha_compare: bool,
    pub first_combine_cycle: CombinerSelectors,
    /// The one-cycle mode's combiner reads this cycle; its fields are `_combineColorA` to `_combineAlphaD`.
    pub second_combine_cycle: CombinerSelectors,
}

/// Derived state: always equal, since two processors with equal state decode equal modes.
impl PartialEq for Modes {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Modes {}

impl Color {
    pub fn from_word(word: u64) -> Color {
        Color { r: ((word >> 24) & 0xFF) as i32, g: ((word >> 16) & 0xFF) as i32, b: ((word >> 8) & 0xFF) as i32, a: (word & 0xFF) as i32 }
    }

    /// A scalar input, the same on every colour channel.
    #[inline(always)]
    pub fn broadcast(value: i32) -> Color {
        Color { r: value, g: value, b: value, a: 0 }
    }
}

impl Rdp {
    /// C#'s `Refresh`: after every write of either word, and after a loaded state.
    pub fn refresh(&mut self) {
        self.decode_other_modes();
        self.decode_combine();
    }

    pub(super) fn set_register(&mut self, id: u32, word: u64) -> bool {
        match id {
            SET_COMBINE => {
                self.combine = word;
                self.decode_combine();
            }
            SET_PRIMITIVE_COLOR => {
                self.primitive_color = Color::from_word(word);
                self.primitive_lod_fraction = ((word >> 32) & 0xFF) as i32;
                self.min_level = ((word >> 40) & 0x1F) as i32;
            }
            SET_ENVIRONMENT_COLOR => self.environment_color = Color::from_word(word),
            SET_BLEND_COLOR => self.blend_color = Color::from_word(word),
            SET_FOG_COLOR => self.fog_color = Color::from_word(word),
            SET_CONVERT => {
                self.k0 = (sign_extend((word >> 45) as u32, 9) << 1) + 1;
                self.k1 = (sign_extend((word >> 36) as u32, 9) << 1) + 1;
                self.k2 = (sign_extend((word >> 27) as u32, 9) << 1) + 1;
                self.k3 = (sign_extend((word >> 18) as u32, 9) << 1) + 1;
                self.k4 = ((word >> 9) & 0x1FF) as i32;
                self.k5 = (word & 0x1FF) as i32;
            }
            SET_KEY_GREEN_BLUE => {
                self.key_width.g = ((word >> 44) & 0xFFF) as i32;
                self.key_width.b = ((word >> 32) & 0xFFF) as i32;
                self.key_center.g = ((word >> 24) & 0xFF) as i32;
                self.key_scale.g = ((word >> 16) & 0xFF) as i32;
                self.key_center.b = ((word >> 8) & 0xFF) as i32;
                self.key_scale.b = (word & 0xFF) as i32;
            }
            SET_KEY_RED => {
                self.key_width.r = ((word >> 16) & 0xFFF) as i32;
                self.key_center.r = ((word >> 8) & 0xFF) as i32;
                self.key_scale.r = (word & 0xFF) as i32;
            }
            SET_PRIMITIVE_DEPTH => {
                self.primitive_delta_z = (word & 0xFFFF) as i32;
                self.primitive_z = ((word as u32) & (0x7FFF << 16)) as i32;
            }
            SET_MASK_IMAGE => {
                self.depth_image = (word as u32) & 0x00FF_FFFF;
                self.split.depth_drawn_to = 0;
            }
            _ => return false,
        }
        true
    }

    pub(super) fn decode_other_modes(&mut self) {
        let o = self.other_modes;
        let bit = |n: u32| ((o >> n) & 1) != 0;
        let two = |n: u32| ((o >> n) & 3) as i32;
        let m = &mut self.modes;
        m.cycle_type = two(52);
        m.perspective = bit(51);
        m.detail_enabled = bit(50);
        m.sharpen_enabled = bit(49);
        m.lod_enabled = bit(48);
        m.palette_enabled = bit(47);
        m.palette_intensity_alpha = bit(46);
        m.sample_four = bit(45);
        m.mid_texel = bit(44);
        m.bilinear_first_cycle = bit(43);
        m.rgb_dither = two(38);
        m.alpha_dither = two(36);
        m.dither_table = ((m.rgb_dither << 2) | m.alpha_dither) as usize;
        m.key_enabled = bit(40);
        m.bilinear_second_cycle = bit(42);
        m.convert_one = bit(41);
        m.first_blend_cycle = BlendSelectors { first_color: two(30), first_alpha: two(26), second_color: two(22), second_alpha: two(18) };
        m.second_blend_cycle = BlendSelectors { first_color: two(28), first_alpha: two(24), second_color: two(20), second_alpha: two(16) };
        m.force_blend = bit(14);
        m.alpha_from_coverage = bit(13);
        m.coverage_times_alpha = bit(12);
        m.depth_mode = two(10);
        m.coverage_destination = two(8);
        m.color_on_coverage = bit(7);
        m.image_read = bit(6);
        m.depth_update = bit(5);
        m.depth_compare = bit(4);
        m.antialias = bit(3);
        m.primitive_depth = bit(2);
        m.dither_alpha = bit(1);
        m.alpha_compare = bit(0);
    }

    pub(super) fn decode_combine(&mut self) {
        let c = self.combine;
        let f = |n: u32, mask: u64| ((c >> n) & mask) as i32;
        self.modes.second_combine_cycle = CombinerSelectors {
            color_a: f(37, 0xF),
            color_b: f(24, 0xF),
            color_c: f(32, 0x1F),
            color_d: f(6, 7),
            alpha_a: f(21, 7),
            alpha_b: f(3, 7),
            alpha_c: f(18, 7),
            alpha_d: f(0, 7),
        };
        self.modes.first_combine_cycle = CombinerSelectors {
            color_a: f(52, 0xF),
            color_b: f(28, 0xF),
            color_c: f(47, 0x1F),
            color_d: f(15, 7),
            alpha_a: f(44, 7),
            alpha_b: f(12, 7),
            alpha_c: f(41, 7),
            alpha_d: f(9, 7),
        };
    }
}
