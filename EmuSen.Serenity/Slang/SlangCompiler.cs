using System;
using System.Diagnostics;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using EmuSen.Serenity.Native;
using Silk.NET.Shaderc;

namespace EmuSen.Serenity.Slang
{
    public enum SlangStage { Vertex, Fragment }

    // A stage's Vulkan GLSL to SPIR-V through Shaderc, unoptimised so the names reflection reads survive - see EmuSen_Serenity.md §7.3.
    public static unsafe class SlangCompiler
    {
        // Every option a compile is given; the cache's key carries them, so one changed here cannot serve an old entry - see EmuSen_Serenity.md §9.4.
        private const OptimizationLevel Optimization = OptimizationLevel.Zero;
        private const TargetEnv Target = TargetEnv.Vulkan;
        private const EnvVersion TargetVersion = EnvVersion.Vulkan11;
        private const SourceLanguage Language = SourceLanguage.Glsl;
        private const string Entry = "main";

        internal static readonly string Options = $"optimization={Optimization} target={Target}/{TargetVersion} language={Language} entry={Entry} defines=none";

        // The compiler itself, as far as its output can depend on it: the library's, which names its own binding, or the C#'s.
        internal static string Identity => SerenityNative.Active ? NativeIdentity.Value : Managed.Identity;

        private static readonly Lazy<string> NativeIdentity = new(SerenityNative.Identity);

        // The library's compile or the C#'s, both through the one Shaderc; text that is no text of its own is the C#'s to take - see EmuSen_RustPlatform.md §16.2.
        public static byte[] Compile(string source, SlangStage stage, string name) =>
            SerenityNative.Active && SerenityNative.Crosses(source) && SerenityNative.Crosses(name) ? CompileNative(source, stage, name) : Managed.Compile(source, stage, name);

        internal static byte[] CompileNative(string source, SlangStage stage, string name)
        {
            byte[] text = Encoding.UTF8.GetBytes(source), file = Encoding.UTF8.GetBytes(name);
            byte[]? spirv = SerenityNative.Bytes((buffer, capacity) =>
            {
                fixed (byte* t = text, f = file)
                    return SerenityNative.Compile(SerenityNative.Pin(t), (nuint)text.Length, (uint)stage, SerenityNative.Pin(f), (nuint)file.Length, buffer, capacity);
            }, out long status);
            if (spirv is not null) return spirv;
            if (status == SerenityNative.Parse) throw new SlangCompileException(SerenityNative.Words());
            throw new InvalidOperationException(SerenityNative.Words());
        }

        // The C# compiler: the default, and what the library's is held to - see EmuSen_RustPlatform.md §16.
        internal static class Managed
        {
            internal static readonly Shaderc Api = Shaderc.GetApi();

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

            // The file the loaded Shaderc library came from, which the library is told so that its identity carries the same digest; null when the process cannot name it.
            internal static string? NativePath()
            {
                try
                {
                    foreach (ProcessModule module in Process.GetCurrentProcess().Modules)
                        if (Path.GetFileName(module.FileName).Contains("shaderc", StringComparison.OrdinalIgnoreCase)) return module.FileName;
                }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException or InvalidOperationException or NotSupportedException or System.ComponentModel.Win32Exception)
                {
                }
                return null;
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
    }

    public sealed class SlangCompileException(string message) : Exception(message);
}
