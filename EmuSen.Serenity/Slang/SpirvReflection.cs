using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Slang
{
    // A member of the uniform block or the push constants: where it sits and how many bytes it takes.
    public sealed record SlangMember(string Name, uint Offset, uint Size);

    // A uniform block or the push-constant block; Binding is meaningless for the latter.
    public sealed record SlangBlock(uint Binding, uint Size, IReadOnlyList<SlangMember> Members)
    {
        public SlangMember? Find(string name) => Members.FirstOrDefault(m => m.Name == name);
    }

    public sealed record SlangSampler(string Name, uint Binding);

    // What a stage binds, read from its SPIR-V's names and decorations rather than from SPIRV-Cross - see EmuSen_Serenity.md §7.3.
    public sealed class SpirvReflection
    {
        public SlangBlock? Uniforms { get; }
        public SlangBlock? PushConstants { get; }
        public IReadOnlyList<SlangSampler> Samplers { get; }

        private SpirvReflection(SlangBlock? uniforms, SlangBlock? push, IReadOnlyList<SlangSampler> samplers)
        {
            Uniforms = uniforms;
            PushConstants = push;
            Samplers = samplers;
        }

        // The library's reading - see EmuSen_RustPlatform.md §16.2.
        public static SpirvReflection Read(byte[] spirv) => ReadNative(spirv ?? throw new ArgumentNullException(nameof(spirv)));

        internal static unsafe SpirvReflection ReadNative(byte[] spirv)
        {
            using JsonDocument document = SerenityNative.Document((buffer, capacity) =>
            {
                fixed (byte* s = spirv) return SerenityNative.Reflect(s, (nuint)spirv.Length, buffer, capacity);
            });
            if (SerenityNative.Failure(document.RootElement) is { } failure) throw failure;
            return FromJson(document.RootElement);
        }

        // The module with every Input variable no instruction reads left out of its entry points' interfaces, or the same array if there is none - see EmuSen_Serenity.md §10.3.
        public static byte[] WithoutUnreadInputs(byte[] spirv) => WithoutUnreadInputsNative(spirv ?? throw new ArgumentNullException(nameof(spirv)));

        internal static unsafe byte[] WithoutUnreadInputsNative(byte[] spirv)
        {
            byte[]? pruned = SerenityNative.Bytes((buffer, capacity) =>
            {
                fixed (byte* s = spirv) return SerenityNative.WithoutUnreadInputs(s, (nuint)spirv.Length, buffer, capacity);
            }, out long status, Math.Max(64, spirv.Length));
            if (pruned is not null) return pruned;
            if (status == SerenityNative.Absent) return spirv;
            if (status == SerenityNative.BadArgument) throw new ArgumentOutOfRangeException();
            throw new InvalidOperationException(SerenityNative.Words());
        }

        // The two stages as one pass: their blocks' members together, and every sampler either names.
        public static SpirvReflection Merge(SpirvReflection vertex, SpirvReflection fragment) =>
            MergeNative(vertex, fragment);

        internal static unsafe SpirvReflection MergeNative(SpirvReflection vertex, SpirvReflection fragment)
        {
            byte[] input = JsonSerializer.SerializeToUtf8Bytes(new { vertex = ToJson(vertex), fragment = ToJson(fragment) });
            using JsonDocument document = SerenityNative.Document((buffer, capacity) =>
            {
                fixed (byte* i = input) return SerenityNative.ReflectMerge(i, (nuint)input.Length, buffer, capacity);
            });
            return FromJson(document.RootElement);
        }

        private static object ToJson(SpirvReflection reflection)
        {
            static object? Block(SlangBlock? block) => block is null ? null : new { binding = block.Binding, size = block.Size, members = block.Members.Select(m => new { name = m.Name, offset = m.Offset, size = m.Size }) };
            return new { uniforms = Block(reflection.Uniforms), push = Block(reflection.PushConstants), samplers = reflection.Samplers.Select(s => new { name = s.Name, binding = s.Binding }) };
        }

        private static SpirvReflection FromJson(JsonElement root)
        {
            static SlangBlock? Block(JsonElement block) => block.ValueKind == JsonValueKind.Null ? null : new SlangBlock(block.GetProperty("binding").GetUInt32(), block.GetProperty("size").GetUInt32(),
                block.GetProperty("members").EnumerateArray().Select(m => new SlangMember(m.GetProperty("name").GetString()!, m.GetProperty("offset").GetUInt32(), m.GetProperty("size").GetUInt32())).ToList());
            return new SpirvReflection(Block(root.GetProperty("uniforms")), Block(root.GetProperty("push")),
                root.GetProperty("samplers").EnumerateArray().Select(s => new SlangSampler(s.GetProperty("name").GetString()!, s.GetProperty("binding").GetUInt32())).ToList());
        }
    }
}
