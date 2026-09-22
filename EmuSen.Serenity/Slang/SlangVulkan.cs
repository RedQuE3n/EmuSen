using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using Silk.NET.Core.Native;
using Silk.NET.Vulkan;
using VkBuffer = Silk.NET.Vulkan.Buffer;
using VkImage = Silk.NET.Vulkan.Image;

namespace EmuSen.Serenity.Slang
{
    // A buffer and its memory; host buffers stay mapped for their whole life.
    internal sealed unsafe class SlangBuffer(SlangVulkan owner, VkBuffer handle, DeviceMemory memory, ulong size, byte* mapped) : IDisposable
    {
        public VkBuffer Handle { get; } = handle;
        public DeviceMemory Memory { get; } = memory;
        public ulong Size { get; } = size;
        public byte* Mapped { get; } = mapped;
        public void Dispose() => owner.Destroy(this);
    }

    // A 2D image with its mip chain, a view of it all to sample and of level 0 to draw into, and the layout it was last left in.
    internal sealed class SlangImage(SlangVulkan owner, VkImage handle, DeviceMemory memory, ImageView view, ImageView target, Format format, uint width, uint height, uint levels) : IDisposable
    {
        public VkImage Handle { get; } = handle;
        public DeviceMemory Memory { get; } = memory;
        public ImageView View { get; } = view;
        public ImageView Target { get; } = target;
        public Format Format { get; } = format;
        public uint Width { get; } = width;
        public uint Height { get; } = height;
        public uint Levels { get; } = levels;
        public ImageLayout Layout { get; set; } = ImageLayout.Undefined;
        public void Dispose() => owner.Destroy(this);
    }

    // One Vulkan device with a graphics queue, and the few resources a slang chain needs - see EmuSen_Serenity.md §7.4.
    public sealed unsafe class SlangVulkan : IDisposable
    {
        public const string DeviceVariable = "EMUSEN_SLANG_GPU_DEVICE";

        internal Vk Vk { get; }
        internal Device Device { get; }
        private readonly Instance _instance;
        private readonly Queue _queue;
        private readonly CommandPool _pool;
        private readonly CommandBuffer _commands;
        private readonly Fence _fence;
        private readonly PhysicalDeviceMemoryProperties _memory;
        private readonly Dictionary<(bool Linear, SlangWrap Wrap, bool Mipmap), Sampler> _samplers = new();
        private bool _disposed;

        public string Name { get; }

        private SlangVulkan(Vk vk, Instance instance, PhysicalDevice physical, uint family, string name)
        {
            Vk = vk;
            _instance = instance;
            Name = name;

            float priority = 1f;
            var queueInfo = new DeviceQueueCreateInfo { SType = StructureType.DeviceQueueCreateInfo, QueueFamilyIndex = family, QueueCount = 1, PQueuePriorities = &priority };
            var deviceInfo = new DeviceCreateInfo { SType = StructureType.DeviceCreateInfo, QueueCreateInfoCount = 1, PQueueCreateInfos = &queueInfo };
            Check(vk.CreateDevice(physical, &deviceInfo, null, out Device device), "vkCreateDevice");
            Device = device;
            vk.GetDeviceQueue(device, family, 0, out _queue);

            var poolInfo = new CommandPoolCreateInfo { SType = StructureType.CommandPoolCreateInfo, QueueFamilyIndex = family, Flags = CommandPoolCreateFlags.ResetCommandBufferBit };
            Check(vk.CreateCommandPool(device, &poolInfo, null, out _pool), "vkCreateCommandPool");
            var allocate = new CommandBufferAllocateInfo { SType = StructureType.CommandBufferAllocateInfo, CommandPool = _pool, Level = CommandBufferLevel.Primary, CommandBufferCount = 1 };
            Check(vk.AllocateCommandBuffers(device, &allocate, out _commands), "vkAllocateCommandBuffers");
            var fenceInfo = new FenceCreateInfo { SType = StructureType.FenceCreateInfo };
            Check(vk.CreateFence(device, &fenceInfo, null, out _fence), "vkCreateFence");
            vk.GetPhysicalDeviceMemoryProperties(physical, out _memory);
        }

        // Null, with the reason, when no device with a graphics queue is there; the frontend then shows the picture unfiltered.
        public static SlangVulkan? TryCreate(out string report, string? nameContains = null)
        {
            nameContains ??= Environment.GetEnvironmentVariable(DeviceVariable);
            Vk? vk = null;
            Instance instance = default;
            try
            {
                vk = Vk.GetApi();
                var application = new ApplicationInfo { SType = StructureType.ApplicationInfo, ApiVersion = Vk.Version11 };
                var create = new InstanceCreateInfo { SType = StructureType.InstanceCreateInfo, PApplicationInfo = &application };
                Check(vk.CreateInstance(&create, null, &instance), "vkCreateInstance");

                uint count = 0;
                vk.EnumeratePhysicalDevices(instance, &count, null);
                var devices = new PhysicalDevice[count];
                fixed (PhysicalDevice* p = devices) vk.EnumeratePhysicalDevices(instance, &count, p);

                var candidates = new List<(PhysicalDevice Physical, uint Family, string Name, int Rank)>();
                foreach (PhysicalDevice physical in devices)
                {
                    vk.GetPhysicalDeviceProperties(physical, out PhysicalDeviceProperties properties);
                    uint families = 0;
                    vk.GetPhysicalDeviceQueueFamilyProperties(physical, &families, null);
                    var queues = new QueueFamilyProperties[families];
                    fixed (QueueFamilyProperties* q = queues) vk.GetPhysicalDeviceQueueFamilyProperties(physical, &families, q);
                    int family = Array.FindIndex(queues, q => (q.QueueFlags & QueueFlags.GraphicsBit) != 0);
                    if (family < 0) continue;
                    string name = Marshal.PtrToStringAnsi((nint)properties.DeviceName) ?? "unnamed";
                    int rank = properties.DeviceType switch { PhysicalDeviceType.DiscreteGpu => 0, PhysicalDeviceType.IntegratedGpu => 1, PhysicalDeviceType.VirtualGpu => 2, _ => 3 };
                    candidates.Add((physical, (uint)family, name, rank));
                }

                var chosen = candidates.Where(c => string.IsNullOrEmpty(nameContains) || c.Name.Contains(nameContains, StringComparison.OrdinalIgnoreCase))
                    .OrderBy(c => c.Rank).FirstOrDefault();
                if (chosen.Name is null)
                {
                    report = candidates.Count == 0 ? "no Vulkan device offers a graphics queue" : $"no Vulkan device is named like \"{nameContains}\"";
                    vk.DestroyInstance(instance, null);
                    vk.Dispose();
                    return null;
                }

                var device = new SlangVulkan(vk, instance, chosen.Physical, chosen.Family, chosen.Name);
                report = chosen.Name;
                return device;
            }
            catch (Exception e)
            {
                report = $"Vulkan is not available: {e.Message}";
                if (vk is not null)
                {
                    if (instance.Handle != 0) vk.DestroyInstance(instance, null);
                    vk.Dispose();
                }
                return null;
            }
        }

        internal static void Check(Result result, string call)
        {
            if (result != Result.Success) throw new InvalidOperationException($"{call} returned {result}");
        }

        private uint MemoryType(uint allowed, MemoryPropertyFlags need, MemoryPropertyFlags want)
        {
            foreach (MemoryPropertyFlags flags in new[] { need | want, need })
                for (int i = 0; i < _memory.MemoryTypeCount; i++)
                    if ((allowed & (1u << i)) != 0 && (_memory.MemoryTypes[i].PropertyFlags & flags) == flags) return (uint)i;
            throw new InvalidOperationException($"no memory type with {need}");
        }

        internal SlangBuffer CreateBuffer(ulong bytes, BufferUsageFlags usage)
        {
            var info = new BufferCreateInfo { SType = StructureType.BufferCreateInfo, Size = Math.Max(bytes, 16), Usage = usage, SharingMode = SharingMode.Exclusive };
            Check(Vk.CreateBuffer(Device, &info, null, out VkBuffer buffer), "vkCreateBuffer");
            Vk.GetBufferMemoryRequirements(Device, buffer, out MemoryRequirements needs);
            var allocate = new MemoryAllocateInfo
            {
                SType = StructureType.MemoryAllocateInfo, AllocationSize = needs.Size,
                MemoryTypeIndex = MemoryType(needs.MemoryTypeBits, MemoryPropertyFlags.HostVisibleBit | MemoryPropertyFlags.HostCoherentBit, MemoryPropertyFlags.HostCachedBit),
            };
            Check(Vk.AllocateMemory(Device, &allocate, null, out DeviceMemory memory), "vkAllocateMemory");
            Check(Vk.BindBufferMemory(Device, buffer, memory, 0), "vkBindBufferMemory");
            void* mapped = null;
            Check(Vk.MapMemory(Device, memory, 0, Vk.WholeSize, 0, &mapped), "vkMapMemory");
            return new SlangBuffer(this, buffer, memory, info.Size, (byte*)mapped);
        }

        internal void Destroy(SlangBuffer buffer)
        {
            if (_disposed) return;
            Vk.UnmapMemory(Device, buffer.Memory);
            Vk.DestroyBuffer(Device, buffer.Handle, null);
            Vk.FreeMemory(Device, buffer.Memory, null);
        }

        internal static uint LevelsFor(uint width, uint height) => (uint)Math.Floor(Math.Log2(Math.Max(width, height))) + 1;

        internal SlangImage CreateImage(uint width, uint height, Format format, bool mipmapped)
        {
            uint levels = mipmapped ? LevelsFor(width, height) : 1;
            var info = new ImageCreateInfo
            {
                SType = StructureType.ImageCreateInfo, ImageType = ImageType.Type2D, Format = format,
                Extent = new Extent3D(width, height, 1), MipLevels = levels, ArrayLayers = 1, Samples = SampleCountFlags.Count1Bit,
                Tiling = ImageTiling.Optimal, SharingMode = SharingMode.Exclusive, InitialLayout = ImageLayout.Undefined,
                Usage = ImageUsageFlags.ColorAttachmentBit | ImageUsageFlags.SampledBit | ImageUsageFlags.TransferSrcBit | ImageUsageFlags.TransferDstBit,
            };
            Check(Vk.CreateImage(Device, &info, null, out VkImage image), "vkCreateImage");
            Vk.GetImageMemoryRequirements(Device, image, out MemoryRequirements needs);
            var allocate = new MemoryAllocateInfo
            {
                SType = StructureType.MemoryAllocateInfo, AllocationSize = needs.Size,
                MemoryTypeIndex = MemoryType(needs.MemoryTypeBits, MemoryPropertyFlags.DeviceLocalBit, 0),
            };
            Check(Vk.AllocateMemory(Device, &allocate, null, out DeviceMemory memory), "vkAllocateMemory");
            Check(Vk.BindImageMemory(Device, image, memory, 0), "vkBindImageMemory");
            var viewInfo = new ImageViewCreateInfo
            {
                SType = StructureType.ImageViewCreateInfo, Image = image, ViewType = ImageViewType.Type2D, Format = format,
                SubresourceRange = new ImageSubresourceRange(ImageAspectFlags.ColorBit, 0, levels, 0, 1),
            };
            Check(Vk.CreateImageView(Device, &viewInfo, null, out ImageView view), "vkCreateImageView");
            ImageView target = view;
            if (levels > 1)
            {
                viewInfo.SubresourceRange = new ImageSubresourceRange(ImageAspectFlags.ColorBit, 0, 1, 0, 1);
                Check(Vk.CreateImageView(Device, &viewInfo, null, out target), "vkCreateImageView");
            }
            return new SlangImage(this, image, memory, view, target, format, width, height, levels);
        }

        internal void Destroy(SlangImage image)
        {
            if (_disposed) return;
            if (image.Target.Handle != image.View.Handle) Vk.DestroyImageView(Device, image.Target, null);
            Vk.DestroyImageView(Device, image.View, null);
            Vk.DestroyImage(Device, image.Handle, null);
            Vk.FreeMemory(Device, image.Memory, null);
        }

        internal Sampler SamplerFor(bool linear, SlangWrap wrap, bool mipmap)
        {
            if (_samplers.TryGetValue((linear, wrap, mipmap), out Sampler existing)) return existing;
            SamplerAddressMode mode = wrap switch
            {
                SlangWrap.ClampToEdge => SamplerAddressMode.ClampToEdge,
                SlangWrap.Repeat => SamplerAddressMode.Repeat,
                SlangWrap.MirroredRepeat => SamplerAddressMode.MirroredRepeat,
                _ => SamplerAddressMode.ClampToBorder,
            };
            Filter filter = linear ? Filter.Linear : Filter.Nearest;
            var info = new SamplerCreateInfo
            {
                SType = StructureType.SamplerCreateInfo, MagFilter = filter, MinFilter = filter,
                MipmapMode = linear ? SamplerMipmapMode.Linear : SamplerMipmapMode.Nearest,
                AddressModeU = mode, AddressModeV = mode, AddressModeW = mode,
                MinLod = 0, MaxLod = mipmap ? 1000f : 0f, BorderColor = BorderColor.FloatTransparentBlack,
            };
            Check(Vk.CreateSampler(Device, &info, null, out Sampler sampler), "vkCreateSampler");
            _samplers[(linear, wrap, mipmap)] = sampler;
            return sampler;
        }

        // Records, submits and waits; nothing recorded here is still running when it returns.
        internal void Run(Action<CommandBuffer> record)
        {
            Check(Vk.ResetCommandBuffer(_commands, 0), "vkResetCommandBuffer");
            var begin = new CommandBufferBeginInfo { SType = StructureType.CommandBufferBeginInfo, Flags = CommandBufferUsageFlags.OneTimeSubmitBit };
            Check(Vk.BeginCommandBuffer(_commands, &begin), "vkBeginCommandBuffer");
            record(_commands);
            Check(Vk.EndCommandBuffer(_commands), "vkEndCommandBuffer");
            CommandBuffer commands = _commands;
            var submit = new SubmitInfo { SType = StructureType.SubmitInfo, CommandBufferCount = 1, PCommandBuffers = &commands };
            Fence fence = _fence;
            Check(Vk.QueueSubmit(_queue, 1, &submit, fence), "vkQueueSubmit");
            Check(Vk.WaitForFences(Device, 1, &fence, true, ulong.MaxValue), "vkWaitForFences");
            Check(Vk.ResetFences(Device, 1, &fence), "vkResetFences");
        }

        // Every level of an image moved to a layout, with the widest barrier, since a chain's passes run one after another anyway.
        internal void Transition(CommandBuffer commands, SlangImage image, ImageLayout to, uint baseLevel = 0, uint levels = uint.MaxValue, ImageLayout? from = null)
        {
            uint count = levels == uint.MaxValue ? image.Levels - baseLevel : levels;
            var barrier = new ImageMemoryBarrier
            {
                SType = StructureType.ImageMemoryBarrier, OldLayout = from ?? image.Layout, NewLayout = to, Image = image.Handle,
                SrcAccessMask = AccessFlags.MemoryWriteBit, DstAccessMask = AccessFlags.MemoryReadBit | AccessFlags.MemoryWriteBit,
                SrcQueueFamilyIndex = Vk.QueueFamilyIgnored, DstQueueFamilyIndex = Vk.QueueFamilyIgnored,
                SubresourceRange = new ImageSubresourceRange(ImageAspectFlags.ColorBit, baseLevel, count, 0, 1),
            };
            Vk.CmdPipelineBarrier(commands, PipelineStageFlags.AllCommandsBit, PipelineStageFlags.AllCommandsBit, 0, 0, null, 0, null, 1, &barrier);
            if (baseLevel == 0 && count == image.Levels) image.Layout = to;
        }

        // Level 0 blitted down the chain, every level left ready to sample.
        internal void GenerateMipmaps(CommandBuffer commands, SlangImage image)
        {
            if (image.Levels <= 1)
            {
                Transition(commands, image, ImageLayout.ShaderReadOnlyOptimal);
                return;
            }
            Transition(commands, image, ImageLayout.TransferDstOptimal);
            int w = (int)image.Width, h = (int)image.Height;
            for (uint level = 1; level < image.Levels; level++)
            {
                Transition(commands, image, ImageLayout.TransferSrcOptimal, level - 1, 1, ImageLayout.TransferDstOptimal);
                int nw = Math.Max(1, w / 2), nh = Math.Max(1, h / 2);
                var blit = new ImageBlit
                {
                    SrcSubresource = new ImageSubresourceLayers(ImageAspectFlags.ColorBit, level - 1, 0, 1),
                    DstSubresource = new ImageSubresourceLayers(ImageAspectFlags.ColorBit, level, 0, 1),
                };
                blit.SrcOffsets[1] = new Offset3D(w, h, 1);
                blit.DstOffsets[1] = new Offset3D(nw, nh, 1);
                Vk.CmdBlitImage(commands, image.Handle, ImageLayout.TransferSrcOptimal, image.Handle, ImageLayout.TransferDstOptimal, 1, &blit, Filter.Linear);
                Transition(commands, image, ImageLayout.ShaderReadOnlyOptimal, level - 1, 1, ImageLayout.TransferSrcOptimal);
                (w, h) = (nw, nh);
            }
            Transition(commands, image, ImageLayout.ShaderReadOnlyOptimal, image.Levels - 1, 1, ImageLayout.TransferDstOptimal);
            image.Layout = ImageLayout.ShaderReadOnlyOptimal;
        }

        // Pixels in RGBA8 from the host into level 0, and the chain built if the image has one.
        internal void Upload(SlangImage image, ReadOnlySpan<byte> rgba, SlangBuffer staging)
        {
            fixed (byte* source = rgba) System.Buffer.MemoryCopy(source, staging.Mapped, (long)staging.Size, rgba.Length);
            Run(commands =>
            {
                Transition(commands, image, ImageLayout.TransferDstOptimal);
                var region = new BufferImageCopy
                {
                    ImageSubresource = new ImageSubresourceLayers(ImageAspectFlags.ColorBit, 0, 0, 1), ImageExtent = new Extent3D(image.Width, image.Height, 1),
                };
                Vk.CmdCopyBufferToImage(commands, staging.Handle, image.Handle, ImageLayout.TransferDstOptimal, 1, &region);
                if (image.Levels > 1) GenerateMipmaps(commands, image);
                else Transition(commands, image, ImageLayout.ShaderReadOnlyOptimal);
            });
        }

        public void Dispose()
        {
            if (_disposed) return;
            Vk.DeviceWaitIdle(Device);
            foreach (Sampler sampler in _samplers.Values) Vk.DestroySampler(Device, sampler, null);
            _disposed = true;
            Vk.DestroyFence(Device, _fence, null);
            Vk.DestroyCommandPool(Device, _pool, null);
            Vk.DestroyDevice(Device, null);
            Vk.DestroyInstance(_instance, null);
            Vk.Dispose();
        }
    }
}
