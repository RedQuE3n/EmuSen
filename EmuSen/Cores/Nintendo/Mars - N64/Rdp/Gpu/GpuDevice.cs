using System.Runtime.InteropServices;
using Silk.NET.Core.Native;
using Silk.NET.Vulkan;
using VkBuffer = Silk.NET.Vulkan.Buffer;

namespace EmuSen.Cores.Nintendo.Mars.Rdp.Gpu
{
    // Where a buffer's memory lives: the host's, mapped for good, or the device's own, reached only by a copy.
    public enum GpuMemory { Host, Device }

    // One Vulkan compute device and the little of Vulkan the multiple's shading needs - see Mars_Gpu.md §1.
    public sealed unsafe class GpuDevice : IDisposable
    {
        // Names a device by a part of its name, for tests and for a machine with more than one.
        public const string DeviceVariable = "EMUSEN_MARS_GPU_DEVICE";

        private readonly Vk _vk;
        private readonly Instance _instance;
        private readonly PhysicalDevice _physical;
        private readonly Device _device;
        private readonly Queue _queue;
        private readonly CommandPool _pool;
        private readonly CommandBuffer _commands;
        private readonly Fence _fence;
        private readonly PhysicalDeviceMemoryProperties _memory;
        private bool _disposed;

        public string Name { get; }
        public bool IsSoftware { get; }

        // The most one storage buffer may bind, which the memory at a multiple is measured against - see Mars_Gpu.md §5.
        public ulong MaxBufferBytes { get; }

        private GpuDevice(Vk vk, Instance instance, PhysicalDevice physical, string name, bool software, uint family)
        {
            _vk = vk;
            _instance = instance;
            _physical = physical;
            Name = name;
            IsSoftware = software;

            float priority = 1f;
            var queueInfo = new DeviceQueueCreateInfo
            {
                SType = StructureType.DeviceQueueCreateInfo, QueueFamilyIndex = family, QueueCount = 1, PQueuePriorities = &priority,
            };
            var deviceInfo = new DeviceCreateInfo { SType = StructureType.DeviceCreateInfo, QueueCreateInfoCount = 1, PQueueCreateInfos = &queueInfo };

            // A layered implementation must be told its subset is accepted; this is what MoltenVK over Metal asks - see Mars_Gpu.md §4.
            nint subset = SilkMarshal.StringToPtr(PortabilitySubset);
            try
            {
                byte* name0 = (byte*)subset;
                if (DeviceExtensions(vk, physical).Contains(PortabilitySubset))
                {
                    deviceInfo.EnabledExtensionCount = 1;
                    deviceInfo.PpEnabledExtensionNames = &name0;
                }
                Check(_vk.CreateDevice(physical, &deviceInfo, null, out _device), "vkCreateDevice");
            }
            finally { SilkMarshal.Free(subset); }
            _vk.GetDeviceQueue(_device, family, 0, out _queue);

            var poolInfo = new CommandPoolCreateInfo
            {
                SType = StructureType.CommandPoolCreateInfo, QueueFamilyIndex = family, Flags = CommandPoolCreateFlags.ResetCommandBufferBit,
            };
            Check(_vk.CreateCommandPool(_device, &poolInfo, null, out _pool), "vkCreateCommandPool");

            var allocate = new CommandBufferAllocateInfo
            {
                SType = StructureType.CommandBufferAllocateInfo, CommandPool = _pool, Level = CommandBufferLevel.Primary, CommandBufferCount = 1,
            };
            Check(_vk.AllocateCommandBuffers(_device, &allocate, out _commands), "vkAllocateCommandBuffers");

            var fenceInfo = new FenceCreateInfo { SType = StructureType.FenceCreateInfo };
            Check(_vk.CreateFence(_device, &fenceInfo, null, out _fence), "vkCreateFence");

            _vk.GetPhysicalDeviceMemoryProperties(physical, out _memory);
            _vk.GetPhysicalDeviceProperties(physical, out PhysicalDeviceProperties properties);
            MaxBufferBytes = properties.Limits.MaxStorageBufferRange;
        }

        // Every compute-capable device the loader offers, in the order it would be chosen; empty with no loader or no driver.
        public static IReadOnlyList<string> DeviceNames()
        {
            try
            {
                using var probe = new Enumeration();
                return probe.Candidates.Select(c => c.Name).ToArray();
            }
            catch (Exception) { return Array.Empty<string>(); }
        }

        // Null, with the reason, when there is no usable device: the caller's answer to that is the CPU path.
        public static GpuDevice? TryCreate(string? nameContains, out string report)
        {
            nameContains ??= Environment.GetEnvironmentVariable(DeviceVariable);
            Enumeration? probe = null;

            try
            {
                probe = new Enumeration();
                Candidate? chosen = probe.Candidates
                    .Where(c => string.IsNullOrEmpty(nameContains) || c.Name.Contains(nameContains, StringComparison.OrdinalIgnoreCase))
                    .Cast<Candidate?>().FirstOrDefault();

                if (chosen is not { } pick)
                {
                    report = probe.Candidates.Count == 0 ? "no Vulkan device offers a compute queue" : $"no Vulkan device is named like \"{nameContains}\"";
                    return null;
                }

                var device = new GpuDevice(probe.Vk, probe.Instance, pick.Physical, pick.Name, pick.Software, pick.Family);
                probe.Release();
                report = $"{pick.Name}, Vulkan {pick.Api >> 22}.{(pick.Api >> 12) & 0x3FF}";
                return device;
            }
            catch (Exception e)
            {
                report = $"Vulkan is not available: {e.Message}";
                return null;
            }
            finally { probe?.Dispose(); }
        }

        public const string PortabilityEnumeration = "VK_KHR_portability_enumeration";
        public const string PortabilitySubset = "VK_KHR_portability_subset";

        private static HashSet<string> Names(uint count, ExtensionProperties[] found)
        {
            var names = new HashSet<string>();
            for (int i = 0; i < count; i++)
                fixed (byte* name = found[i].ExtensionName) names.Add(Marshal.PtrToStringAnsi((nint)name) ?? "");
            return names;
        }

        private static HashSet<string> InstanceExtensions(Vk vk)
        {
            uint count = 0;
            vk.EnumerateInstanceExtensionProperties((byte*)null, &count, null);
            var found = new ExtensionProperties[count];
            fixed (ExtensionProperties* p = found) vk.EnumerateInstanceExtensionProperties((byte*)null, &count, p);
            return Names(count, found);
        }

        private static HashSet<string> DeviceExtensions(Vk vk, PhysicalDevice physical)
        {
            uint count = 0;
            vk.EnumerateDeviceExtensionProperties(physical, (byte*)null, &count, null);
            var found = new ExtensionProperties[count];
            fixed (ExtensionProperties* p = found) vk.EnumerateDeviceExtensionProperties(physical, (byte*)null, &count, p);
            return Names(count, found);
        }

        private readonly record struct Candidate(PhysicalDevice Physical, string Name, bool Software, uint Family, uint Api, int Rank);

        // The instance and what it found; it owns the instance until a device takes it over.
        private sealed class Enumeration : IDisposable
        {
            public Vk Vk { get; }
            public Instance Instance { get; }
            public List<Candidate> Candidates { get; } = new();
            private bool _released;

            public Enumeration()
            {
                Vk = Vk.GetApi();

                var application = new ApplicationInfo { SType = StructureType.ApplicationInfo, ApiVersion = Vk.Version11 };
                var create = new InstanceCreateInfo { SType = StructureType.InstanceCreateInfo, PApplicationInfo = &application };

                // Without this a loader hides layered devices, which on macOS is every device there is - see Mars_Gpu.md §4.
                nint enumeration = SilkMarshal.StringToPtr(PortabilityEnumeration);
                try
                {
                    byte* name0 = (byte*)enumeration;
                    if (InstanceExtensions(Vk).Contains(PortabilityEnumeration))
                    {
                        create.EnabledExtensionCount = 1;
                        create.PpEnabledExtensionNames = &name0;
                        create.Flags = InstanceCreateFlags.EnumeratePortabilityBitKhr;
                    }

                    Instance instance;
                    Check(Vk.CreateInstance(&create, null, &instance), "vkCreateInstance");
                    Instance = instance;
                }
                finally { SilkMarshal.Free(enumeration); }

                uint count = 0;
                Vk.EnumeratePhysicalDevices(Instance, &count, null);
                var devices = new PhysicalDevice[count];
                fixed (PhysicalDevice* p = devices) Vk.EnumeratePhysicalDevices(Instance, &count, p);

                foreach (PhysicalDevice physical in devices)
                {
                    Vk.GetPhysicalDeviceProperties(physical, out PhysicalDeviceProperties properties);

                    uint families = 0;
                    Vk.GetPhysicalDeviceQueueFamilyProperties(physical, &families, null);
                    var queues = new QueueFamilyProperties[families];
                    fixed (QueueFamilyProperties* q = queues) Vk.GetPhysicalDeviceQueueFamilyProperties(physical, &families, q);

                    int family = Array.FindIndex(queues, q => (q.QueueFlags & QueueFlags.ComputeBit) != 0);
                    if (family < 0) continue;

                    int rank = properties.DeviceType switch
                    {
                        PhysicalDeviceType.DiscreteGpu => 0,
                        PhysicalDeviceType.IntegratedGpu => 1,
                        PhysicalDeviceType.VirtualGpu => 2,
                        _ => 3,
                    };

                    string name = Marshal.PtrToStringAnsi((nint)properties.DeviceName) ?? "unnamed";
                    Candidates.Add(new Candidate(physical, name, properties.DeviceType == PhysicalDeviceType.Cpu, (uint)family, properties.ApiVersion, rank));
                }

                Candidates.Sort((a, b) => a.Rank.CompareTo(b.Rank));
            }

            public void Release() => _released = true;

            public void Dispose()
            {
                if (_released) return;
                _released = true;
                Vk.DestroyInstance(Instance, null);
                Vk.Dispose();
            }
        }

        private static void Check(Result result, string call)
        {
            if (result != Result.Success) throw new InvalidOperationException($"{call} returned {result}");
        }

        public GpuBuffer CreateBuffer(ulong bytes, GpuMemory where = GpuMemory.Host)
        {
            var info = new BufferCreateInfo
            {
                SType = StructureType.BufferCreateInfo,
                Size = Math.Max(bytes, 4),
                Usage = BufferUsageFlags.StorageBufferBit | BufferUsageFlags.TransferSrcBit | BufferUsageFlags.TransferDstBit,
                SharingMode = SharingMode.Exclusive,
            };
            Check(_vk.CreateBuffer(_device, &info, null, out VkBuffer buffer), "vkCreateBuffer");
            _vk.GetBufferMemoryRequirements(_device, buffer, out MemoryRequirements needs);

            var allocate = new MemoryAllocateInfo
            {
                SType = StructureType.MemoryAllocateInfo, AllocationSize = needs.Size, MemoryTypeIndex = MemoryType(needs.MemoryTypeBits, where),
            };
            Check(_vk.AllocateMemory(_device, &allocate, null, out DeviceMemory memory), "vkAllocateMemory");
            Check(_vk.BindBufferMemory(_device, buffer, memory, 0), "vkBindBufferMemory");

            void* mapped = null;
            if (where == GpuMemory.Host) Check(_vk.MapMemory(_device, memory, 0, Vk.WholeSize, 0, &mapped), "vkMapMemory");

            return new GpuBuffer(this, buffer, memory, info.Size, (byte*)mapped);
        }

        // Host memory the CPU can read back quickly if there is any, since a readback from write-combined memory crawls.
        private uint MemoryType(uint allowed, GpuMemory where)
        {
            MemoryPropertyFlags need = where == GpuMemory.Host
                ? MemoryPropertyFlags.HostVisibleBit | MemoryPropertyFlags.HostCoherentBit
                : MemoryPropertyFlags.DeviceLocalBit;
            MemoryPropertyFlags want = where == GpuMemory.Host ? need | MemoryPropertyFlags.HostCachedBit : need;

            foreach (MemoryPropertyFlags flags in new[] { want, need })
                for (int i = 0; i < _memory.MemoryTypeCount; i++)
                    if ((allowed & (1u << i)) != 0 && (_memory.MemoryTypes[i].PropertyFlags & flags) == flags) return (uint)i;

            throw new InvalidOperationException($"no {where} memory type suits the buffer");
        }

        internal void Destroy(GpuBuffer buffer)
        {
            if (_disposed) return;
            if (buffer.Mapped != null) _vk.UnmapMemory(_device, buffer.Memory);
            _vk.DestroyBuffer(_device, buffer.Handle, null);
            _vk.FreeMemory(_device, buffer.Memory, null);
        }

        // A compute shader with its storage buffers at bindings 0..buffers-1 of set 0 and one block of push constants.
        public GpuProgram CreateProgram(ReadOnlySpan<byte> spirv, int buffers, int pushBytes)
        {
            if (spirv.Length == 0 || spirv.Length % 4 != 0) throw new ArgumentException("SPIR-V is a whole number of words", nameof(spirv));

            ShaderModule module;
            fixed (byte* code = spirv)
            {
                var moduleInfo = new ShaderModuleCreateInfo { SType = StructureType.ShaderModuleCreateInfo, CodeSize = (nuint)spirv.Length, PCode = (uint*)code };
                Check(_vk.CreateShaderModule(_device, &moduleInfo, null, &module), "vkCreateShaderModule");
            }

            var bindings = stackalloc DescriptorSetLayoutBinding[buffers];
            for (int i = 0; i < buffers; i++)
                bindings[i] = new DescriptorSetLayoutBinding
                {
                    Binding = (uint)i, DescriptorType = DescriptorType.StorageBuffer, DescriptorCount = 1, StageFlags = ShaderStageFlags.ComputeBit,
                };

            var setInfo = new DescriptorSetLayoutCreateInfo { SType = StructureType.DescriptorSetLayoutCreateInfo, BindingCount = (uint)buffers, PBindings = bindings };
            Check(_vk.CreateDescriptorSetLayout(_device, &setInfo, null, out DescriptorSetLayout setLayout), "vkCreateDescriptorSetLayout");

            var range = new PushConstantRange { StageFlags = ShaderStageFlags.ComputeBit, Offset = 0, Size = (uint)pushBytes };
            var layoutInfo = new PipelineLayoutCreateInfo
            {
                SType = StructureType.PipelineLayoutCreateInfo, SetLayoutCount = 1, PSetLayouts = &setLayout,
                PushConstantRangeCount = pushBytes > 0 ? 1u : 0u, PPushConstantRanges = pushBytes > 0 ? &range : null,
            };
            Check(_vk.CreatePipelineLayout(_device, &layoutInfo, null, out PipelineLayout layout), "vkCreatePipelineLayout");

            nint entry = SilkMarshal.StringToPtr("main");
            Pipeline pipeline;
            try
            {
                var pipelineInfo = new ComputePipelineCreateInfo
                {
                    SType = StructureType.ComputePipelineCreateInfo,
                    Layout = layout,
                    Stage = new PipelineShaderStageCreateInfo
                    {
                        SType = StructureType.PipelineShaderStageCreateInfo, Stage = ShaderStageFlags.ComputeBit, Module = module, PName = (byte*)entry,
                    },
                };
                Check(_vk.CreateComputePipelines(_device, default, 1, &pipelineInfo, null, &pipeline), "vkCreateComputePipelines");
            }
            finally
            {
                SilkMarshal.Free(entry);
                _vk.DestroyShaderModule(_device, module, null);
            }

            var size = new DescriptorPoolSize { Type = DescriptorType.StorageBuffer, DescriptorCount = (uint)buffers };
            var poolInfo = new DescriptorPoolCreateInfo { SType = StructureType.DescriptorPoolCreateInfo, MaxSets = 1, PoolSizeCount = 1, PPoolSizes = &size };
            Check(_vk.CreateDescriptorPool(_device, &poolInfo, null, out DescriptorPool pool), "vkCreateDescriptorPool");

            var setAllocate = new DescriptorSetAllocateInfo { SType = StructureType.DescriptorSetAllocateInfo, DescriptorPool = pool, DescriptorSetCount = 1, PSetLayouts = &setLayout };
            Check(_vk.AllocateDescriptorSets(_device, &setAllocate, out DescriptorSet set), "vkAllocateDescriptorSets");

            return new GpuProgram(this, pipeline, layout, setLayout, pool, set, buffers, pushBytes);
        }

        internal void Destroy(GpuProgram program)
        {
            if (_disposed) return;
            _vk.DestroyPipeline(_device, program.Pipeline, null);
            _vk.DestroyPipelineLayout(_device, program.Layout, null);
            _vk.DestroyDescriptorPool(_device, program.Pool, null);
            _vk.DestroyDescriptorSetLayout(_device, program.SetLayout, null);
        }

        // Records, submits and waits: nothing of a batch is in flight when this returns, which is what lets a set be rebound freely.
        public void Submit(Action<GpuCommands> record)
        {
            Check(_vk.ResetCommandBuffer(_commands, 0), "vkResetCommandBuffer");
            var begin = new CommandBufferBeginInfo { SType = StructureType.CommandBufferBeginInfo, Flags = CommandBufferUsageFlags.OneTimeSubmitBit };
            Check(_vk.BeginCommandBuffer(_commands, &begin), "vkBeginCommandBuffer");

            record(new GpuCommands(_vk, _device, _commands));

            Check(_vk.EndCommandBuffer(_commands), "vkEndCommandBuffer");

            CommandBuffer commands = _commands;
            var submit = new SubmitInfo { SType = StructureType.SubmitInfo, CommandBufferCount = 1, PCommandBuffers = &commands };
            Check(_vk.QueueSubmit(_queue, 1, &submit, _fence), "vkQueueSubmit");

            Fence fence = _fence;
            Check(_vk.WaitForFences(_device, 1, &fence, true, ulong.MaxValue), "vkWaitForFences");
            Check(_vk.ResetFences(_device, 1, &fence), "vkResetFences");
        }

        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;

            _vk.DeviceWaitIdle(_device);
            _vk.DestroyFence(_device, _fence, null);
            _vk.DestroyCommandPool(_device, _pool, null);
            _vk.DestroyDevice(_device, null);
            _vk.DestroyInstance(_instance, null);
            _vk.Dispose();
        }
    }

    // A storage buffer; a host one is mapped for its whole life and read or written through Span.
    public sealed unsafe class GpuBuffer : IDisposable
    {
        private readonly GpuDevice _owner;
        internal VkBuffer Handle { get; }
        internal DeviceMemory Memory { get; }
        internal byte* Mapped { get; }
        public ulong Bytes { get; }

        internal GpuBuffer(GpuDevice owner, VkBuffer handle, DeviceMemory memory, ulong bytes, byte* mapped)
        {
            _owner = owner;
            Handle = handle;
            Memory = memory;
            Bytes = bytes;
            Mapped = mapped;
        }

        public Span<T> Span<T>() where T : unmanaged =>
            Mapped == null ? throw new InvalidOperationException("device memory is reached by a copy, not mapped") : new Span<T>(Mapped, (int)(Bytes / (ulong)sizeof(T)));

        public void Dispose() => _owner.Destroy(this);
    }

    public sealed class GpuProgram : IDisposable
    {
        private readonly GpuDevice _owner;
        internal Pipeline Pipeline { get; }
        internal PipelineLayout Layout { get; }
        internal DescriptorSetLayout SetLayout { get; }
        internal DescriptorPool Pool { get; }
        internal DescriptorSet Set { get; }
        public int Buffers { get; }
        public int PushBytes { get; }

        internal GpuProgram(GpuDevice owner, Pipeline pipeline, PipelineLayout layout, DescriptorSetLayout setLayout, DescriptorPool pool, DescriptorSet set, int buffers, int pushBytes)
        {
            _owner = owner;
            Pipeline = pipeline;
            Layout = layout;
            SetLayout = setLayout;
            Pool = pool;
            Set = set;
            Buffers = buffers;
            PushBytes = pushBytes;
        }

        public void Dispose() => _owner.Destroy(this);
    }

    // What one submission may hold: copies and dispatches, each made to wait for the one before it.
    public readonly unsafe struct GpuCommands
    {
        private readonly Vk _vk;
        private readonly Device _device;
        private readonly CommandBuffer _commands;

        internal GpuCommands(Vk vk, Device device, CommandBuffer commands)
        {
            _vk = vk;
            _device = device;
            _commands = commands;
        }

        public void Fill(GpuBuffer buffer, uint word)
        {
            _vk.CmdFillBuffer(_commands, buffer.Handle, 0, Vk.WholeSize, word);
            Barrier();
        }

        public void Copy(GpuBuffer from, GpuBuffer to, ulong bytes = 0, ulong fromOffset = 0, ulong toOffset = 0)
        {
            var region = new BufferCopy { SrcOffset = fromOffset, DstOffset = toOffset, Size = bytes == 0 ? Math.Min(from.Bytes, to.Bytes) : bytes };
            _vk.CmdCopyBuffer(_commands, from.Handle, to.Handle, 1, &region);
            Barrier();
        }

        public void Dispatch<T>(GpuProgram program, ReadOnlySpan<GpuBuffer> buffers, in T push, uint x, uint y = 1, uint z = 1) where T : unmanaged
        {
            if (buffers.Length != program.Buffers) throw new ArgumentException($"the program binds {program.Buffers} buffers, not {buffers.Length}");
            if (sizeof(T) != program.PushBytes) throw new ArgumentException($"the program takes {program.PushBytes} bytes of push constants, not {sizeof(T)}");

            var infos = stackalloc DescriptorBufferInfo[buffers.Length];
            var writes = stackalloc WriteDescriptorSet[buffers.Length];
            for (int i = 0; i < buffers.Length; i++)
            {
                infos[i] = new DescriptorBufferInfo { Buffer = buffers[i].Handle, Offset = 0, Range = Vk.WholeSize };
                writes[i] = new WriteDescriptorSet
                {
                    SType = StructureType.WriteDescriptorSet, DstSet = program.Set, DstBinding = (uint)i, DescriptorCount = 1,
                    DescriptorType = DescriptorType.StorageBuffer, PBufferInfo = &infos[i],
                };
            }
            _vk.UpdateDescriptorSets(_device, (uint)buffers.Length, writes, 0, null);

            DescriptorSet set = program.Set;
            _vk.CmdBindPipeline(_commands, PipelineBindPoint.Compute, program.Pipeline);
            _vk.CmdBindDescriptorSets(_commands, PipelineBindPoint.Compute, program.Layout, 0, 1, &set, 0, null);
            fixed (T* p = &push) _vk.CmdPushConstants(_commands, program.Layout, ShaderStageFlags.ComputeBit, 0, (uint)sizeof(T), p);
            _vk.CmdDispatch(_commands, x, y, z);
            Barrier();
        }

        // Everything after waits for everything before: coarse, and one submission holds a handful of commands.
        private void Barrier()
        {
            var barrier = new MemoryBarrier
            {
                SType = StructureType.MemoryBarrier,
                SrcAccessMask = AccessFlags.ShaderWriteBit | AccessFlags.TransferWriteBit,
                DstAccessMask = AccessFlags.ShaderReadBit | AccessFlags.ShaderWriteBit | AccessFlags.TransferReadBit | AccessFlags.TransferWriteBit,
            };
            const PipelineStageFlags stages = PipelineStageFlags.ComputeShaderBit | PipelineStageFlags.TransferBit;
            _vk.CmdPipelineBarrier(_commands, stages, stages, 0, 1, &barrier, 0, null, 0, null);
        }
    }

    // The compiled shaders, embedded; their sources sit beside them and WiseMan holds the two equal - see Mars_Gpu.md §2.
    public static class GpuShaders
    {
        public static byte[] Load(string name)
        {
            using Stream stream = typeof(GpuShaders).Assembly.GetManifestResourceStream($"Mars.Gpu.{name}.spv")
                ?? throw new FileNotFoundException($"no compiled shader named {name}");
            using var copy = new MemoryStream();
            stream.CopyTo(copy);
            return copy.ToArray();
        }
    }
}
