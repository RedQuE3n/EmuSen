using System;
using System.Text;
using Silk.NET.Shaderc;

namespace EmuSen.Serenity.Slang
{
    public enum SlangStage { Vertex, Fragment }

    // A stage's Vulkan GLSL to SPIR-V through Shaderc, unoptimised so the names reflection reads survive - see EmuSen_Serenity.md §7.3.
    public static unsafe class SlangCompiler
    {
        private static readonly Shaderc Api = Shaderc.GetApi();

        public static byte[] Compile(string source, SlangStage stage, string name)
        {
            Compiler* compiler = Api.CompilerInitialize();
            CompileOptions* options = Api.CompileOptionsInitialize();
            try
            {
                Api.CompileOptionsSetOptimizationLevel(options, OptimizationLevel.Zero);
                Api.CompileOptionsSetTargetEnv(options, TargetEnv.Vulkan, (uint)EnvVersion.Vulkan11);
                Api.CompileOptionsSetSourceLanguage(options, SourceLanguage.Glsl);
                byte[] text = Encoding.UTF8.GetBytes(source);
                ShaderKind kind = stage == SlangStage.Vertex ? ShaderKind.VertexShader : ShaderKind.FragmentShader;
                CompilationResult* result;
                fixed (byte* p = text) result = Api.CompileIntoSpv(compiler, p, (nuint)text.Length, kind, name, "main", options);
                try
                {
                    if (Api.ResultGetCompilationStatus(result) != CompilationStatus.Success)
                        throw new SlangCompileException($"{name} ({stage}) does not compile:\n{Api.ResultGetErrorMessageS(result)}");
                    return new ReadOnlySpan<byte>(Api.ResultGetBytes(result), (int)Api.ResultGetLength(result)).ToArray();
                }
                finally
                {
                    Api.ResultRelease(result);
                }
            }
            finally
            {
                Api.CompileOptionsRelease(options);
                Api.CompilerRelease(compiler);
            }
        }
    }

    public sealed class SlangCompileException(string message) : Exception(message);
}
