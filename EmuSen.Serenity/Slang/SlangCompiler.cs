using System;
using System.Diagnostics;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using Silk.NET.Shaderc;

namespace EmuSen.Serenity.Slang
{
    public enum SlangStage { Vertex, Fragment }

    // A stage's Vulkan GLSL to SPIR-V through Shaderc, unoptimised so the names reflection reads survive - see EmuSen_Serenity.md §7.3.
    public static unsafe class SlangCompiler
    {
        private static readonly Shaderc Api = Shaderc.GetApi();

        // Every option a compile is given; the cache's key carries them, so one changed here cannot serve an old entry - see EmuSen_Serenity.md §9.4.
        private const OptimizationLevel Optimization = OptimizationLevel.Zero;
        private const TargetEnv Target = TargetEnv.Vulkan;
        private const EnvVersion TargetVersion = EnvVersion.Vulkan11;
        private const SourceLanguage Language = SourceLanguage.Glsl;
        private const string Entry = "main";

        internal static readonly string Options = $"optimization={Optimization} target={Target}/{TargetVersion} language={Language} entry={Entry} defines=none";

        private static readonly Lazy<string> LazyIdentity = new(ReadIdentity);

        // The compiler itself, as far as its output can depend on it: the native library's bytes, its SPIR-V version, the binding's and the options.
        internal static string Identity => LazyIdentity.Value;

        public static byte[] Compile(string source, SlangStage stage, string name)
        {
            Compiler* compiler = Api.CompilerInitialize();
            CompileOptions* options = Api.CompileOptionsInitialize();
            try
            {
                Api.CompileOptionsSetOptimizationLevel(options, Optimization);
                Api.CompileOptionsSetTargetEnv(options, Target, (uint)TargetVersion);
                Api.CompileOptionsSetSourceLanguage(options, Language);
                byte[] text = Encoding.UTF8.GetBytes(source);
                ShaderKind kind = stage == SlangStage.Vertex ? ShaderKind.VertexShader : ShaderKind.FragmentShader;
                CompilationResult* result;
                fixed (byte* p = text) result = Api.CompileIntoSpv(compiler, p, (nuint)text.Length, kind, name, Entry, options);
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

        private static string ReadIdentity()
        {
            uint version = 0, revision = 0;
            Api.GetSpvVersion(&version, &revision);
            return $"shaderc {NativeDigest()}; spv {version:X}.{revision}; binding {typeof(Shaderc).Assembly.GetName().Version}; {Options}";
        }

        // The loaded Shaderc library's SHA-256, or its absence said, when the process cannot name the file it came from.
        private static string NativeDigest()
        {
            try
            {
                foreach (ProcessModule module in Process.GetCurrentProcess().Modules)
                {
                    if (!Path.GetFileName(module.FileName).Contains("shaderc", StringComparison.OrdinalIgnoreCase)) continue;
                    using FileStream file = File.OpenRead(module.FileName);
                    return Convert.ToHexString(SHA256.HashData(file));
                }
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or InvalidOperationException or NotSupportedException or System.ComponentModel.Win32Exception)
            {
                return $"unread ({e.GetType().Name})";
            }
            return "unfound";
        }
    }

    public sealed class SlangCompileException(string message) : Exception(message);
}
