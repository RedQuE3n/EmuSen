//! A preset built for one device: a pipeline per pass, their outputs, and each frame's run from the game's picture
//! to the screen's. The C# `SlangChain`, whose every rule this reproduces: what a name in a pass means, what a block's
//! members are filled with, how large each pass is, and which readback a picture may be lent.
//!
//! A lookup image's pixels are handed in already decoded, since a picture is the same only if the decoder is. A
//! readback lent to a picture is not written again until it is given back, by `release`, from whatever thread; a
//! chain dropped under a lent readback leaves it to that release. See EmuSen_Serenity.md §7.4, §9.1 and §9.2, and
//! EmuSen_RustPlatform.md §3.3 and §17.

use crate::compiler::Stage;
use crate::preset::{PassSpec, Preset, ScaleType, TextureSpec, Wrap};
use crate::reflection::{self, Block, Fault, Reflection};
use crate::source::{self, Parameter};
use crate::text::parse_int;
use crate::vulkan::{Buffer, Checked, Device, Image, point};
use crate::{Error, format, parameters};
use ash::vk;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// How many readbacks a chain may have out with pictures at once; past that a picture is a copy.
pub const MAX_LENT: usize = 3;

/// Why a chain could not be built, as the C# exception it is.
#[derive(Debug)]
pub enum Failure {
    /// A pass's source could not be read.
    Source(Error),
    /// A stage does not compile: `SlangCompileException`, with its words.
    Compile(String),
    /// A stage's SPIR-V could not be read back.
    Reflection(Fault),
    /// A Vulkan call failed: `InvalidOperationException`, with its words.
    Device(String),
    /// A pass samples a frame further back than can be counted: `OverflowException`.
    Overflow,
}

impl From<String> for Failure {
    fn from(words: String) -> Failure {
        Failure::Device(words)
    }
}

/// Why a frame was not taken.
#[derive(Debug, PartialEq, Eq)]
pub enum Unfed {
    /// Fewer bytes than the frame's size says: `ArgumentOutOfRangeException`.
    Short,
    /// A Vulkan call failed.
    Device(String),
}

impl From<String> for Unfed {
    fn from(words: String) -> Unfed {
        Unfed::Device(words)
    }
}

/// What compiles a stage: its text, which stage, and the name its messages carry. A cache, or the compiler itself.
pub type Compile<'a> = dyn Fn(&str, Stage, &str) -> Result<Vec<u8>, String> + Sync + 'a;

const UNORM: vk::Format = vk::Format::from_raw(format::R8G8B8A8_UNORM);
const SRGB: vk::Format = vk::Format::from_raw(format::R8G8B8A8_SRGB);
const SFLOAT: vk::Format = vk::Format::from_raw(format::R16G16B16A16_SFLOAT);
const READ: vk::ImageLayout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
const BOTH_STAGES: vk::ShaderStageFlags = vk::ShaderStageFlags::from_raw(vk::ShaderStageFlags::VERTEX.as_raw() | vk::ShaderStageFlags::FRAGMENT.as_raw());

/// The projection every pass is given: the unit square onto the whole target.
const MVP: [f32; 16] = [2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0];

/// Two triangles in a strip, each corner its position then its texture coordinate.
const QUAD: [f32; 16] = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0];

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// An image a pass draws into, and the framebuffer over it.
struct Target {
    image: Image,
    framebuffer: vk::Framebuffer,
}

impl Drop for Target {
    fn drop(&mut self) {
        unsafe { self.image.device().vk().destroy_framebuffer(self.framebuffer, None) };
    }
}

struct Pass {
    device: Device,
    spec: PassSpec,
    pass_name: Option<String>,
    reflection: Reflection,
    format: vk::Format,
    render_pass: vk::RenderPass,
    set_layout: vk::DescriptorSetLayout,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    pool: vk::DescriptorPool,
    set: vk::DescriptorSet,
    uniforms: Option<Buffer>,
    output: Option<Target>,
    previous: Option<Target>,
    wants_feedback: bool,
    wants_mipmaps: bool,
    width: u32,
    height: u32,
}

impl Pass {
    /// The name later passes know this one's output by: the preset's alias, or else the source's own name.
    fn alias(&self) -> Option<&str> {
        self.spec.alias.as_deref().or(self.pass_name.as_deref())
    }

    fn create_render_pass(&mut self) -> Result<(), String> {
        let attachment = [vk::AttachmentDescription::default()
            .format(self.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(READ)];
        let reference = [vk::AttachmentReference { attachment: 0, layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL }];
        let subpass = [vk::SubpassDescription::default().pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS).color_attachments(&reference)];
        // The pass's writes and final transition made visible to a later pass's shaders (EmuSen_Serenity.md §10).
        let sampled = [vk::SubpassDependency::default()
            .src_subpass(0)
            .dst_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags::VERTEX_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)];
        let info = vk::RenderPassCreateInfo::default().attachments(&attachment).subpasses(&subpass).dependencies(&sampled);
        self.render_pass = unsafe { self.device.vk().create_render_pass(&info, None) }.check("vkCreateRenderPass")?;
        Ok(())
    }

    fn module(&self, spirv: &[u8]) -> Result<vk::ShaderModule, String> {
        // The words are copied to where they are aligned; the size is the module's own, as the C# hands it.
        let mut words = vec![0u32; spirv.len().div_ceil(4)];
        for (word, bytes) in words.iter_mut().zip(spirv.chunks(4)) {
            let mut four = [0u8; 4];
            four[..bytes.len()].copy_from_slice(bytes);
            *word = u32::from_ne_bytes(four);
        }
        let info = vk::ShaderModuleCreateInfo { code_size: spirv.len(), p_code: words.as_ptr(), ..Default::default() };
        unsafe { self.device.vk().create_shader_module(&info, None) }.check("vkCreateShaderModule")
    }

    fn create_pipeline(&mut self, vertex_code: &[u8], fragment_code: &[u8]) -> Result<(), String> {
        let device = self.device.clone();
        let device = device.vk();
        let binding = |binding: u32, kind: vk::DescriptorType| vk::DescriptorSetLayoutBinding::default().binding(binding).descriptor_type(kind).descriptor_count(1).stage_flags(BOTH_STAGES);
        let mut bindings = Vec::new();
        if let Some(uniforms) = &self.reflection.uniforms {
            bindings.push(binding(uniforms.binding, vk::DescriptorType::UNIFORM_BUFFER));
        }
        for sampler in &self.reflection.samplers {
            bindings.push(binding(sampler.binding, vk::DescriptorType::COMBINED_IMAGE_SAMPLER));
        }
        let set_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        self.set_layout = unsafe { device.create_descriptor_set_layout(&set_info, None) }.check("vkCreateDescriptorSetLayout")?;

        let push_size = self.reflection.push_constants.as_ref().map_or(0, |push| push.size);
        let range = [vk::PushConstantRange { stage_flags: BOTH_STAGES, offset: 0, size: push_size }];
        let set_layouts = [self.set_layout];
        let mut layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        if push_size > 0 {
            layout_info = layout_info.push_constant_ranges(&range);
        }
        self.layout = unsafe { device.create_pipeline_layout(&layout_info, None) }.check("vkCreatePipelineLayout")?;

        let vertex = self.module(vertex_code)?;
        let fragment = match self.module(fragment_code) {
            Ok(fragment) => fragment,
            Err(words) => {
                unsafe { device.destroy_shader_module(vertex, None) };
                return Err(words);
            }
        };
        let stages = [
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::VERTEX).module(vertex).name(c"main"),
            vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::FRAGMENT).module(fragment).name(c"main"),
        ];
        let vertex_binding = [vk::VertexInputBindingDescription { binding: 0, stride: 16, input_rate: vk::VertexInputRate::VERTEX }];
        let attributes = [
            vk::VertexInputAttributeDescription { location: 0, binding: 0, format: vk::Format::R32G32_SFLOAT, offset: 0 },
            vk::VertexInputAttributeDescription { location: 1, binding: 0, format: vk::Format::R32G32_SFLOAT, offset: 8 },
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default().vertex_binding_descriptions(&vertex_binding).vertex_attribute_descriptions(&attributes);
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default().topology(vk::PrimitiveTopology::TRIANGLE_STRIP);
        let viewport = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
        let raster = vk::PipelineRasterizationStateCreateInfo::default().polygon_mode(vk::PolygonMode::FILL).cull_mode(vk::CullModeFlags::NONE).front_face(vk::FrontFace::COUNTER_CLOCKWISE).line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default().rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let blend_attachment = [vk::PipelineColorBlendAttachmentState::default().blend_enable(false).color_write_mask(vk::ColorComponentFlags::RGBA)];
        let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachment);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let info = [vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&assembly)
            .viewport_state(&viewport)
            .rasterization_state(&raster)
            .multisample_state(&multisample)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(self.layout)
            .render_pass(self.render_pass)
            .subpass(0)];
        let made = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &info, None) };
        unsafe {
            device.destroy_shader_module(vertex, None);
            device.destroy_shader_module(fragment, None);
        }
        match made {
            Ok(pipelines) => self.pipeline = pipelines[0],
            Err((pipelines, result)) => {
                self.pipeline = pipelines.first().copied().unwrap_or_default();
                return Err(format!("vkCreateGraphicsPipelines returned {}", crate::vulkan::result_name(result)));
            }
        }

        let sizes = [
            vk::DescriptorPoolSize { ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER, descriptor_count: self.reflection.samplers.len().max(1) as u32 },
            vk::DescriptorPoolSize { ty: vk::DescriptorType::UNIFORM_BUFFER, descriptor_count: 1 },
        ];
        let pool_info = vk::DescriptorPoolCreateInfo::default().max_sets(1).pool_sizes(&sizes);
        self.pool = unsafe { device.create_descriptor_pool(&pool_info, None) }.check("vkCreateDescriptorPool")?;
        let allocate = vk::DescriptorSetAllocateInfo::default().descriptor_pool(self.pool).set_layouts(&set_layouts);
        self.set = unsafe { device.allocate_descriptor_sets(&allocate) }.check("vkAllocateDescriptorSets")?[0];
        Ok(())
    }

    fn drop_outputs(&mut self) {
        self.output = None;
        self.previous = None;
    }
}

impl Drop for Pass {
    fn drop(&mut self) {
        self.drop_outputs();
        self.uniforms = None;
        let device = self.device.vk();
        // A handle that was never made is null, which each of these takes as nothing to do.
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
            device.destroy_descriptor_pool(self.pool, None);
            device.destroy_descriptor_set_layout(self.set_layout, None);
            device.destroy_render_pass(self.render_pass, None);
        }
    }
}

/// A pass and the parameters its source declares, or why it could not be built.
type Built = Result<(Pass, Vec<Parameter>), Failure>;

/// One pass built: its source read, its stages compiled and read back, its pipeline and its uniforms made.
fn build(device: &Device, spec: &PassSpec, index: usize, count: usize, compile: &Compile<'_>) -> Built {
    let source = source::load(&spec.shader_path).map_err(Failure::Source)?;
    let vertex = compile(&source.vertex, Stage::Vertex, &spec.shader_path).map_err(Failure::Compile)?;
    let fragment = compile(&source.fragment, Stage::Fragment, &spec.shader_path).map_err(Failure::Compile)?;
    let vertex_binds = reflection::read(&vertex).map_err(Failure::Reflection)?;
    let reflection = reflection::merge(&vertex_binds, &reflection::read(&fragment).map_err(Failure::Reflection)?);
    let fragment = reflection::without_unread_inputs(&fragment).map_err(Failure::Reflection)?.unwrap_or(fragment);

    // The last pass is what the screen shows, so it is 8-bit whatever it asks, sRGB if it says so, as RetroArch's swapchain is.
    let plain = if spec.srgb_framebuffer {
        SRGB
    } else if spec.float_framebuffer {
        SFLOAT
    } else {
        UNORM
    };
    let format = if index == count - 1 {
        if spec.srgb_framebuffer { SRGB } else { UNORM }
    } else {
        source.framebuffer_format.as_deref().and_then(format::parse).map_or(plain, vk::Format::from_raw)
    };

    // Made before its objects are, so that a failure part way frees what was made.
    let mut pass = Pass {
        device: device.clone(),
        spec: spec.clone(),
        pass_name: source.pass_name,
        reflection,
        format,
        render_pass: vk::RenderPass::null(),
        set_layout: vk::DescriptorSetLayout::null(),
        layout: vk::PipelineLayout::null(),
        pipeline: vk::Pipeline::null(),
        pool: vk::DescriptorPool::null(),
        set: vk::DescriptorSet::null(),
        uniforms: None,
        output: None,
        previous: None,
        wants_feedback: false,
        wants_mipmaps: false,
        width: 0,
        height: 0,
    };
    pass.create_render_pass()?;
    pass.create_pipeline(&vertex, &fragment)?;
    if let Some(uniforms) = &pass.reflection.uniforms {
        pass.uniforms = Some(device.create_buffer(uniforms.size as u64, vk::BufferUsageFlags::UNIFORM_BUFFER)?);
    }
    Ok((pass, source.parameters))
}

/// Every pass built, side by side when asked. A serial build stops at the first failure; a parallel one builds
/// every pass and gives the lowest-numbered pass's failure, as a serial build would. The passes built are freed.
fn build_passes(device: &Device, preset: &Preset, compile: &Compile<'_>, builders: usize) -> Result<Vec<(Pass, Vec<Parameter>)>, Failure> {
    let count = preset.passes.len();
    let one = |index: usize| build(device, &preset.passes[index], index, count, compile);
    let mut built = Vec::with_capacity(count);
    if builders <= 1 || count <= 1 {
        for index in 0..count {
            built.push(one(index)?);
        }
        return Ok(built);
    }
    let next = AtomicUsize::new(0);
    let results: Vec<Mutex<Option<Built>>> = (0..count).map(|_| Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..builders.min(count) {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    if index >= count {
                        break;
                    }
                    let result = one(index);
                    *locked(&results[index]) = Some(result);
                }
            });
        }
    });
    for result in results {
        built.push(result.into_inner().unwrap_or_else(PoisonError::into_inner).expect("every pass was built")?);
    }
    Ok(built)
}

/// An image cleared to nothing, every level ready to sample.
fn cleared(device: &Device, width: u32, height: u32, format: vk::Format, mipmapped: bool) -> Result<Image, String> {
    let image = device.create_image(width, height, format, mipmapped)?;
    device.run(|commands| {
        device.transition(commands, &image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, 0, None, None);
        let range = [vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR, base_mip_level: 0, level_count: image.levels, base_array_layer: 0, layer_count: 1 }];
        unsafe { device.vk().cmd_clear_color_image(commands, image.handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &vk::ClearColorValue { float32: [0.0; 4] }, &range) };
        device.transition(commands, &image, READ, 0, None, None);
    })?;
    Ok(image)
}

fn target(device: &Device, render_pass: vk::RenderPass, width: u32, height: u32, format: vk::Format, mipmapped: bool) -> Result<Target, String> {
    let image = cleared(device, width, height, format, mipmapped)?;
    let attachments = [image.target];
    let info = vk::FramebufferCreateInfo::default().render_pass(render_pass).attachments(&attachments).width(image.width).height(image.height).layers(1);
    let framebuffer = unsafe { device.vk().create_framebuffer(&info, None) }.check("vkCreateFramebuffer")?;
    Ok(Target { image, framebuffer })
}

/// One axis of a pass's size: the source's or the viewport's by the scale, or the scale itself, rounded to the
/// nearest whole with a half going to the even one, and never less than one. A scale that is no number gives one.
fn axis(kind: ScaleType, scale: f32, source: u32, view: u32) -> u32 {
    let size = match kind {
        ScaleType::Source => source as f32 * scale,
        ScaleType::Viewport => view as f32 * scale,
        ScaleType::Absolute => scale,
    };
    ((size as f64).round_ties_even() as u32).max(1)
}

/// The number after a prefix, for a name that is the prefix and a number.
fn numbered(name: &str, prefix: &str) -> Option<i32> {
    name.strip_prefix(prefix).filter(|rest| !rest.is_empty()).and_then(parse_int)
}

fn floats(slot: &mut [u8], values: &[f32]) {
    for (bytes, value) in slot.as_chunks_mut::<4>().0.iter_mut().zip(values) {
        *bytes = value.to_ne_bytes();
    }
}

fn word(slot: &mut [u8], value: u32) {
    if let Some(bytes) = slot.get_mut(..4) {
        bytes.copy_from_slice(&value.to_ne_bytes());
    }
}

/// What a name in a pass means, and how that pass samples it.
struct Texture<'a> {
    image: &'a Image,
    linear: bool,
    wrap: Wrap,
    mipmap: bool,
}

struct Lut {
    spec: TextureSpec,
    image: Option<Image>,
}

/// One readback and whether a picture holds it.
struct Slot {
    id: u64,
    state: Mutex<SlotState>,
}

#[derive(Default)]
struct SlotState {
    held: bool,
    retired: bool,
    buffer: Option<Buffer>,
}

impl Slot {
    /// No longer lent; freed now, or when the picture holding it lets go.
    fn retire(&self) {
        let mut state = locked(&self.state);
        state.retired = true;
        if !state.held {
            state.buffer = None;
        }
    }
}

static NEXT_SLOT: AtomicU64 = AtomicU64::new(1);

/// Every readback a picture holds, by the number its release names it by.
static LENT: Mutex<BTreeMap<u64, Arc<Slot>>> = Mutex::new(BTreeMap::new());

/// Gives a lent readback back, from any thread: its chain may write it again, or, if its chain let go of it
/// meanwhile, it is freed. False for a number that names no lent readback, which a second release of one is.
pub fn release(slot: u64) -> bool {
    let Some(slot) = locked(&LENT).remove(&slot) else { return false };
    let mut state = locked(&slot.state);
    state.held = false;
    if state.retired {
        state.buffer = None;
    }
    true
}

/// A picture over a readback: where its pixels are, and what gives the readback back. `slot` is zero for a picture
/// in the chain's own scratch readback, which nothing holds and the chain's next render writes again.
#[derive(Clone, Copy, Debug)]
pub struct Lent {
    pub pixels: *const u8,
    pub bytes: usize,
    pub width: i32,
    pub height: i32,
    pub row_bytes: i32,
    pub slot: u64,
}

pub struct Chain {
    device: Device,
    passes: Vec<Pass>,
    textures: Vec<TextureSpec>,
    luts: HashMap<String, Lut>,
    parameters: Vec<Parameter>,
    values: HashMap<String, f32>,
    history: Vec<Option<Image>>,
    blank: Image,
    vertices: Buffer,
    staging: Option<Buffer>,
    rows: Option<Image>,
    head: usize,
    original_width: u32,
    original_height: u32,
    upload_rows: u32,
    upload_repeat: u32,
    frame_count: u32,
    readbacks: Vec<Arc<Slot>>,
    scratch: Option<Buffer>,
    copied_images: i32,
}

impl Chain {
    /// The preset's passes built on the device, each stage through `compile`, `builders` of them at a time. Its
    /// lookup images are not loaded: each is handed to `load_lut`, decoded, and a pass that samples one before then
    /// reads nothing.
    pub fn new(device: &Device, preset: &Preset, compile: &Compile<'_>, builders: usize) -> Result<Chain, Failure> {
        if preset.passes.is_empty() {
            return Err(Failure::Device("A preset has at least one pass.".to_string()));
        }
        let (mut passes, declared): (Vec<Pass>, Vec<Vec<Parameter>>) = build_passes(device, preset, compile, builders)?.into_iter().unzip();
        let parameters = parameters::merge(declared.iter().map(Vec::as_slice), &preset.parameters);

        // History is as deep as the deepest OriginalHistory# any pass samples, plus the frame itself.
        let mut depth = 0;
        for sampler in passes.iter().flat_map(|pass| &pass.reflection.samplers) {
            if let Some(back) = sampler.name.strip_prefix("OriginalHistory").and_then(parse_int) {
                depth = depth.max(back);
            }
        }
        let depth = depth.checked_add(1).ok_or(Failure::Overflow)? as usize;

        let feedback: Vec<bool> = (0..passes.len())
            .map(|i| {
                let (numbered, alias) = (format!("PassFeedback{i}"), passes[i].alias());
                passes.iter().flat_map(|pass| &pass.reflection.samplers).any(|s| s.name == numbered || alias.is_some_and(|alias| s.name.strip_suffix("Feedback") == Some(alias)))
            })
            .collect();
        for i in 0..passes.len() {
            passes[i].wants_feedback = feedback[i];
            passes[i].wants_mipmaps = i + 1 < passes.len() && passes[i + 1].spec.mipmap_input;
        }

        let blank = device.create_image(1, 1, UNORM, false)?;
        device.upload(&blank, &[0; 4], &device.create_buffer(16, vk::BufferUsageFlags::TRANSFER_SRC)?)?;
        let vertices = device.create_buffer(64, vk::BufferUsageFlags::VERTEX_BUFFER)?;
        vertices.with_bytes(64, |bytes| floats(bytes, &QUAD));

        let mut chain = Chain {
            device: device.clone(),
            passes,
            textures: preset.textures.clone(),
            luts: preset.textures.iter().map(|texture| (texture.name.clone(), Lut { spec: texture.clone(), image: None })).collect(),
            parameters,
            values: HashMap::new(),
            history: (0..depth).map(|_| None).collect(),
            blank,
            vertices,
            staging: None,
            rows: None,
            head: 0,
            original_width: 0,
            original_height: 0,
            upload_rows: 0,
            upload_repeat: 0,
            frame_count: 0,
            readbacks: Vec::new(),
            scratch: None,
            copied_images: 0,
        };
        chain.set_parameters(|_| None);
        Ok(chain)
    }

    /// Every parameter the passes declare, with the preset's value as its default where it gives one.
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    /// A value for each parameter by its id, its default where `value` has none; read by the next render.
    pub fn set_parameters(&mut self, value: impl Fn(&str) -> Option<f32>) {
        for parameter in &self.parameters {
            let chosen = value(&parameter.id).unwrap_or(parameter.initial);
            match self.values.get_mut(&parameter.id) {
                Some(kept) => *kept = chosen,
                None => drop(self.values.insert(parameter.id.clone(), chosen)),
            }
        }
    }

    /// The same, the values given in the parameters' own order; false when they are not one for each.
    pub fn set_values(&mut self, values: &[f32]) -> bool {
        if values.len() != self.parameters.len() {
            return false;
        }
        for (parameter, value) in self.parameters.iter().zip(values) {
            match self.values.get_mut(&parameter.id) {
                Some(kept) => *kept = *value,
                None => drop(self.values.insert(parameter.id.clone(), *value)),
            }
        }
        true
    }

    pub fn value_of(&self, id: &str) -> Option<f32> {
        self.values.get(id).copied()
    }

    /// How many pictures were made by copying because every readback the chain may lend was held.
    pub fn copied_images(&self) -> i32 {
        self.copied_images
    }

    pub fn readback_count(&self) -> usize {
        self.readbacks.len()
    }

    /// The preset's lookup image at `index`, from its pixels in RGBA8, a row after a row.
    pub fn load_lut(&mut self, index: usize, width: i32, height: i32, rgba: &[u8]) -> Result<(), String> {
        let Some(spec) = self.textures.get(index).cloned() else { return Err("The preset has no such lookup image.".to_string()) };
        let image = self.device.create_image(width as u32, height as u32, UNORM, spec.mipmap)?;
        let staging = self.device.create_buffer(width.wrapping_mul(height).wrapping_mul(4) as i64 as u64, vk::BufferUsageFlags::TRANSFER_SRC)?;
        self.device.upload(&image, rgba, &staging)?;
        self.luts.insert(spec.name.clone(), Lut { spec, image: Some(image) });
        Ok(())
    }

    /// A new frame joins the history with its repeated rows, since a slang pass sees the picture as the screen
    /// would; the rows are repeated on the device (EmuSen_Serenity.md §9.2).
    pub fn advance(&mut self, rgba: &[u8], width: i32, height: i32, row_repeat: i32) -> Result<(), Unfed> {
        self.device.phase(point::ADVANCE_BEGIN);
        let repeat = row_repeat.max(1) as u32;
        let (w, rows) = (width as u32, height as u32);
        let h = rows.wrapping_mul(repeat);
        let bytes = w.wrapping_mul(rows).wrapping_mul(4);
        if w != self.original_width || h != self.original_height || rows != self.upload_rows || repeat != self.upload_repeat {
            self.device.wait_idle();
            for i in 0..self.history.len() {
                self.history[i] = None;
                self.history[i] = Some(cleared(&self.device, w, h, UNORM, self.passes[0].spec.mipmap_input)?);
            }
            self.staging = None;
            self.staging = Some(self.device.create_buffer(bytes as u64, vk::BufferUsageFlags::TRANSFER_SRC)?);
            self.rows = None;
            self.rows = if repeat > 1 { Some(self.device.create_image(w, rows, UNORM, false)?) } else { None };
            (self.original_width, self.original_height, self.upload_rows, self.upload_repeat) = (w, h, rows, repeat);
        }
        self.head = (self.head + self.history.len() - 1) % self.history.len();
        let picture = usize::try_from(bytes as i32).ok().and_then(|length| rgba.get(..length)).ok_or(Unfed::Short)?;
        let (Some(image), Some(staging)) = (&self.history[self.head], &self.staging) else { return Err(Unfed::Device("The chain has no frame to write.".to_string())) };
        match &self.rows {
            None => self.device.upload(image, picture, staging)?,
            Some(rows) => self.device.upload_repeated(image, picture, rows, repeat, staging)?,
        }
        self.frame_count = self.frame_count.wrapping_add(1);
        self.device.phase(point::ADVANCE_END);
        Ok(())
    }

    fn original_at(&self, back: usize) -> &Image {
        self.history[(self.head + back.min(self.history.len() - 1)) % self.history.len()].as_ref().unwrap_or(&self.blank)
    }

    /// Runs every pass for the viewport's size and gives the last one's pixels, RGBA8, a row after a row, where
    /// they stay until the chain's next render.
    pub fn render(&mut self, view_width: i32, view_height: i32) -> Result<&[u8], String> {
        let mut scratch = self.scratch.take();
        let rendered = self.render_into(&mut scratch, view_width, view_height);
        self.scratch = scratch;
        let bytes = rendered? as usize;
        let scratch = self.scratch.as_ref().expect("a render leaves its readback");
        // SAFETY: the readback holds at least `bytes`, and nothing writes it while the chain is borrowed.
        Ok(unsafe { std::slice::from_raw_parts(scratch.mapped(), bytes) })
    }

    /// Runs every pass and lends the readback itself, no copy: it is not written again until `release` is given
    /// its number. When every readback the chain may lend is held, the picture is in the scratch readback instead,
    /// for the caller to copy (EmuSen_Serenity.md §9.1).
    pub fn render_lent(&mut self, view_width: i32, view_height: i32) -> Result<Lent, String> {
        let (width, height) = (view_width.max(1), view_height.max(1));
        let bytes = width as u64 * height as u64 * 4;
        let lent = |pixels: *const u8, slot: u64| Lent { pixels, bytes: bytes as usize, width, height, row_bytes: width.wrapping_mul(4), slot };
        let Some(slot) = self.lendable(bytes) else {
            let mut scratch = self.scratch.take();
            let rendered = self.render_into(&mut scratch, view_width, view_height);
            self.scratch = scratch;
            rendered?;
            self.copied_images = self.copied_images.wrapping_add(1);
            return Ok(lent(self.scratch.as_ref().map_or(std::ptr::null(), |scratch| scratch.mapped().cast_const()), 0));
        };
        let mut state = locked(&slot.state);
        self.render_into(&mut state.buffer, view_width, view_height)?;
        state.held = true;
        let pixels = state.buffer.as_ref().map_or(std::ptr::null(), |buffer| buffer.mapped().cast_const());
        drop(state);
        locked(&LENT).insert(slot.id, slot.clone());
        Ok(lent(pixels, slot.id))
    }

    /// A readback no picture holds and large enough, one made if none is, or none when every one the chain may lend is held.
    fn lendable(&mut self, bytes: u64) -> Option<Arc<Slot>> {
        let mut found = None;
        for i in (0..self.readbacks.len()).rev() {
            let state = locked(&self.readbacks[i].state);
            if state.held {
                continue;
            }
            if state.buffer.as_ref().is_some_and(|buffer| buffer.size() >= bytes) {
                if found.is_none() {
                    found = Some(self.readbacks[i].clone());
                }
                continue;
            }
            drop(state);
            self.readbacks.remove(i).retire();
        }
        if found.is_some() || self.readbacks.len() >= MAX_LENT {
            return found;
        }
        let slot = Arc::new(Slot { id: NEXT_SLOT.fetch_add(1, Ordering::Relaxed), state: Mutex::default() });
        self.readbacks.push(slot.clone());
        Some(slot)
    }

    fn resize(&mut self, index: usize, width: u32, height: u32) -> Result<(), String> {
        let pass = &mut self.passes[index];
        if pass.output.is_some() && pass.width == width && pass.height == height {
            return Ok(());
        }
        pass.drop_outputs();
        (pass.width, pass.height) = (width, height);
        pass.output = Some(target(&self.device, pass.render_pass, width, height, pass.format, pass.wants_mipmaps)?);
        if pass.wants_feedback {
            pass.previous = Some(target(&self.device, pass.render_pass, width, height, pass.format, pass.wants_mipmaps)?);
        }
        Ok(())
    }

    /// The passes run for the viewport's size and the last one's pixels left in the readback, visible to the host.
    fn render_into(&mut self, readback: &mut Option<Buffer>, view_width: i32, view_height: i32) -> Result<u64, String> {
        if self.original_width == 0 {
            return Err("No frame has been given to the chain yet.".to_string());
        }
        self.device.phase(point::RENDER_BEGIN);
        let (vw, vh) = (view_width.max(1) as u32, view_height.max(1) as u32);
        let count = self.passes.len();

        let (mut sw, mut sh) = (self.original_width, self.original_height);
        for i in 0..count {
            let spec = &self.passes[i].spec;
            let (w, h) = if i == count - 1 { (vw, vh) } else { (axis(spec.scale_type_x, spec.scale_x, sw, vw), axis(spec.scale_type_y, spec.scale_y, sh, vh)) };
            self.resize(i, w, h)?;
            (sw, sh) = (w, h);
        }

        let bytes = vw as u64 * vh as u64 * 4;
        if !readback.as_ref().is_some_and(|buffer| buffer.size() >= bytes) {
            *readback = None;
            *readback = Some(self.device.create_buffer(bytes, vk::BufferUsageFlags::TRANSFER_DST)?);
        }
        let Some(readback) = readback.as_ref() else { unreachable!("the readback was made above") };

        for i in 0..count {
            self.bind(i, vw, vh)?;
        }
        self.device.phase(point::BOUND);

        let device = &self.device;
        device.run(|commands| {
            for i in 0..count {
                self.draw(commands, i);
                device.mark(commands, i as i32);
            }
            let Some(last) = &self.passes[count - 1].output else { return };
            device.transition(commands, &last.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, 0, None, None);
            let region = [vk::BufferImageCopy::default()
                .image_subresource(vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: 0, base_array_layer: 0, layer_count: 1 })
                .image_extent(vk::Extent3D { width: vw, height: vh, depth: 1 })];
            unsafe { device.vk().cmd_copy_image_to_buffer(commands, last.image.handle, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, readback.handle, &region) };
            device.host_read_barrier(commands, readback);
            device.transition(commands, &last.image, READ, 0, None, None);
        })?;

        self.device.phase(point::RAN);
        for pass in &mut self.passes {
            if pass.wants_feedback {
                std::mem::swap(&mut pass.output, &mut pass.previous);
            }
        }
        self.device.phase(point::COPIED);
        Ok(bytes)
    }

    /// What a name in a pass means, by libretro's semantics, and how that pass samples it.
    fn resolve(&self, name: &str, index: usize) -> Option<Texture<'_>> {
        let (first, own) = (&self.passes[0].spec, &self.passes[index].spec);
        let count = self.passes.len();
        let output = |n: usize, previous: bool| {
            let reader = if n + 1 < count { &self.passes[n + 1].spec } else { own };
            let target = if previous { &self.passes[n].previous } else { &self.passes[n].output };
            Texture { image: target.as_ref().map_or(&self.blank, |target| &target.image), linear: reader.filter_linear, wrap: reader.wrap, mipmap: reader.mipmap_input }
        };
        let pass_numbered = |prefix: &str, below: usize| numbered(name, prefix).and_then(|n| usize::try_from(n).ok()).filter(|n| *n < below);

        if name == "Original" {
            return Some(Texture { image: self.original_at(0), linear: first.filter_linear, wrap: first.wrap, mipmap: first.mipmap_input });
        }
        if name == "Source" {
            let image = if index == 0 { self.original_at(0) } else { output(index - 1, false).image };
            return Some(Texture { image, linear: own.filter_linear, wrap: own.wrap, mipmap: own.mipmap_input });
        }
        if let Some(back) = numbered(name, "OriginalHistory") {
            return Some(Texture { image: self.original_at(usize::try_from(back).unwrap_or(0)), linear: first.filter_linear, wrap: first.wrap, mipmap: false });
        }
        if let Some(n) = pass_numbered("PassOutput", index) {
            return Some(output(n, false));
        }
        if let Some(n) = pass_numbered("PassFeedback", count) {
            return Some(output(n, true));
        }
        if let Some(Lut { spec, image: Some(image) }) = self.luts.get(name) {
            return Some(Texture { image, linear: spec.linear, wrap: spec.wrap, mipmap: spec.mipmap });
        }
        for (n, pass) in self.passes.iter().enumerate() {
            let Some(alias) = pass.alias() else { continue };
            if name == alias && n < index {
                return Some(output(n, false));
            }
            if name.strip_suffix("Feedback") == Some(alias) {
                return Some(output(n, true));
            }
        }
        None
    }

    /// A size by libretro's convention: width, height, and their reciprocals.
    fn size_of(&self, texture: &str, index: usize, vw: u32, vh: u32) -> Option<[f32; 4]> {
        let of = |width: u32, height: u32| {
            let (width, height) = (width as f32, height as f32);
            Some([width, height, 1.0 / width, 1.0 / height])
        };
        let pass = |n: usize| of(self.passes[n].width, self.passes[n].height);
        let pass_numbered = |prefix: &str| numbered(texture, prefix).and_then(|n| usize::try_from(n).ok()).filter(|n| *n < self.passes.len());
        if texture == "Output" {
            return pass(index);
        }
        if texture == "FinalViewport" {
            return of(vw, vh);
        }
        if numbered(texture, "OriginalHistory").is_some() || texture == "Original" {
            return of(self.original_width, self.original_height);
        }
        if let Some(n) = pass_numbered("PassOutput") {
            return pass(n);
        }
        if let Some(n) = pass_numbered("PassFeedback") {
            return pass(n);
        }
        if texture == "Source" {
            return if index == 0 { of(self.original_width, self.original_height) } else { pass(index - 1) };
        }
        if let Some(Lut { image: Some(image), .. }) = self.luts.get(texture) {
            return of(image.width, image.height);
        }
        self.passes.iter().find(|pass| pass.alias().is_some_and(|alias| texture == alias || texture.strip_suffix("Feedback") == Some(alias))).and_then(|pass| of(pass.width, pass.height))
    }

    /// Every member a block declares, by name: the frontend's semantics, a parameter's value, or zero.
    fn fill(&self, block: &Block, into: &mut [u8], index: usize, vw: u32, vh: u32) {
        into.fill(0);
        for member in &block.members {
            let offset = member.offset as usize;
            if offset >= into.len() {
                continue;
            }
            let length = (member.size as usize).min(into.len() - offset);
            let slot = &mut into[offset..offset + length];
            match member.name.as_str() {
                "MVP" => floats(slot, &MVP),
                "FrameCount" => {
                    let modulus = self.passes[index].spec.frame_count_mod;
                    word(slot, if modulus > 0 { self.frame_count % modulus as u32 } else { self.frame_count });
                }
                "FrameDirection" | "TotalSubFrames" | "CurrentSubFrame" => word(slot, 1),
                "Rotation" => word(slot, 0),
                "OriginalAspect" | "OriginalAspectRotated" => floats(slot, &[self.original_width as f32 / self.original_height.max(1) as f32]),
                "OriginalFPS" => floats(slot, &[60.0]),
                name => {
                    if let Some(size) = name.strip_suffix("Size").and_then(|texture| self.size_of(texture, index, vw, vh)) {
                        floats(slot, &size);
                    } else if let Some(value) = self.values.get(name) {
                        floats(slot, &[*value]);
                    }
                }
            }
        }
    }

    fn bind(&self, index: usize, vw: u32, vh: u32) -> Result<(), String> {
        let pass = &self.passes[index];
        let mut buffer_info = [vk::DescriptorBufferInfo::default()];
        if let (Some(block), Some(buffer)) = (&pass.reflection.uniforms, &pass.uniforms) {
            buffer.with_bytes(block.size as usize, |into| self.fill(block, into, index, vw, vh));
            buffer_info[0] = vk::DescriptorBufferInfo { buffer: buffer.handle, offset: 0, range: block.size as u64 };
        }

        let samplers = &pass.reflection.samplers;
        let mut images = Vec::with_capacity(samplers.len());
        for sampler in samplers {
            let texture = self.resolve(&sampler.name, index).unwrap_or(Texture { image: &self.blank, linear: false, wrap: Wrap::ClampToEdge, mipmap: false });
            images.push(vk::DescriptorImageInfo {
                sampler: self.device.sampler_for(texture.linear, texture.wrap, texture.mipmap && texture.image.levels > 1)?,
                image_view: texture.image.view,
                image_layout: READ,
            });
        }

        let mut writes = Vec::with_capacity(images.len() + 1);
        for (sampler, image) in samplers.iter().zip(&images) {
            writes.push(vk::WriteDescriptorSet::default().dst_set(pass.set).dst_binding(sampler.binding).descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER).image_info(std::slice::from_ref(image)));
        }
        if let (Some(block), Some(_)) = (&pass.reflection.uniforms, &pass.uniforms) {
            writes.push(vk::WriteDescriptorSet::default().dst_set(pass.set).dst_binding(block.binding).descriptor_type(vk::DescriptorType::UNIFORM_BUFFER).buffer_info(&buffer_info));
        }
        unsafe { self.device.vk().update_descriptor_sets(&writes, &[]) };
        Ok(())
    }

    fn draw(&self, commands: vk::CommandBuffer, index: usize) {
        let pass = &self.passes[index];
        let Some(output) = &pass.output else { return };
        let device = self.device.vk();
        let area = vk::Rect2D { offset: vk::Offset2D::default(), extent: vk::Extent2D { width: pass.width, height: pass.height } };
        let clear = [vk::ClearValue { color: vk::ClearColorValue { float32: [0.0; 4] } }];
        let begin = vk::RenderPassBeginInfo::default().render_pass(pass.render_pass).framebuffer(output.framebuffer).render_area(area).clear_values(&clear);
        unsafe {
            device.cmd_begin_render_pass(commands, &begin, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(commands, vk::PipelineBindPoint::GRAPHICS, pass.pipeline);
            device.cmd_bind_descriptor_sets(commands, vk::PipelineBindPoint::GRAPHICS, pass.layout, 0, &[pass.set], &[]);
            if let Some(push) = &pass.reflection.push_constants {
                let last = &self.passes[self.passes.len() - 1];
                let mut bytes = vec![0u8; push.size as usize];
                self.fill(push, &mut bytes, index, last.width, last.height);
                device.cmd_push_constants(commands, pass.layout, BOTH_STAGES, 0, &bytes);
            }
            let viewport = [vk::Viewport { x: 0.0, y: 0.0, width: pass.width as f32, height: pass.height as f32, min_depth: 0.0, max_depth: 1.0 }];
            device.cmd_set_viewport(commands, 0, &viewport);
            device.cmd_set_scissor(commands, 0, &[area]);
            device.cmd_bind_vertex_buffers(commands, 0, &[self.vertices.handle], &[0]);
            device.cmd_draw(commands, 4, 1, 0, 0);
            device.cmd_end_render_pass(commands);
        }
        output.image.layout.set(READ);
        if pass.wants_mipmaps {
            self.device.generate_mipmaps(commands, &output.image);
        }
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        self.device.wait_idle();
        for slot in self.readbacks.drain(..) {
            slot.retire();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each size is the C#'s for the same scale and source, measured 2026-10-09: a product in single precision,
    /// rounded half to even, a number past a 32-bit count held at its end, and what is no number counted as nothing.
    #[test]
    fn an_axis_is_rounded_as_the_csharp_rounds_it() {
        let sources = [0u32, 1, 2, 3, 5, 7, 15, 255, 256, 320, 1920, 4294967295, 16777217];
        for (scale, sizes, alone) in [
            (0xFFC00000u32, [0u32; 13], 0u32),
            (0x7F800000, [0, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295], 4294967295),
            (0xFF800000, [0; 13], 0),
            (0xBFC00000, [0; 13], 0),
            (0x3F000000, [0, 0, 1, 2, 2, 4, 8, 128, 128, 160, 960, 2147483648, 8388608], 0),
            (0x3FC00000, [0, 2, 3, 4, 8, 10, 22, 382, 384, 480, 2880, 4294967295, 25165824], 2),
            (0x40200000, [0, 2, 5, 8, 12, 18, 38, 638, 640, 800, 4800, 4294967295, 41943040], 2),
            (0x40600000, [0, 4, 7, 10, 18, 24, 52, 892, 896, 1120, 6720, 4294967295, 58720256], 4),
            (0x3EFFFFFF, [0, 0, 1, 1, 2, 3, 7, 127, 128, 160, 960, 2147483520, 8388608], 0),
            (0x4F7FFFFF, [0, 4294967040, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295], 4294967040),
            (0x3DCCCCCD, [0, 0, 0, 0, 0, 1, 2, 26, 26, 32, 192, 429496736, 1677722], 0),
            (0x3E99999A, [0, 0, 1, 1, 2, 2, 4, 76, 77, 96, 576, 1288490240, 5033165], 0),
            (0x3EAAAAAB, [0, 0, 1, 1, 2, 2, 5, 85, 85, 107, 640, 1431655808, 5592406], 0),
            (0x3F2AAAAB, [0, 1, 1, 2, 3, 5, 10, 170, 171, 213, 1280, 2863311616, 11184811], 1),
            (0x40F00000, [0, 8, 15, 22, 38, 52, 112, 1912, 1920, 2400, 14400, 4294967295, 125829120], 8),
            (0x3F800001, [0, 1, 2, 3, 5, 7, 15, 255, 256, 320, 1920, 4294967295, 16777218], 1),
        ] {
            let scale = f32::from_bits(scale);
            for (source, size) in sources.iter().zip(sizes) {
                // The C# takes the greater of the size and one, which the measured sizes are before.
                assert_eq!(axis(ScaleType::Source, scale, *source, 9), size.max(1), "{scale} of {source}");
                assert_eq!(axis(ScaleType::Viewport, scale, 9, *source), size.max(1), "{scale} of {source}");
            }
            assert_eq!(axis(ScaleType::Absolute, scale, 9, 9), alone.max(1), "{scale}");
        }
    }

    #[test]
    fn a_name_is_numbered_only_by_a_number_after_its_prefix() {
        assert_eq!(numbered("OriginalHistory3", "OriginalHistory"), Some(3));
        assert_eq!(numbered("PassOutput12", "PassOutput"), Some(12));
        assert_eq!(numbered("PassOutput", "PassOutput"), None);
        assert_eq!(numbered("PassOutputs1", "PassOutput"), None);
        assert_eq!(numbered("PassFeedback2147483648", "PassFeedback"), None);
        assert_eq!(numbered("Original", "OriginalHistory"), None);
    }

    #[test]
    fn a_member_is_written_only_as_far_as_its_room_goes() {
        let mut slot = [0xAAu8; 10];
        floats(&mut slot, &[1.0, 2.0, 3.0]);
        assert_eq!(slot[..4], 1.0f32.to_ne_bytes());
        assert_eq!(slot[4..8], 2.0f32.to_ne_bytes());
        assert_eq!(slot[8..], [0xAA, 0xAA]);
        let mut small = [0xAAu8; 3];
        word(&mut small, 7);
        floats(&mut small, &[1.0]);
        assert_eq!(small, [0xAA; 3]);
        let mut exact = [0u8; 4];
        word(&mut exact, 0x01020304);
        assert_eq!(exact, 0x01020304u32.to_ne_bytes());
    }

    #[test]
    fn a_readback_given_back_twice_is_given_back_once() {
        let slot = Arc::new(Slot { id: NEXT_SLOT.fetch_add(1, Ordering::Relaxed), state: Mutex::default() });
        locked(&slot.state).held = true;
        locked(&LENT).insert(slot.id, slot.clone());
        slot.retire();
        assert!(locked(&slot.state).held && locked(&slot.state).retired);
        assert!(release(slot.id));
        assert!(!locked(&slot.state).held);
        assert!(!release(slot.id));
        assert!(!release(0));
    }
}
