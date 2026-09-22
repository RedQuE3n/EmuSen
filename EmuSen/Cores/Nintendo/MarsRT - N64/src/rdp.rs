//! The display processor, the C# `Rdp`: its serialized state here, and the rasteriser behind `accept` in `rdp/`. See Mars_Native.md §5.3.

mod chroma_key;
mod copy;
mod coverage;
mod depth;
mod fill;
mod filter;
mod lod;
mod modes;
mod one_cycle;
#[cfg(test)]
mod replay;
mod tables;
mod texture_memory;
mod textures;
mod two_cycle;
mod walker;

pub use modes::{BlendSelectors, CombinerSelectors, Modes};
use texture_memory::LoadKind;

use crate::state::{State, StateReader, StateResult, StateWriter, boxed};

pub const TEXTURE_RECTANGLE: u32 = 0x24;
pub const TEXTURE_RECTANGLE_FLIPPED: u32 = 0x25;
pub const SYNC_FULL: u32 = 0x29;
pub const SET_SCISSOR: u32 = 0x2D;
pub const SET_OTHER_MODES: u32 = 0x2F;
pub const FILL_RECTANGLE: u32 = 0x36;
pub const SET_FILL_COLOR: u32 = 0x37;
pub const SET_COLOR_IMAGE: u32 = 0x3F;
pub const LOAD_PALETTE: u32 = 0x30;
pub const SET_TILE_SIZE: u32 = 0x32;
pub const LOAD_BLOCK: u32 = 0x33;
pub const LOAD_TILE: u32 = 0x34;
pub const SET_TILE: u32 = 0x35;
pub const SET_TEXTURE_IMAGE: u32 = 0x3D;

const ONE_CYCLE: i32 = 0;
const TWO_CYCLE: i32 = 1;
const COPY_CYCLE: i32 = 2;
const FILL_CYCLE: i32 = 3;

/// Red, green, blue, alpha, depth, and the texture's s, t and w.
const ATTRIBUTES: usize = 8;
const ATTRIBUTE_Z: usize = 4;
const ATTRIBUTE_S: usize = 5;
const ATTRIBUTE_T: usize = 6;
const ATTRIBUTE_W: usize = 7;

/// The first and last rows a walk wrote, which are the only ones to draw.
type Rows = (i32, i32);

/// The low `bits` of a field, sign-extended.
#[inline(always)]
fn sign_extend(field: u32, bits: u32) -> i32 {
    ((field << (32 - bits)) as i32) >> (32 - bits)
}

/// A command's number, from its first word.
#[inline(always)]
pub fn command_id(word: u64) -> u32 {
    ((word >> 56) & 0x3F) as u32
}

/// In words: triangles grow by what they carry, texture rectangles take two, everything else one.
#[inline(always)]
pub fn command_length(id: u32) -> i32 {
    match id {
        0x08..=0x0F => 4 + (if (id & 4) != 0 { 8 } else { 0 }) + (if (id & 2) != 0 { 8 } else { 0 }) + (if (id & 1) != 0 { 2 } else { 0 }),
        0x24 | 0x25 => 2,
        _ => 1,
    }
}

/// The C# `Color` struct, whose fields the serializer writes A, B, G, R, by ordinal name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Color {
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: i32,
}

impl State for Color {
    fn write_state(&self, w: &mut StateWriter) {
        w.i32("A", self.a);
        w.i32("B", self.b);
        w.i32("G", self.g);
        w.i32("R", self.r);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.a = r.i32()?; // A
        self.b = r.i32()?; // B
        self.g = r.i32()?; // G
        self.r = r.i32()?; // R
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TextureTile {
    pub clamp_s: bool,
    pub clamp_t: bool,
    pub format: i32,
    pub line: i32,
    pub mask_s: i32,
    pub mask_t: i32,
    pub memory: i32,
    pub mirror_s: bool,
    pub mirror_t: bool,
    pub palette: i32,
    pub sh: i32,
    pub sl: i32,
    pub shift_s: i32,
    pub shift_t: i32,
    pub size: i32,
    pub th: i32,
    pub tl: i32,
}

impl State for TextureTile {
    fn write_state(&self, w: &mut StateWriter) {
        w.bool("ClampS", self.clamp_s);
        w.bool("ClampT", self.clamp_t);
        w.i32("Format", self.format);
        w.i32("Line", self.line);
        w.i32("MaskS", self.mask_s);
        w.i32("MaskT", self.mask_t);
        w.i32("Memory", self.memory);
        w.bool("MirrorS", self.mirror_s);
        w.bool("MirrorT", self.mirror_t);
        w.i32("Palette", self.palette);
        w.i32("SH", self.sh);
        w.i32("SL", self.sl);
        w.i32("ShiftS", self.shift_s);
        w.i32("ShiftT", self.shift_t);
        w.i32("Size", self.size);
        w.i32("TH", self.th);
        w.i32("TL", self.tl);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.clamp_s = r.bool()?; // ClampS
        self.clamp_t = r.bool()?; // ClampT
        self.format = r.i32()?; // Format
        self.line = r.i32()?; // Line
        self.mask_s = r.i32()?; // MaskS
        self.mask_t = r.i32()?; // MaskT
        self.memory = r.i32()?; // Memory
        self.mirror_s = r.bool()?; // MirrorS
        self.mirror_t = r.bool()?; // MirrorT
        self.palette = r.i32()?; // Palette
        self.sh = r.i32()?; // SH
        self.sl = r.i32()?; // SL
        self.shift_s = r.i32()?; // ShiftS
        self.shift_t = r.i32()?; // ShiftT
        self.size = r.i32()?; // Size
        self.th = r.i32()?; // TH
        self.tl = r.i32()?; // TL
        Ok(())
    }
}

/// The C# `Rdp`'s 74 serialized fields. The 1024-row arrays are the walker's scratch, at the scale the machine's own processor draws at, one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rdp {
    /// `<TextureMemory>k__BackingField`, the auto-property's backing field.
    pub texture_memory: Box<[u8; 4096]>,
    pub attribute_de: [i32; 8],
    pub attribute_dx: [i32; 8],
    pub attribute_dy: [i32; 8],
    pub attribute_value: [i32; 8],
    pub blend_color: Color,
    pub blend_shift_a: i32,
    pub blend_shift_b: i32,
    pub blended: Color,
    pub blender_shade_alpha: i32,
    pub color_image: u32,
    pub color_image_bytes: i32,
    pub color_image_format: i32,
    pub color_image_size: i32,
    pub color_image_width: i32,
    pub combine: u64,
    pub combined: Color,
    pub command: [u64; 22],
    pub coverage: Box<[u8; 1024]>,
    pub depth_correct_dx: i32,
    pub depth_correct_dy: i32,
    pub depth_image: u32,
    pub depth_slope: i32,
    pub depth_step: i32,
    pub edge_invalid: Box<[bool; 4096]>,
    pub edge_left: Box<[i32; 4096]>,
    pub edge_right: Box<[i32; 4096]>,
    pub environment_color: Color,
    pub fill_color: u32,
    pub fog_color: Color,
    pub k0: i32,
    pub k1: i32,
    pub k2: i32,
    pub k3: i32,
    pub k4: i32,
    pub k5: i32,
    pub key_center: Color,
    pub key_scale: Color,
    pub key_width: Color,
    pub lod_fraction: i32,
    pub memory: Color,
    pub min_level: i32,
    pub other_modes: u64,
    pub past_shift_a: i32,
    pub past_shift_b: i32,
    pub past_stored_encoded: i32,
    pub pixel: Color,
    pub primitive_color: Color,
    pub primitive_delta_z: i32,
    pub primitive_lod_fraction: i32,
    pub primitive_z: i32,
    pub scissor_bottom: i32,
    pub scissor_field: bool,
    pub scissor_keep_odd: bool,
    pub scissor_left: i32,
    pub scissor_right: i32,
    pub scissor_top: i32,
    pub shade: Color,
    pub shade_correct_dx: [i32; 4],
    pub shade_correct_dy: [i32; 4],
    pub shade_step: [i32; 4],
    pub span_attributes: Box<[i32; 8192]>,
    pub span_drawn: Box<[bool; 1024]>,
    pub span_left: Box<[i32; 1024]>,
    pub span_major_x: Box<[i32; 1024]>,
    pub span_right: Box<[i32; 1024]>,
    pub taken: i32,
    pub texel0: Color,
    pub texel1: Color,
    pub texture_image: u32,
    pub texture_image_size: i32,
    pub texture_image_width: i32,
    pub texture_step: [i32; 3],
    pub tiles: [TextureTile; 8],
    /// `[SkipInState]` in C#: the modes decoded from `other_modes` and `combine`, which `refresh` rebuilds.
    pub modes: Modes,
}

impl Default for Rdp {
    fn default() -> Self {
        let mut rdp = Rdp {
            texture_memory: boxed(0),
            attribute_de: [0; 8],
            attribute_dx: [0; 8],
            attribute_dy: [0; 8],
            attribute_value: [0; 8],
            blend_color: Color::default(),
            blend_shift_a: 0,
            blend_shift_b: 0,
            blended: Color::default(),
            blender_shade_alpha: 0,
            color_image: 0,
            color_image_bytes: 0,
            color_image_format: 0,
            color_image_size: 0,
            color_image_width: 0,
            combine: 0,
            combined: Color::default(),
            command: [0; 22],
            coverage: boxed(0),
            depth_correct_dx: 0,
            depth_correct_dy: 0,
            depth_image: 0,
            depth_slope: 0,
            depth_step: 0,
            edge_invalid: boxed(false),
            edge_left: boxed(0),
            edge_right: boxed(0),
            environment_color: Color::default(),
            fill_color: 0,
            fog_color: Color::default(),
            k0: 0,
            k1: 0,
            k2: 0,
            k3: 0,
            k4: 0,
            k5: 0,
            key_center: Color::default(),
            key_scale: Color::default(),
            key_width: Color::default(),
            lod_fraction: 0,
            memory: Color::default(),
            min_level: 0,
            other_modes: 0,
            past_shift_a: 0,
            past_shift_b: 0,
            past_stored_encoded: 0,
            pixel: Color::default(),
            primitive_color: Color::default(),
            primitive_delta_z: 0,
            primitive_lod_fraction: 0,
            primitive_z: 0,
            scissor_bottom: 0,
            scissor_field: false,
            scissor_keep_odd: false,
            scissor_left: 0,
            scissor_right: 0,
            scissor_top: 0,
            shade: Color::default(),
            shade_correct_dx: [0; 4],
            shade_correct_dy: [0; 4],
            shade_step: [0; 4],
            span_attributes: boxed(0),
            span_drawn: boxed(false),
            span_left: boxed(0),
            span_major_x: boxed(0),
            span_right: boxed(0),
            taken: 0,
            texel0: Color::default(),
            texel1: Color::default(),
            texture_image: 0,
            texture_image_size: 0,
            texture_image_width: 0,
            texture_step: [0; 3],
            tiles: [TextureTile::default(); 8],
            modes: Modes::default(),
        };
        rdp.refresh();
        rdp
    }
}

impl State for Rdp {
    fn write_state(&self, w: &mut StateWriter) {
        w.bytes("<TextureMemory>k__BackingField", &self.texture_memory[..]);
        w.i32s("_attributeDe", &self.attribute_de[..]);
        w.i32s("_attributeDx", &self.attribute_dx[..]);
        w.i32s("_attributeDy", &self.attribute_dy[..]);
        w.i32s("_attributeValue", &self.attribute_value[..]);
        w.structure("_blendColor", &self.blend_color);
        w.i32("_blendShiftA", self.blend_shift_a);
        w.i32("_blendShiftB", self.blend_shift_b);
        w.structure("_blended", &self.blended);
        w.i32("_blenderShadeAlpha", self.blender_shade_alpha);
        w.u32("_colorImage", self.color_image);
        w.i32("_colorImageBytes", self.color_image_bytes);
        w.i32("_colorImageFormat", self.color_image_format);
        w.i32("_colorImageSize", self.color_image_size);
        w.i32("_colorImageWidth", self.color_image_width);
        w.u64("_combine", self.combine);
        w.structure("_combined", &self.combined);
        w.u64s("_command", &self.command[..]);
        w.bytes("_coverage", &self.coverage[..]);
        w.i32("_depthCorrectDx", self.depth_correct_dx);
        w.i32("_depthCorrectDy", self.depth_correct_dy);
        w.u32("_depthImage", self.depth_image);
        w.i32("_depthSlope", self.depth_slope);
        w.i32("_depthStep", self.depth_step);
        w.bools("_edgeInvalid", &self.edge_invalid[..]);
        w.i32s("_edgeLeft", &self.edge_left[..]);
        w.i32s("_edgeRight", &self.edge_right[..]);
        w.structure("_environmentColor", &self.environment_color);
        w.u32("_fillColor", self.fill_color);
        w.structure("_fogColor", &self.fog_color);
        w.i32("_k0", self.k0);
        w.i32("_k1", self.k1);
        w.i32("_k2", self.k2);
        w.i32("_k3", self.k3);
        w.i32("_k4", self.k4);
        w.i32("_k5", self.k5);
        w.structure("_keyCenter", &self.key_center);
        w.structure("_keyScale", &self.key_scale);
        w.structure("_keyWidth", &self.key_width);
        w.i32("_lodFraction", self.lod_fraction);
        w.structure("_memory", &self.memory);
        w.i32("_minLevel", self.min_level);
        w.u64("_otherModes", self.other_modes);
        w.i32("_pastShiftA", self.past_shift_a);
        w.i32("_pastShiftB", self.past_shift_b);
        w.i32("_pastStoredEncoded", self.past_stored_encoded);
        w.structure("_pixel", &self.pixel);
        w.structure("_primitiveColor", &self.primitive_color);
        w.i32("_primitiveDeltaZ", self.primitive_delta_z);
        w.i32("_primitiveLodFraction", self.primitive_lod_fraction);
        w.i32("_primitiveZ", self.primitive_z);
        w.i32("_scissorBottom", self.scissor_bottom);
        w.bool("_scissorField", self.scissor_field);
        w.bool("_scissorKeepOdd", self.scissor_keep_odd);
        w.i32("_scissorLeft", self.scissor_left);
        w.i32("_scissorRight", self.scissor_right);
        w.i32("_scissorTop", self.scissor_top);
        w.structure("_shade", &self.shade);
        w.i32s("_shadeCorrectDx", &self.shade_correct_dx[..]);
        w.i32s("_shadeCorrectDy", &self.shade_correct_dy[..]);
        w.i32s("_shadeStep", &self.shade_step[..]);
        w.i32s("_spanAttributes", &self.span_attributes[..]);
        w.bools("_spanDrawn", &self.span_drawn[..]);
        w.i32s("_spanLeft", &self.span_left[..]);
        w.i32s("_spanMajorX", &self.span_major_x[..]);
        w.i32s("_spanRight", &self.span_right[..]);
        w.i32("_taken", self.taken);
        w.structure("_texel0", &self.texel0);
        w.structure("_texel1", &self.texel1);
        w.u32("_textureImage", self.texture_image);
        w.i32("_textureImageSize", self.texture_image_size);
        w.i32("_textureImageWidth", self.texture_image_width);
        w.i32s("_textureStep", &self.texture_step[..]);
        w.structures("_tiles", &self.tiles);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.bytes(&mut self.texture_memory[..])?; // <TextureMemory>k__BackingField
        r.i32s(&mut self.attribute_de[..])?; // _attributeDe
        r.i32s(&mut self.attribute_dx[..])?; // _attributeDx
        r.i32s(&mut self.attribute_dy[..])?; // _attributeDy
        r.i32s(&mut self.attribute_value[..])?; // _attributeValue
        self.blend_color.read_state(r)?; // _blendColor
        self.blend_shift_a = r.i32()?; // _blendShiftA
        self.blend_shift_b = r.i32()?; // _blendShiftB
        self.blended.read_state(r)?; // _blended
        self.blender_shade_alpha = r.i32()?; // _blenderShadeAlpha
        self.color_image = r.u32()?; // _colorImage
        self.color_image_bytes = r.i32()?; // _colorImageBytes
        self.color_image_format = r.i32()?; // _colorImageFormat
        self.color_image_size = r.i32()?; // _colorImageSize
        self.color_image_width = r.i32()?; // _colorImageWidth
        self.combine = r.u64()?; // _combine
        self.combined.read_state(r)?; // _combined
        r.u64s(&mut self.command[..])?; // _command
        r.bytes(&mut self.coverage[..])?; // _coverage
        self.depth_correct_dx = r.i32()?; // _depthCorrectDx
        self.depth_correct_dy = r.i32()?; // _depthCorrectDy
        self.depth_image = r.u32()?; // _depthImage
        self.depth_slope = r.i32()?; // _depthSlope
        self.depth_step = r.i32()?; // _depthStep
        r.bools(&mut self.edge_invalid[..])?; // _edgeInvalid
        r.i32s(&mut self.edge_left[..])?; // _edgeLeft
        r.i32s(&mut self.edge_right[..])?; // _edgeRight
        self.environment_color.read_state(r)?; // _environmentColor
        self.fill_color = r.u32()?; // _fillColor
        self.fog_color.read_state(r)?; // _fogColor
        self.k0 = r.i32()?; // _k0
        self.k1 = r.i32()?; // _k1
        self.k2 = r.i32()?; // _k2
        self.k3 = r.i32()?; // _k3
        self.k4 = r.i32()?; // _k4
        self.k5 = r.i32()?; // _k5
        self.key_center.read_state(r)?; // _keyCenter
        self.key_scale.read_state(r)?; // _keyScale
        self.key_width.read_state(r)?; // _keyWidth
        self.lod_fraction = r.i32()?; // _lodFraction
        self.memory.read_state(r)?; // _memory
        self.min_level = r.i32()?; // _minLevel
        self.other_modes = r.u64()?; // _otherModes
        self.past_shift_a = r.i32()?; // _pastShiftA
        self.past_shift_b = r.i32()?; // _pastShiftB
        self.past_stored_encoded = r.i32()?; // _pastStoredEncoded
        self.pixel.read_state(r)?; // _pixel
        self.primitive_color.read_state(r)?; // _primitiveColor
        self.primitive_delta_z = r.i32()?; // _primitiveDeltaZ
        self.primitive_lod_fraction = r.i32()?; // _primitiveLodFraction
        self.primitive_z = r.i32()?; // _primitiveZ
        self.scissor_bottom = r.i32()?; // _scissorBottom
        self.scissor_field = r.bool()?; // _scissorField
        self.scissor_keep_odd = r.bool()?; // _scissorKeepOdd
        self.scissor_left = r.i32()?; // _scissorLeft
        self.scissor_right = r.i32()?; // _scissorRight
        self.scissor_top = r.i32()?; // _scissorTop
        self.shade.read_state(r)?; // _shade
        r.i32s(&mut self.shade_correct_dx[..])?; // _shadeCorrectDx
        r.i32s(&mut self.shade_correct_dy[..])?; // _shadeCorrectDy
        r.i32s(&mut self.shade_step[..])?; // _shadeStep
        r.i32s(&mut self.span_attributes[..])?; // _spanAttributes
        r.bools(&mut self.span_drawn[..])?; // _spanDrawn
        r.i32s(&mut self.span_left[..])?; // _spanLeft
        r.i32s(&mut self.span_major_x[..])?; // _spanMajorX
        r.i32s(&mut self.span_right[..])?; // _spanRight
        self.taken = r.i32()?; // _taken
        self.texel0.read_state(r)?; // _texel0
        self.texel1.read_state(r)?; // _texel1
        self.texture_image = r.u32()?; // _textureImage
        self.texture_image_size = r.i32()?; // _textureImageSize
        self.texture_image_width = r.i32()?; // _textureImageWidth
        r.i32s(&mut self.texture_step[..])?; // _textureStep
        r.structures(&mut self.tiles)?; // _tiles
        self.refresh();
        Ok(())
    }
}

/// RDRAM and its hidden bits by raw pointer, so a drain can hold them while the machine touches other bytes. See Mars_Native.md §5.6.1.
pub struct RdpMemory<'a> {
    rdram: *mut u8,
    rdram_len: usize,
    hidden: *mut u8,
    hidden_len: usize,
    check: Option<(&'a crate::dp_threads::Shared, i64)>,
    _memories: std::marker::PhantomData<&'a mut [u8]>,
}

impl<'a> RdpMemory<'a> {
    pub fn new(rdram: &'a mut [u8], hidden: &'a mut [u8]) -> Self {
        RdpMemory { rdram: rdram.as_mut_ptr(), rdram_len: rdram.len(), hidden: hidden.as_mut_ptr(), hidden_len: hidden.len(), check: None, _memories: std::marker::PhantomData }
    }

    /// # Safety
    /// Both memories must stay allocated for `'a`, and no other thread may touch a byte this view touches unless ordered by the page marks.
    pub(crate) unsafe fn shared(rdram: (*mut u8, usize), hidden: (*mut u8, usize), check: Option<(&'a crate::dp_threads::Shared, i64)>) -> Self {
        RdpMemory { rdram: rdram.0, rdram_len: rdram.1, hidden: hidden.0, hidden_len: hidden.1, check, _memories: std::marker::PhantomData }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.rdram_len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.rdram_len == 0
    }

    #[inline(always)]
    fn touch(&self, at: usize, write: bool) {
        if let Some((shared, word)) = self.check {
            shared.verify(at, word, write);
        }
    }

    #[inline(always)]
    pub fn get(&self, at: usize) -> u8 {
        assert!(at < self.rdram_len);
        self.touch(at, false);
        // SAFETY: in bounds; see `shared`.
        unsafe { self.rdram.add(at).read() }
    }

    #[inline(always)]
    pub fn set(&mut self, at: usize, value: u8) {
        assert!(at < self.rdram_len);
        self.touch(at, true);
        // SAFETY: in bounds; see `shared`.
        unsafe { self.rdram.add(at).write(value) }
    }

    /// The hidden bits beside bytes `2 index` and `2 index + 1`, checked as the second, which every writer of them writes.
    #[inline(always)]
    pub fn get_hidden(&self, index: usize) -> u8 {
        assert!(index < self.hidden_len);
        self.touch(index * 2 + 1, false);
        // SAFETY: in bounds; see `shared`.
        unsafe { self.hidden.add(index).read() }
    }

    #[inline(always)]
    pub fn set_hidden(&mut self, index: usize, value: u8) {
        assert!(index < self.hidden_len);
        self.touch(index * 2 + 1, true);
        // SAFETY: in bounds; see `shared`.
        unsafe { self.hidden.add(index).write(value) }
    }

    #[inline(always)]
    pub fn be32(&self, at: usize) -> u32 {
        u32::from_be_bytes([self.get(at), self.get(at + 1), self.get(at + 2), self.get(at + 3)])
    }
}

impl Rdp {
    /// C#'s `Rdp.Accept`: one command word, gathered until its command is whole and then run; true when it completed a full sync.
    pub fn accept(&mut self, word: u64, memory: &mut RdpMemory) -> bool {
        self.command[self.taken as usize] = word;
        self.taken += 1;

        let id = command_id(self.command[0]);
        if self.taken < command_length(id) {
            return false;
        }

        self.taken = 0;
        self.execute(id, self.command[0], memory)
    }

    fn execute(&mut self, id: u32, word: u64, memory: &mut RdpMemory) -> bool {
        match id {
            0x08..=0x0F => self.triangle(id, memory),
            TEXTURE_RECTANGLE | TEXTURE_RECTANGLE_FLIPPED => self.textured_rectangle(id == TEXTURE_RECTANGLE_FLIPPED, memory),
            SET_TEXTURE_IMAGE => self.set_texture_image(word),
            SET_TILE => self.set_tile(word),
            SET_TILE_SIZE => {
                self.set_tile_size(word);
            }
            LOAD_TILE => self.load(word, LoadKind::Tile, memory),
            LOAD_BLOCK => self.load(word, LoadKind::Block, memory),
            LOAD_PALETTE => self.load(word, LoadKind::Palette, memory),
            SYNC_FULL => return true,
            SET_SCISSOR => self.set_scissor(word),
            SET_OTHER_MODES => {
                self.other_modes = word;
                self.decode_other_modes();
            }
            FILL_RECTANGLE => self.fill(word, memory),
            SET_FILL_COLOR => self.fill_color = word as u32,
            SET_COLOR_IMAGE => self.set_color_image(word),
            _ => {
                self.set_register(id, word);
            }
        }
        false
    }
}
