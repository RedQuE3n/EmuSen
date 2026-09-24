using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Silk.NET.Core.Native;
using Silk.NET.Vulkan;
using SkiaSharp;

namespace EmuSen.Serenity.Slang
{
    // A preset built for one device: a pipeline per pass, their outputs, and each frame's run from the game's picture to the screen's - see EmuSen_Serenity.md §7.4.
    public sealed unsafe class SlangChain : IDisposable
    {
        private sealed class Pass
        {
            public required SlangPassSpec Spec;
            public required SlangSource Source;
            public required SpirvReflection Reflection;
            public required Format Format;
            public RenderPass RenderPass;
            public DescriptorSetLayout SetLayout;
            public PipelineLayout Layout;
            public Pipeline Pipeline;
            public DescriptorPool Pool;
            public DescriptorSet Set;
            public SlangBuffer? Uniforms;
            public SlangImage? Output, Previous;
            public Framebuffer OutputFramebuffer, PreviousFramebuffer;
            public bool WantsFeedback, WantsMipmaps;
            public uint Width, Height;
        }

        private readonly SlangVulkan _gpu;
        private readonly Vk _vk;
        private readonly Pass[] _passes;
        private readonly Dictionary<string, SlangImage> _luts = new(StringComparer.Ordinal);
        private readonly Dictionary<string, SlangTextureSpec> _lutSpecs = new(StringComparer.Ordinal);
        private readonly Dictionary<string, float> _parameters = new(StringComparer.Ordinal);
        private readonly SlangImage?[] _history;
        private readonly SlangImage _blank;
        private readonly SlangBuffer _vertices;
        private SlangBuffer? _staging;
        private SlangImage? _rows;
        private int _head;
        private uint _originalWidth, _originalHeight, _uploadRows, _uploadRepeat;
        private uint _frameCount;
        private bool _disposed;

        // Readbacks an image may be lent, and one never lent for a copy; a lent one is written again only once its image lets go - see EmuSen_Serenity.md §9.1.
        private readonly List<ReadbackSlot> _readbacks = new();
        private ReadbackSlot? _scratch;
        internal const int MaxLent = 3;

        // Images made by copying because every lendable readback was still held, for a test.
        internal int CopiedImages { get; private set; }

        internal int ReadbackCount => _readbacks.Count;

        public SlangPreset Preset { get; }

        // Every parameter the passes declare, with the preset's value as its default where it gives one - see EmuSen_Serenity.md §7.6.
        public IReadOnlyList<SlangParameter> Parameters { get; }

        // Passes compiled on every processor but one, leaving one for the emulation thread - see EmuSen_Serenity.md §9.5.
        public static int DefaultBuilders => Math.Max(1, Environment.ProcessorCount - 1);

        public SlangChain(SlangVulkan gpu, SlangPreset preset, SpirvCache? cache = null) : this(gpu, preset, cache, DefaultBuilders) { }

        internal SlangChain(SlangVulkan gpu, SlangPreset preset, SpirvCache? cache, int builders)
        {
            _gpu = gpu;
            _vk = gpu.Vk;
            Preset = preset;

            _passes = new Pass[preset.Passes.Count];
            BuildPasses(preset, cache, builders);
            Parameters = SlangParameters.Merge(_passes.Select(p => p.Source), preset.Parameters);
            SetParameters(null);

            // History is as deep as the deepest OriginalHistory# any pass samples, plus the frame itself.
            int depth = 0;
            foreach (Pass pass in _passes)
                foreach (SlangSampler sampler in pass.Reflection.Samplers)
                    if (sampler.Name.StartsWith("OriginalHistory", StringComparison.Ordinal) && int.TryParse(sampler.Name["OriginalHistory".Length..], out int n)) depth = Math.Max(depth, n);
            _history = new SlangImage[depth + 1];

            for (int i = 0; i < _passes.Length; i++)
            {
                string? alias = _passes[i].Spec.Alias ?? _passes[i].Source.PassName;
                _passes[i].WantsFeedback = _passes.Any(p => p.Reflection.Samplers.Any(s => s.Name == $"PassFeedback{i}" || (alias is not null && s.Name == alias + "Feedback")));
                _passes[i].WantsMipmaps = i + 1 < _passes.Length && _passes[i + 1].Spec.MipmapInput;
            }

            foreach (SlangTextureSpec lut in preset.Textures)
            {
                _lutSpecs[lut.Name] = lut;
                _luts[lut.Name] = LoadLut(lut);
            }

            _blank = gpu.CreateImage(1, 1, Format.R8G8B8A8Unorm, false);
            using (SlangBuffer zero = gpu.CreateBuffer(16, BufferUsageFlags.TransferSrcBit)) gpu.Upload(_blank, new byte[4], zero);

            _vertices = gpu.CreateBuffer(64, BufferUsageFlags.VertexBufferBit);
            float[] quad = { 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1 };
            fixed (float* q = quad) System.Buffer.MemoryCopy(q, _vertices.Mapped, 64, 64);
        }

        // Values by id over the defaults, read by the next Render; an id no pass declares is ignored, one left out is its default - see EmuSen_Serenity.md §7.6.
        public void SetParameters(IReadOnlyDictionary<string, float>? values)
        {
            foreach (SlangParameter parameter in Parameters)
                _parameters[parameter.Id] = values is not null && values.TryGetValue(parameter.Id, out float value) ? value : parameter.Initial;
        }

        public float ValueOf(string id) => _parameters.TryGetValue(id, out float value) ? value : float.NaN;

        public static Format ParseFormat(string? name, Format fallback)
        {
            if (string.IsNullOrEmpty(name)) return fallback;
            string[] parts = name.Split('_');
            string spelled = parts[0] + string.Concat(parts.Skip(1).Select(p => p.Length == 0 ? p : char.ToUpperInvariant(p[0]) + p[1..].ToLowerInvariant()));
            return Enum.TryParse(spelled, out Format format) ? format : fallback;
        }

        // Every pass built, in parallel when asked; on any failure the passes built so far are freed and the lowest-numbered pass's error is thrown, as a serial build would.
        private void BuildPasses(SlangPreset preset, SpirvCache? cache, int builders)
        {
            int count = _passes.Length;
            var failures = new Exception?[count];
            void One(int i)
            {
                try { Build(preset.Passes[i], i, count, cache); }
                catch (Exception e) { failures[i] = e; }
            }
            if (builders <= 1 || count <= 1) { for (int i = 0; i < count && failures.All(f => f is null); i++) One(i); }
            else System.Threading.Tasks.Parallel.For(0, count, new System.Threading.Tasks.ParallelOptions { MaxDegreeOfParallelism = builders }, One);

            if (failures.FirstOrDefault(f => f is not null) is not { } first) return;
            foreach (Pass? pass in _passes) if (pass is not null) FreePass(pass);
            System.Runtime.ExceptionServices.ExceptionDispatchInfo.Capture(first).Throw();
        }

        private void Build(SlangPassSpec spec, int index, int count, SpirvCache? cache)
        {
            SlangSource source = SlangSource.Load(spec.ShaderPath);
            byte[] vertex = cache?.Compile(source.Vertex, SlangStage.Vertex, spec.ShaderPath) ?? SlangCompiler.Compile(source.Vertex, SlangStage.Vertex, spec.ShaderPath);
            byte[] fragment = cache?.Compile(source.Fragment, SlangStage.Fragment, spec.ShaderPath) ?? SlangCompiler.Compile(source.Fragment, SlangStage.Fragment, spec.ShaderPath);
            SpirvReflection reflection = SpirvReflection.Merge(SpirvReflection.Read(vertex), SpirvReflection.Read(fragment));

            // The last pass is what the screen shows, so it is 8-bit whatever it asks, sRGB if it says so, as RetroArch's swapchain is.
            bool last = index == count - 1;
            Format plain = spec.SrgbFramebuffer ? Format.R8G8B8A8Srgb : spec.FloatFramebuffer ? Format.R16G16B16A16Sfloat : Format.R8G8B8A8Unorm;
            Format format = last ? (spec.SrgbFramebuffer ? Format.R8G8B8A8Srgb : Format.R8G8B8A8Unorm) : ParseFormat(source.FramebufferFormat, plain);

            // Registered before its objects are made, so a failure part way frees what was made.
            var pass = new Pass { Spec = spec, Source = source, Reflection = reflection, Format = format };
            _passes[index] = pass;
            pass.RenderPass = CreateRenderPass(format);
            CreatePipeline(pass, vertex, fragment);
            if (reflection.Uniforms is { } ubo) pass.Uniforms = _gpu.CreateBuffer(ubo.Size, BufferUsageFlags.UniformBufferBit);
        }

        private void FreePass(Pass pass)
        {
            DropOutputs(pass);
            pass.Uniforms?.Dispose();
            _vk.DestroyPipeline(_gpu.Device, pass.Pipeline, null);
            _vk.DestroyPipelineLayout(_gpu.Device, pass.Layout, null);
            _vk.DestroyDescriptorPool(_gpu.Device, pass.Pool, null);
            _vk.DestroyDescriptorSetLayout(_gpu.Device, pass.SetLayout, null);
            _vk.DestroyRenderPass(_gpu.Device, pass.RenderPass, null);
        }

        private RenderPass CreateRenderPass(Format format)
        {
            var attachment = new AttachmentDescription
            {
                Format = format, Samples = SampleCountFlags.Count1Bit, LoadOp = AttachmentLoadOp.Clear, StoreOp = AttachmentStoreOp.Store,
                StencilLoadOp = AttachmentLoadOp.DontCare, StencilStoreOp = AttachmentStoreOp.DontCare,
                InitialLayout = ImageLayout.Undefined, FinalLayout = ImageLayout.ShaderReadOnlyOptimal,
            };
            var reference = new AttachmentReference { Attachment = 0, Layout = ImageLayout.ColorAttachmentOptimal };
            var subpass = new SubpassDescription { PipelineBindPoint = PipelineBindPoint.Graphics, ColorAttachmentCount = 1, PColorAttachments = &reference };
            var info = new RenderPassCreateInfo { SType = StructureType.RenderPassCreateInfo, AttachmentCount = 1, PAttachments = &attachment, SubpassCount = 1, PSubpasses = &subpass };
            SlangVulkan.Check(_vk.CreateRenderPass(_gpu.Device, &info, null, out RenderPass renderPass), "vkCreateRenderPass");
            return renderPass;
        }

        private ShaderModule Module(byte[] spirv)
        {
            fixed (byte* code = spirv)
            {
                var info = new ShaderModuleCreateInfo { SType = StructureType.ShaderModuleCreateInfo, CodeSize = (nuint)spirv.Length, PCode = (uint*)code };
                SlangVulkan.Check(_vk.CreateShaderModule(_gpu.Device, &info, null, out ShaderModule module), "vkCreateShaderModule");
                return module;
            }
        }

        private void CreatePipeline(Pass pass, byte[] vertexCode, byte[] fragmentCode)
        {
            var bindings = new List<DescriptorSetLayoutBinding>();
            if (pass.Reflection.Uniforms is { } ubo)
                bindings.Add(new DescriptorSetLayoutBinding { Binding = ubo.Binding, DescriptorType = DescriptorType.UniformBuffer, DescriptorCount = 1, StageFlags = ShaderStageFlags.VertexBit | ShaderStageFlags.FragmentBit });
            foreach (SlangSampler sampler in pass.Reflection.Samplers)
                bindings.Add(new DescriptorSetLayoutBinding { Binding = sampler.Binding, DescriptorType = DescriptorType.CombinedImageSampler, DescriptorCount = 1, StageFlags = ShaderStageFlags.VertexBit | ShaderStageFlags.FragmentBit });

            DescriptorSetLayoutBinding[] array = bindings.ToArray();
            fixed (DescriptorSetLayoutBinding* b = array)
            {
                var setInfo = new DescriptorSetLayoutCreateInfo { SType = StructureType.DescriptorSetLayoutCreateInfo, BindingCount = (uint)array.Length, PBindings = b };
                SlangVulkan.Check(_vk.CreateDescriptorSetLayout(_gpu.Device, &setInfo, null, out pass.SetLayout), "vkCreateDescriptorSetLayout");
            }

            uint pushSize = pass.Reflection.PushConstants?.Size ?? 0;
            var range = new PushConstantRange { StageFlags = ShaderStageFlags.VertexBit | ShaderStageFlags.FragmentBit, Offset = 0, Size = pushSize };
            DescriptorSetLayout setLayout = pass.SetLayout;
            var layoutInfo = new PipelineLayoutCreateInfo
            {
                SType = StructureType.PipelineLayoutCreateInfo, SetLayoutCount = 1, PSetLayouts = &setLayout,
                PushConstantRangeCount = pushSize > 0 ? 1u : 0u, PPushConstantRanges = pushSize > 0 ? &range : null,
            };
            SlangVulkan.Check(_vk.CreatePipelineLayout(_gpu.Device, &layoutInfo, null, out pass.Layout), "vkCreatePipelineLayout");

            ShaderModule vertex = Module(vertexCode), fragment = Module(fragmentCode);
            nint main = SilkMarshal.StringToPtr("main");
            try
            {
                var stages = stackalloc PipelineShaderStageCreateInfo[2];
                stages[0] = new PipelineShaderStageCreateInfo { SType = StructureType.PipelineShaderStageCreateInfo, Stage = ShaderStageFlags.VertexBit, Module = vertex, PName = (byte*)main };
                stages[1] = new PipelineShaderStageCreateInfo { SType = StructureType.PipelineShaderStageCreateInfo, Stage = ShaderStageFlags.FragmentBit, Module = fragment, PName = (byte*)main };

                var binding = new VertexInputBindingDescription { Binding = 0, Stride = 16, InputRate = VertexInputRate.Vertex };
                var attributes = stackalloc VertexInputAttributeDescription[2];
                attributes[0] = new VertexInputAttributeDescription { Location = 0, Binding = 0, Format = Format.R32G32Sfloat, Offset = 0 };
                attributes[1] = new VertexInputAttributeDescription { Location = 1, Binding = 0, Format = Format.R32G32Sfloat, Offset = 8 };
                var vertexInput = new PipelineVertexInputStateCreateInfo
                {
                    SType = StructureType.PipelineVertexInputStateCreateInfo, VertexBindingDescriptionCount = 1, PVertexBindingDescriptions = &binding,
                    VertexAttributeDescriptionCount = 2, PVertexAttributeDescriptions = attributes,
                };
                var assembly = new PipelineInputAssemblyStateCreateInfo { SType = StructureType.PipelineInputAssemblyStateCreateInfo, Topology = PrimitiveTopology.TriangleStrip };
                var viewport = new PipelineViewportStateCreateInfo { SType = StructureType.PipelineViewportStateCreateInfo, ViewportCount = 1, ScissorCount = 1 };
                var raster = new PipelineRasterizationStateCreateInfo
                {
                    SType = StructureType.PipelineRasterizationStateCreateInfo, PolygonMode = PolygonMode.Fill, CullMode = CullModeFlags.None,
                    FrontFace = FrontFace.CounterClockwise, LineWidth = 1f,
                };
                var multisample = new PipelineMultisampleStateCreateInfo { SType = StructureType.PipelineMultisampleStateCreateInfo, RasterizationSamples = SampleCountFlags.Count1Bit };
                var blendAttachment = new PipelineColorBlendAttachmentState
                {
                    BlendEnable = false,
                    ColorWriteMask = ColorComponentFlags.RBit | ColorComponentFlags.GBit | ColorComponentFlags.BBit | ColorComponentFlags.ABit,
                };
                var blend = new PipelineColorBlendStateCreateInfo { SType = StructureType.PipelineColorBlendStateCreateInfo, AttachmentCount = 1, PAttachments = &blendAttachment };
                var dynamicStates = stackalloc DynamicState[2] { DynamicState.Viewport, DynamicState.Scissor };
                var dynamic = new PipelineDynamicStateCreateInfo { SType = StructureType.PipelineDynamicStateCreateInfo, DynamicStateCount = 2, PDynamicStates = dynamicStates };

                var info = new GraphicsPipelineCreateInfo
                {
                    SType = StructureType.GraphicsPipelineCreateInfo, StageCount = 2, PStages = stages,
                    PVertexInputState = &vertexInput, PInputAssemblyState = &assembly, PViewportState = &viewport, PRasterizationState = &raster,
                    PMultisampleState = &multisample, PColorBlendState = &blend, PDynamicState = &dynamic,
                    Layout = pass.Layout, RenderPass = pass.RenderPass, Subpass = 0,
                };
                SlangVulkan.Check(_vk.CreateGraphicsPipelines(_gpu.Device, default, 1, &info, null, out pass.Pipeline), "vkCreateGraphicsPipelines");
            }
            finally
            {
                SilkMarshal.Free(main);
                _vk.DestroyShaderModule(_gpu.Device, vertex, null);
                _vk.DestroyShaderModule(_gpu.Device, fragment, null);
            }

            int samplers = Math.Max(1, pass.Reflection.Samplers.Count);
            var sizes = stackalloc DescriptorPoolSize[2];
            sizes[0] = new DescriptorPoolSize { Type = DescriptorType.CombinedImageSampler, DescriptorCount = (uint)samplers };
            sizes[1] = new DescriptorPoolSize { Type = DescriptorType.UniformBuffer, DescriptorCount = 1 };
            var poolInfo = new DescriptorPoolCreateInfo { SType = StructureType.DescriptorPoolCreateInfo, MaxSets = 1, PoolSizeCount = 2, PPoolSizes = sizes };
            SlangVulkan.Check(_vk.CreateDescriptorPool(_gpu.Device, &poolInfo, null, out pass.Pool), "vkCreateDescriptorPool");
            var allocate = new DescriptorSetAllocateInfo { SType = StructureType.DescriptorSetAllocateInfo, DescriptorPool = pass.Pool, DescriptorSetCount = 1, PSetLayouts = &setLayout };
            SlangVulkan.Check(_vk.AllocateDescriptorSets(_gpu.Device, &allocate, out pass.Set), "vkAllocateDescriptorSets");
        }

        private SlangImage LoadLut(SlangTextureSpec lut)
        {
            using SKBitmap? decoded = SKBitmap.Decode(lut.Path);
            if (decoded is null) throw new InvalidDataException($"The lookup image {lut.Path} does not decode.");
            using SKBitmap rgba = new(new SKImageInfo(decoded.Width, decoded.Height, SKColorType.Rgba8888, SKAlphaType.Unpremul));
            using (var canvas = new SKCanvas(rgba)) canvas.DrawBitmap(decoded, 0, 0);
            SlangImage image = _gpu.CreateImage((uint)rgba.Width, (uint)rgba.Height, Format.R8G8B8A8Unorm, lut.Mipmap);
            using SlangBuffer staging = _gpu.CreateBuffer((ulong)(rgba.Width * rgba.Height * 4), BufferUsageFlags.TransferSrcBit);
            _gpu.Upload(image, rgba.GetPixelSpan(), staging);
            return image;
        }

        private (uint W, uint H) SizeOf(int index, uint sourceW, uint sourceH, uint viewW, uint viewH)
        {
            SlangPassSpec spec = _passes[index].Spec;
            static uint Axis(SlangScaleType type, float scale, uint source, uint view) => Math.Max(1u, type switch
            {
                SlangScaleType.Source => (uint)Math.Round(source * scale),
                SlangScaleType.Viewport => (uint)Math.Round(view * scale),
                _ => (uint)Math.Round(scale),
            });
            return (Axis(spec.ScaleTypeX, spec.ScaleX, sourceW, viewW), Axis(spec.ScaleTypeY, spec.ScaleY, sourceH, viewH));
        }

        private Framebuffer FramebufferFor(Pass pass, SlangImage image)
        {
            ImageView view = image.Target;
            var info = new FramebufferCreateInfo
            {
                SType = StructureType.FramebufferCreateInfo, RenderPass = pass.RenderPass, AttachmentCount = 1, PAttachments = &view,
                Width = image.Width, Height = image.Height, Layers = 1,
            };
            SlangVulkan.Check(_vk.CreateFramebuffer(_gpu.Device, &info, null, out Framebuffer framebuffer), "vkCreateFramebuffer");
            return framebuffer;
        }

        private SlangImage Blank(uint w, uint h, Format format, bool mips)
        {
            SlangImage image = _gpu.CreateImage(w, h, format, mips);
            _gpu.Run(commands =>
            {
                _gpu.Transition(commands, image, ImageLayout.TransferDstOptimal);
                var clear = new ClearColorValue(0, 0, 0, 0);
                var range = new ImageSubresourceRange(ImageAspectFlags.ColorBit, 0, image.Levels, 0, 1);
                _vk.CmdClearColorImage(commands, image.Handle, ImageLayout.TransferDstOptimal, &clear, 1, &range);
                _gpu.Transition(commands, image, ImageLayout.ShaderReadOnlyOptimal);
            });
            return image;
        }

        private void Resize(Pass pass, uint w, uint h)
        {
            if (pass.Output is not null && pass.Width == w && pass.Height == h) return;
            DropOutputs(pass);
            pass.Width = w;
            pass.Height = h;
            pass.Output = Blank(w, h, pass.Format, pass.WantsMipmaps);
            pass.OutputFramebuffer = FramebufferFor(pass, pass.Output);
            if (pass.WantsFeedback)
            {
                pass.Previous = Blank(w, h, pass.Format, pass.WantsMipmaps);
                pass.PreviousFramebuffer = FramebufferFor(pass, pass.Previous);
            }
        }

        private void DropOutputs(Pass pass)
        {
            if (pass.Output is not null) { _vk.DestroyFramebuffer(_gpu.Device, pass.OutputFramebuffer, null); pass.Output.Dispose(); pass.Output = null; }
            if (pass.Previous is not null) { _vk.DestroyFramebuffer(_gpu.Device, pass.PreviousFramebuffer, null); pass.Previous.Dispose(); pass.Previous = null; }
        }

        // A new frame joins the history with its repeated rows, since a slang pass sees the picture as the screen would; the rows are repeated on the device - see EmuSen_Serenity.md §9.2.
        public void Advance(ReadOnlySpan<byte> rgba, int width, int height, int rowRepeat)
        {
            SlangProbe.Current?.Phase(SlangProbe.AdvanceBegin);
            uint repeat = (uint)Math.Max(1, rowRepeat);
            uint w = (uint)width, rows = (uint)height, h = rows * repeat;
            if (w != _originalWidth || h != _originalHeight || rows != _uploadRows || repeat != _uploadRepeat)
            {
                _vk.DeviceWaitIdle(_gpu.Device);
                for (int i = 0; i < _history.Length; i++)
                {
                    _history[i]?.Dispose();
                    _history[i] = Blank(w, h, Format.R8G8B8A8Unorm, _passes[0].Spec.MipmapInput);
                }
                _staging?.Dispose();
                _staging = _gpu.CreateBuffer(w * rows * 4, BufferUsageFlags.TransferSrcBit);
                _rows?.Dispose();
                _rows = repeat > 1 ? _gpu.CreateImage(w, rows, Format.R8G8B8A8Unorm, false) : null;
                (_originalWidth, _originalHeight, _uploadRows, _uploadRepeat) = (w, h, rows, repeat);
            }
            _head = (_head + _history.Length - 1) % _history.Length;
            ReadOnlySpan<byte> picture = rgba[..(int)(w * rows * 4)];
            if (_rows is null) _gpu.Upload(_history[_head]!, picture, _staging!);
            else _gpu.UploadRepeated(_history[_head]!, picture, _rows, repeat, _staging!);
            _frameCount++;
            SlangProbe.Current?.Phase(SlangProbe.AdvanceEnd);
        }

        private SlangImage OriginalAt(int back) => _history[(_head + Math.Min(back, _history.Length - 1)) % _history.Length]!;

        // Runs every pass for the viewport's size and returns a copy of the last one's pixels, RGBA8, a row after row.
        public byte[] Render(int viewWidth, int viewHeight)
        {
            _scratch ??= new ReadbackSlot();
            ulong bytes = RenderInto(_scratch, viewWidth, viewHeight);
            var pixels = new byte[bytes];
            fixed (byte* p = pixels) System.Buffer.MemoryCopy(_scratch.Buffer!.Mapped, p, (long)bytes, (long)bytes);
            return pixels;
        }

        // Runs every pass and returns an image over the readback itself, no copy; the readback is not written again while Skia holds the image - see EmuSen_Serenity.md §9.1.
        public SKImage RenderImage(int viewWidth, int viewHeight)
        {
            var info = new SKImageInfo(Math.Max(1, viewWidth), Math.Max(1, viewHeight), SKColorType.Rgba8888, SKAlphaType.Opaque);
            ReadbackSlot? slot = Lendable((ulong)info.BytesSize64);
            if (slot is null)
            {
                _scratch ??= new ReadbackSlot();
                RenderInto(_scratch, viewWidth, viewHeight);
                CopiedImages++;
                return SKImage.FromPixelCopy(info, (nint)_scratch.Buffer!.Mapped, info.RowBytes) ?? throw new InvalidOperationException("Skia refused the chain's picture");
            }
            RenderInto(slot, viewWidth, viewHeight);
            slot.Lend();
            var pixmap = new SKPixmap(info, (nint)slot.Buffer!.Mapped, info.RowBytes);
            SKImage? image = SKImage.FromPixels(pixmap, ReadbackSlot.Released, slot);
            if (image is null) { slot.Return(); throw new InvalidOperationException("Skia refused the chain's picture"); }
            return image;
        }

        // A readback no image holds and large enough, one made if none is, or null when every one the chain may lend is held.
        private ReadbackSlot? Lendable(ulong bytes)
        {
            ReadbackSlot? found = null;
            for (int i = _readbacks.Count - 1; i >= 0; i--)
            {
                ReadbackSlot slot = _readbacks[i];
                if (slot.Held) continue;
                if (slot.Buffer!.Size >= bytes) { found ??= slot; continue; }
                slot.Retire();
                _readbacks.RemoveAt(i);
            }
            if (found is not null || _readbacks.Count >= MaxLent) return found;
            found = new ReadbackSlot();
            _readbacks.Add(found);
            return found;
        }

        // The passes run for the viewport's size and the last one's pixels left in the slot's buffer, visible to the host.
        private ulong RenderInto(ReadbackSlot slot, int viewWidth, int viewHeight)
        {
            if (_originalWidth == 0) throw new InvalidOperationException("No frame has been given to the chain yet.");
            SlangProbe.Current?.Phase(SlangProbe.RenderBegin);
            uint vw = (uint)Math.Max(1, viewWidth), vh = (uint)Math.Max(1, viewHeight);

            uint sw = _originalWidth, sh = _originalHeight;
            for (int i = 0; i < _passes.Length; i++)
            {
                (uint w, uint h) = i == _passes.Length - 1 ? (vw, vh) : SizeOf(i, sw, sh, vw, vh);
                Resize(_passes[i], w, h);
                (sw, sh) = (w, h);
            }

            ulong bytes = (ulong)vw * vh * 4;
            if (slot.Buffer is null || slot.Buffer.Size < bytes)
            {
                slot.Buffer?.Dispose();
                slot.Buffer = _gpu.CreateBuffer(bytes, BufferUsageFlags.TransferDstBit);
            }
            SlangBuffer readback = slot.Buffer;

            for (int i = 0; i < _passes.Length; i++) Bind(i, vw, vh);
            SlangProbe.Current?.Phase(SlangProbe.Bound);

            _gpu.Run(commands =>
            {
                for (int i = 0; i < _passes.Length; i++) { Draw(commands, i); SlangProbe.Current?.Mark(commands, i); }
                SlangImage final = _passes[^1].Output!;
                _gpu.Transition(commands, final, ImageLayout.TransferSrcOptimal);
                var region = new BufferImageCopy { ImageSubresource = new ImageSubresourceLayers(ImageAspectFlags.ColorBit, 0, 0, 1), ImageExtent = new Extent3D(vw, vh, 1) };
                _vk.CmdCopyImageToBuffer(commands, final.Handle, ImageLayout.TransferSrcOptimal, readback.Handle, 1, &region);
                _gpu.HostReadBarrier(commands, readback);
                _gpu.Transition(commands, final, ImageLayout.ShaderReadOnlyOptimal);
            });

            SlangProbe.Current?.Phase(SlangProbe.Ran);
            foreach (Pass pass in _passes)
            {
                if (!pass.WantsFeedback) continue;
                (pass.Output, pass.Previous) = (pass.Previous, pass.Output);
                (pass.OutputFramebuffer, pass.PreviousFramebuffer) = (pass.PreviousFramebuffer, pass.OutputFramebuffer);
            }
            SlangProbe.Current?.Phase(SlangProbe.Copied);
            return bytes;
        }

        private readonly record struct Texture(SlangImage Image, bool Linear, SlangWrap Wrap, bool Mipmap);

        // What a name in a pass means, by libretro's semantics, and how that pass samples it.
        private Texture? Resolve(string name, int index)
        {
            SlangPassSpec first = _passes[0].Spec, own = _passes[index].Spec;
            Texture Output(int n, bool previous)
            {
                SlangPassSpec reader = n + 1 < _passes.Length ? _passes[n + 1].Spec : own;
                SlangImage? image = previous ? _passes[n].Previous : _passes[n].Output;
                return new Texture(image ?? _blank, reader.FilterLinear, reader.Wrap, reader.MipmapInput);
            }

            if (name == "Original") return new Texture(OriginalAt(0), first.FilterLinear, first.Wrap, first.MipmapInput);
            if (name == "Source") return index == 0 ? new Texture(OriginalAt(0), own.FilterLinear, own.Wrap, own.MipmapInput) : Output(index - 1, false) with { Linear = own.FilterLinear, Wrap = own.Wrap, Mipmap = own.MipmapInput };
            if (Numbered(name, "OriginalHistory", out int back)) return new Texture(OriginalAt(back), first.FilterLinear, first.Wrap, false);
            if (Numbered(name, "PassOutput", out int output) && output < index) return Output(output, false);
            if (Numbered(name, "PassFeedback", out int feedback) && feedback < _passes.Length) return Output(feedback, true);
            if (_luts.TryGetValue(name, out SlangImage? lut)) return new Texture(lut, _lutSpecs[name].Linear, _lutSpecs[name].Wrap, _lutSpecs[name].Mipmap);
            for (int n = 0; n < _passes.Length; n++)
            {
                string? alias = _passes[n].Spec.Alias ?? _passes[n].Source.PassName;
                if (alias is null) continue;
                if (name == alias && n < index) return Output(n, false);
                if (name == alias + "Feedback") return Output(n, true);
            }
            return null;
        }

        private static bool Numbered(string name, string prefix, out int number)
        {
            number = 0;
            return name.StartsWith(prefix, StringComparison.Ordinal) && name.Length > prefix.Length && int.TryParse(name[prefix.Length..], out number);
        }

        // A size by libretro's convention: width, height, and their reciprocals.
        private (float, float, float, float)? SizeOf(string texture, int index, uint vw, uint vh)
        {
            (float, float, float, float) Of(float w, float h) => (w, h, 1f / w, 1f / h);
            if (texture == "Output") return Of(_passes[index].Width, _passes[index].Height);
            if (texture == "FinalViewport") return Of(vw, vh);
            if (Numbered(texture, "OriginalHistory", out _) || texture == "Original") return Of(_originalWidth, _originalHeight);
            if (Numbered(texture, "PassOutput", out int o) && o < _passes.Length) return Of(_passes[o].Width, _passes[o].Height);
            if (Numbered(texture, "PassFeedback", out int f) && f < _passes.Length) return Of(_passes[f].Width, _passes[f].Height);
            if (texture == "Source") return index == 0 ? Of(_originalWidth, _originalHeight) : Of(_passes[index - 1].Width, _passes[index - 1].Height);
            if (_luts.TryGetValue(texture, out SlangImage? lut)) return Of(lut.Width, lut.Height);
            for (int n = 0; n < _passes.Length; n++)
            {
                string? alias = _passes[n].Spec.Alias ?? _passes[n].Source.PassName;
                if (alias is not null && (texture == alias || texture == alias + "Feedback")) return Of(_passes[n].Width, _passes[n].Height);
            }
            return null;
        }

        // Every member a block declares, by name: the frontend's semantics, a parameter's value, or zero.
        private void Fill(SlangBlock block, Span<byte> into, int index, uint vw, uint vh)
        {
            into.Clear();
            foreach (SlangMember member in block.Members)
            {
                if (member.Offset >= into.Length) continue;
                Span<byte> slot = into.Slice((int)member.Offset, (int)Math.Min(member.Size, (uint)into.Length - member.Offset));

                switch (member.Name)
                {
                    case "MVP": Floats(slot, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 1, 0, -1, -1, 0, 1); break;
                    case "FrameCount":
                        int mod = _passes[index].Spec.FrameCountMod;
                        BitConverter.TryWriteBytes(slot, mod > 0 ? _frameCount % (uint)mod : _frameCount);
                        break;
                    case "FrameDirection": BitConverter.TryWriteBytes(slot, 1); break;
                    case "Rotation": BitConverter.TryWriteBytes(slot, 0u); break;
                    case "TotalSubFrames": case "CurrentSubFrame": BitConverter.TryWriteBytes(slot, 1u); break;
                    case "OriginalAspect": case "OriginalAspectRotated": Floats(slot, _originalWidth / (float)Math.Max(1, _originalHeight)); break;
                    case "OriginalFPS": Floats(slot, 60f); break;
                    default:
                        if (member.Name.EndsWith("Size", StringComparison.Ordinal) && SizeOf(member.Name[..^4], index, vw, vh) is var (w, h, iw, ih)) Floats(slot, w, h, iw, ih);
                        else if (_parameters.TryGetValue(member.Name, out float value)) Floats(slot, value);
                        break;
                }
            }
        }

        private static void Floats(Span<byte> slot, params ReadOnlySpan<float> values)
        {
            for (int i = 0; i < values.Length && (i + 1) * 4 <= slot.Length; i++) BitConverter.TryWriteBytes(slot[(i * 4)..], values[i]);
        }

        private void Bind(int index, uint vw, uint vh)
        {
            Pass pass = _passes[index];
            var writes = new List<WriteDescriptorSet>();
            var images = new DescriptorImageInfo[pass.Reflection.Samplers.Count];
            var bufferInfo = new DescriptorBufferInfo();

            if (pass.Reflection.Uniforms is { } ubo && pass.Uniforms is { } buffer)
            {
                Fill(ubo, new Span<byte>(buffer.Mapped, (int)ubo.Size), index, vw, vh);
                bufferInfo = new DescriptorBufferInfo { Buffer = buffer.Handle, Offset = 0, Range = ubo.Size };
            }

            for (int s = 0; s < images.Length; s++)
            {
                SlangSampler sampler = pass.Reflection.Samplers[s];
                Texture texture = Resolve(sampler.Name, index) ?? new Texture(_blank, false, SlangWrap.ClampToEdge, false);
                images[s] = new DescriptorImageInfo
                {
                    Sampler = _gpu.SamplerFor(texture.Linear, texture.Wrap, texture.Mipmap && texture.Image.Levels > 1),
                    ImageView = texture.Image.View, ImageLayout = ImageLayout.ShaderReadOnlyOptimal,
                };
            }

            fixed (DescriptorImageInfo* imagePointer = images)
            {
                var all = new WriteDescriptorSet[images.Length + (pass.Uniforms is null ? 0 : 1)];
                for (int s = 0; s < images.Length; s++)
                    all[s] = new WriteDescriptorSet
                    {
                        SType = StructureType.WriteDescriptorSet, DstSet = pass.Set, DstBinding = pass.Reflection.Samplers[s].Binding, DescriptorCount = 1,
                        DescriptorType = DescriptorType.CombinedImageSampler, PImageInfo = imagePointer + s,
                    };
                if (pass.Uniforms is not null)
                    all[^1] = new WriteDescriptorSet
                    {
                        SType = StructureType.WriteDescriptorSet, DstSet = pass.Set, DstBinding = pass.Reflection.Uniforms!.Binding, DescriptorCount = 1,
                        DescriptorType = DescriptorType.UniformBuffer, PBufferInfo = &bufferInfo,
                    };
                fixed (WriteDescriptorSet* w = all) _vk.UpdateDescriptorSets(_gpu.Device, (uint)all.Length, w, 0, null);
            }
        }

        private void Draw(CommandBuffer commands, int index)
        {
            Pass pass = _passes[index];
            var clear = new ClearValue(new ClearColorValue(0, 0, 0, 0));
            var begin = new RenderPassBeginInfo
            {
                SType = StructureType.RenderPassBeginInfo, RenderPass = pass.RenderPass, Framebuffer = pass.OutputFramebuffer,
                RenderArea = new Rect2D(new Offset2D(0, 0), new Extent2D(pass.Width, pass.Height)), ClearValueCount = 1, PClearValues = &clear,
            };
            _vk.CmdBeginRenderPass(commands, &begin, SubpassContents.Inline);
            _vk.CmdBindPipeline(commands, PipelineBindPoint.Graphics, pass.Pipeline);
            DescriptorSet set = pass.Set;
            _vk.CmdBindDescriptorSets(commands, PipelineBindPoint.Graphics, pass.Layout, 0, 1, &set, 0, null);
            if (pass.Reflection.PushConstants is { } push)
            {
                Span<byte> bytes = stackalloc byte[(int)push.Size];
                Fill(push, bytes, index, _passes[^1].Width, _passes[^1].Height);
                fixed (byte* p = bytes) _vk.CmdPushConstants(commands, pass.Layout, ShaderStageFlags.VertexBit | ShaderStageFlags.FragmentBit, 0, push.Size, p);
            }
            var viewport = new Viewport(0, 0, pass.Width, pass.Height, 0, 1);
            var scissor = new Rect2D(new Offset2D(0, 0), new Extent2D(pass.Width, pass.Height));
            _vk.CmdSetViewport(commands, 0, 1, &viewport);
            _vk.CmdSetScissor(commands, 0, 1, &scissor);
            var vertexBuffer = _vertices.Handle;
            ulong offset = 0;
            _vk.CmdBindVertexBuffers(commands, 0, 1, &vertexBuffer, &offset);
            _vk.CmdDraw(commands, 4, 1, 0, 0);
            _vk.CmdEndRenderPass(commands);
            pass.Output!.Layout = ImageLayout.ShaderReadOnlyOptimal;
            if (pass.WantsMipmaps) _gpu.GenerateMipmaps(commands, pass.Output);
        }

        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;
            _vk.DeviceWaitIdle(_gpu.Device);
            foreach (Pass pass in _passes) FreePass(pass);
            foreach (SlangImage? image in _history) image?.Dispose();
            foreach (SlangImage lut in _luts.Values) lut.Dispose();
            _blank.Dispose();
            _vertices.Dispose();
            _staging?.Dispose();
            _rows?.Dispose();
            foreach (ReadbackSlot slot in _readbacks) slot.Retire();
            _readbacks.Clear();
            _scratch?.Retire();
        }
    }

    // One readback buffer and whether an image holds it; Skia's release, on whatever thread, gives it back - see EmuSen_Serenity.md §9.1.
    internal sealed class ReadbackSlot
    {
        private readonly object _lock = new();
        private bool _held, _retired;

        public SlangBuffer? Buffer;

        public bool Held { get { lock (_lock) return _held; } }

        public static readonly SKImageRasterReleaseDelegate Released = (_, context) => ((ReadbackSlot)context).Return();

        public void Lend()
        {
            lock (_lock) _held = true;
        }

        public void Return()
        {
            lock (_lock)
            {
                _held = false;
                if (_retired) Free();
            }
        }

        // No longer lent; freed now, or when the image holding it lets go.
        public void Retire()
        {
            lock (_lock)
            {
                _retired = true;
                if (!_held) Free();
            }
        }

        private void Free()
        {
            Buffer?.Dispose();
            Buffer = null;
        }
    }
}
