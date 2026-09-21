using System;
using System.IO;
using System.Text;
using Silk.NET.Shaderc;

namespace EmuSen.WiseMan.Fixtures
{
    // Which devices the GPU tests run on: the machine's first choice, or every one when asked - see Mars_Gpu.md §3.
    public static class GpuTestDevices
    {
        public const string AllVariable = "EMUSEN_MARS_GPU_TEST_DEVICES";

        public static string[] Names
        {
            get
            {
                string[] names = System.Linq.Enumerable.ToArray(System.Linq.Enumerable.Distinct(EmuSen.Cores.Nintendo.Mars.Rdp.Gpu.GpuDevice.DeviceNames()));
                if (names.Length == 0) return new[] { "" };
                return Environment.GetEnvironmentVariable(AllVariable) == "all" ? names : new[] { names[0] };
            }
        }
    }

    // GLSL compute source to SPIR-V, for Mars's shaders; the product ships the result and never this - see Mars_Gpu.md §2.
    public static unsafe class ShaderCompiler
    {
        public const string RecordVariable = "EMUSEN_GPU_SHADERS_RECORD";

        public static string ShaderDirectory
        {
            get
            {
                DirectoryInfo? at = new(AppContext.BaseDirectory);
                while (at is not null && !File.Exists(Path.Combine(at.FullName, "EmuSen.sln"))) at = at.Parent;
                if (at is null) throw new DirectoryNotFoundException("the repository is not above the test binaries");
                return Path.Combine(at.FullName, "EmuSen", "Cores", "Nintendo", "Mars - N64", "Rdp", "Gpu", "Shaders");
            }
        }

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
