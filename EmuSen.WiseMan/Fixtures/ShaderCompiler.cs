using System;
using System.IO;
using System.Text;
using Silk.NET.Shaderc;
using Silk.NET.Vulkan;

namespace EmuSen.WiseMan.Fixtures
{
    // Whether this machine has a Vulkan device with a compute queue, for a test that otherwise stands down.
    public static unsafe class VulkanDevices
    {
        public static bool Any()
        {
            try
            {
                Vk vk = Vk.GetApi();
                var application = new ApplicationInfo { SType = StructureType.ApplicationInfo, ApiVersion = Vk.Version11 };
                var create = new InstanceCreateInfo { SType = StructureType.InstanceCreateInfo, PApplicationInfo = &application };
                Instance instance;
                if (vk.CreateInstance(&create, null, &instance) != Result.Success) return false;
                try
                {
                    uint count = 0;
                    vk.EnumeratePhysicalDevices(instance, &count, null);
                    var devices = new PhysicalDevice[count];
                    fixed (PhysicalDevice* p = devices) vk.EnumeratePhysicalDevices(instance, &count, p);
                    foreach (PhysicalDevice device in devices)
                    {
                        uint families = 0;
                        vk.GetPhysicalDeviceQueueFamilyProperties(device, &families, null);
                        var queues = new QueueFamilyProperties[families];
                        fixed (QueueFamilyProperties* q = queues) vk.GetPhysicalDeviceQueueFamilyProperties(device, &families, q);
                        if (Array.Exists(queues, q => (q.QueueFlags & QueueFlags.ComputeBit) != 0)) return true;
                    }
                    return false;
                }
                finally { vk.DestroyInstance(instance, null); }
            }
            catch (Exception) { return false; }
        }
    }

    // GLSL compute source to SPIR-V, for MarsRT's and Diamond's shaders; the product ships the result and never this - see Mars_Gpu.md §2.
    public static unsafe class ShaderCompiler
    {
        public const string RecordVariable = "EMUSEN_GPU_SHADERS_RECORD";

        public static string RepositoryRoot
        {
            get
            {
                DirectoryInfo? at = new(AppContext.BaseDirectory);
                while (at is not null && !File.Exists(Path.Combine(at.FullName, "EmuSen.sln"))) at = at.Parent;
                if (at is null) throw new DirectoryNotFoundException("the repository is not above the test binaries");
                return at.FullName;
            }
        }

        // MarsRT's shaders, which its crate embeds - see Mars_Gpu.md §2.
        public static string MarsRtShaderDirectory => Path.Combine(RepositoryRoot, "EmuSen", "Cores", "Nintendo", "MarsRT - N64", "shaders");

        // Diamond's renderer's shaders, which its crate embeds - see Diamond_Enhancements.md §14.5.
        public static string DiamondShaderDirectory => Path.Combine(RepositoryRoot, "EmuSen", "Cores", "Sony", "Diamond - PlayStation", "renderer", "shaders");

        public static byte[] Compile(string source, string name)
        {
            Shaderc api = Shaderc.GetApi();
            Compiler* compiler = api.CompilerInitialize();
            CompileOptions* options = api.CompileOptionsInitialize();

            try
            {
                api.CompileOptionsSetOptimizationLevel(options, OptimizationLevel.Performance);

                byte[] text = Encoding.UTF8.GetBytes(source);
                CompilationResult* result;
                fixed (byte* p = text) result = api.CompileIntoSpv(compiler, p, (nuint)text.Length, ShaderKind.ComputeShader, name, "main", options);

                try
                {
                    if (api.ResultGetCompilationStatus(result) != CompilationStatus.Success)
                        throw new InvalidOperationException($"{name} does not compile:\n{api.ResultGetErrorMessageS(result)}");

                    return new ReadOnlySpan<byte>(api.ResultGetBytes(result), (int)api.ResultGetLength(result)).ToArray();
                }
                finally { api.ResultRelease(result); }
            }
            finally
            {
                api.CompileOptionsRelease(options);
                api.CompilerRelease(compiler);
            }
        }
    }
}
