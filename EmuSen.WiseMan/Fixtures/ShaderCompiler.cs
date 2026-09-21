using System;
using System.IO;
using System.Text;
using Silk.NET.Shaderc;

namespace EmuSen.WiseMan.Fixtures
{
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
