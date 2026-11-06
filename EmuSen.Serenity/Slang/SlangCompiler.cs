using System;
using System.Text;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Slang
{
    public enum SlangStage { Vertex, Fragment }

    // A stage's Vulkan GLSL to SPIR-V through the Shaderc the platform library finds beside the program, unoptimised so the names reflection reads survive - see EmuSen_Serenity.md §7.3 and EmuSen_RustPlatform.md §33.
    public static unsafe class SlangCompiler
    {
        // Every option a compile is given, in the library's words; the cache's key carries them - see EmuSen_Serenity.md §9.4.
        internal const string Options = "optimization=Zero target=Vulkan/Vulkan11 language=Glsl entry=main defines=none";

        // The compiler itself, as far as its output can depend on it: the Shaderc file's SHA-256, its SPIR-V version, the library's binding and the options.
        internal static string Identity => LazyIdentity.Value;

        private static readonly Lazy<string> LazyIdentity = new(() =>
        {
            SerenityNative.Up();
            return SerenityNative.Identity();
        });

        // Half a surrogate pair in the text or the name crosses as U+FFFD - see EmuSen_Backend.md §40.6.
        public static byte[] Compile(string source, SlangStage stage, string name) => CompileNative(source, stage, name);

        internal static byte[] CompileNative(string source, SlangStage stage, string name)
        {
            SerenityNative.Up();
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
    }

    public sealed class SlangCompileException(string message) : Exception(message);
}
