using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

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
        private const uint Magic = 0x07230203;
        private const int OpName = 5, OpMemberName = 6, OpTypeInt = 21, OpTypeFloat = 22, OpTypeVector = 23, OpTypeMatrix = 24,
            OpTypeImage = 25, OpTypeSampledImage = 27, OpTypeArray = 28, OpTypeStruct = 30, OpTypePointer = 32, OpConstant = 43,
            OpVariable = 59, OpDecorate = 71, OpMemberDecorate = 72;
        private const uint DecorationBinding = 33, DecorationDescriptorSet = 34, DecorationOffset = 35, DecorationArrayStride = 6, DecorationMatrixStride = 7;
        private const uint StorageUniformConstant = 0, StorageUniform = 2, StoragePushConstant = 9;

        public SlangBlock? Uniforms { get; }
        public SlangBlock? PushConstants { get; }
        public IReadOnlyList<SlangSampler> Samplers { get; }

        private SpirvReflection(SlangBlock? uniforms, SlangBlock? push, IReadOnlyList<SlangSampler> samplers)
        {
            Uniforms = uniforms;
            PushConstants = push;
            Samplers = samplers;
        }

        public static SpirvReflection Read(byte[] spirv)
        {
            if (spirv.Length < 20 || spirv.Length % 4 != 0) throw new ArgumentException("Not SPIR-V: too short or not whole words.");
            uint[] words = new uint[spirv.Length / 4];
            Buffer.BlockCopy(spirv, 0, words, 0, spirv.Length);
            if (words[0] != Magic) throw new ArgumentException("Not SPIR-V: the magic number is wrong.");

            var names = new Dictionary<uint, string>();
            var memberNames = new Dictionary<(uint, uint), string>();
            var bindings = new Dictionary<uint, uint>();
            var memberOffsets = new Dictionary<(uint, uint), uint>();
            var arrayStrides = new Dictionary<uint, uint>();
            var matrixStrides = new Dictionary<(uint, uint), uint>();
            var types = new Dictionary<uint, (int Op, uint[] Operands)>();
            var constants = new Dictionary<uint, uint>();
            var variables = new List<(uint Type, uint Id, uint Storage)>();

            for (int at = 5; at < words.Length;)
            {
                int count = (int)(words[at] >> 16), op = (int)(words[at] & 0xFFFF);
                if (count == 0 || at + count > words.Length) throw new ArgumentException($"Not SPIR-V: an instruction at word {at} runs past the end.");
                uint[] operands = words[(at + 1)..(at + count)];
                switch (op)
                {
                    case OpName: names[operands[0]] = Text(operands, 1); break;
                    case OpMemberName: memberNames[(operands[0], operands[1])] = Text(operands, 2); break;
                    case OpDecorate when operands[1] == DecorationBinding: bindings[operands[0]] = operands[2]; break;
                    case OpDecorate when operands[1] == DecorationArrayStride: arrayStrides[operands[0]] = operands[2]; break;
                    case OpMemberDecorate when operands[2] == DecorationOffset: memberOffsets[(operands[0], operands[1])] = operands[3]; break;
                    case OpMemberDecorate when operands[2] == DecorationMatrixStride: matrixStrides[(operands[0], operands[1])] = operands[3]; break;
                    case OpTypeInt or OpTypeFloat or OpTypeVector or OpTypeMatrix or OpTypeImage or OpTypeSampledImage or OpTypeArray or OpTypeStruct or OpTypePointer:
                        types[operands[0]] = (op, operands[1..]); break;
                    case OpConstant when operands.Length >= 3: constants[operands[1]] = operands[2]; break;
                    case OpVariable: variables.Add((operands[0], operands[1], operands[2])); break;
                }
                at += count;
            }

            uint Pointee(uint pointer) => types.TryGetValue(pointer, out var t) && t.Op == OpTypePointer ? t.Operands[1] : pointer;

            uint SizeOf(uint type, uint matrixStride)
            {
                if (!types.TryGetValue(type, out var t)) return 0;
                return t.Op switch
                {
                    OpTypeInt or OpTypeFloat => t.Operands[0] / 8,
                    OpTypeVector => SizeOf(t.Operands[0], 0) * t.Operands[1],
                    OpTypeMatrix => t.Operands[1] * (matrixStride != 0 ? matrixStride : SizeOf(t.Operands[0], 0)),
                    OpTypeArray => (arrayStrides.TryGetValue(type, out uint stride) ? stride : SizeOf(t.Operands[0], matrixStride)) * (constants.TryGetValue(t.Operands[1], out uint length) ? length : 1),
                    OpTypeStruct => StructSize(type, t.Operands),
                    _ => 0,
                };
            }

            uint StructSize(uint type, uint[] members)
            {
                uint end = 0;
                for (uint m = 0; m < members.Length; m++)
                {
                    uint offset = memberOffsets.TryGetValue((type, m), out uint o) ? o : 0;
                    end = Math.Max(end, offset + SizeOf(members[m], matrixStrides.TryGetValue((type, m), out uint s) ? s : 0));
                }
                return end;
            }

            SlangBlock Block(uint variable, uint structType)
            {
                uint[] members = types[structType].Operands;
                var list = new List<SlangMember>();
                for (uint m = 0; m < members.Length; m++)
                {
                    string name = memberNames.TryGetValue((structType, m), out string? n) ? n : $"_m{m}";
                    uint offset = memberOffsets.TryGetValue((structType, m), out uint o) ? o : 0;
                    list.Add(new SlangMember(name, offset, SizeOf(members[m], matrixStrides.TryGetValue((structType, m), out uint s) ? s : 0)));
                }
                return new SlangBlock(bindings.TryGetValue(variable, out uint b) ? b : 0, StructSize(structType, members), list);
            }

            SlangBlock? uniforms = null, push = null;
            var samplers = new List<SlangSampler>();
            foreach (var (pointer, id, storage) in variables)
            {
                uint pointee = Pointee(pointer);
                if (!types.TryGetValue(pointee, out var t)) continue;
                if (storage == StorageUniform && t.Op == OpTypeStruct) uniforms = Block(id, pointee);
                else if (storage == StoragePushConstant && t.Op == OpTypeStruct) push = Block(id, pointee);
                else if (storage == StorageUniformConstant && t.Op == OpTypeSampledImage && names.TryGetValue(id, out string? name))
                    samplers.Add(new SlangSampler(name, bindings.TryGetValue(id, out uint binding) ? binding : 0));
            }
            return new SpirvReflection(uniforms, push, samplers);
        }

        // A SPIR-V literal string: UTF-8, zero-terminated, packed four bytes to a word.
        private static string Text(uint[] operands, int from)
        {
            var bytes = new List<byte>();
            for (int i = from; i < operands.Length; i++)
                for (int shift = 0; shift < 32; shift += 8)
                {
                    byte b = (byte)(operands[i] >> shift);
                    if (b == 0) return Encoding.UTF8.GetString(bytes.ToArray());
                    bytes.Add(b);
                }
            return Encoding.UTF8.GetString(bytes.ToArray());
        }

        // The two stages as one pass: their blocks' members together, and every sampler either names.
        public static SpirvReflection Merge(SpirvReflection vertex, SpirvReflection fragment)
        {
            static SlangBlock? Join(SlangBlock? a, SlangBlock? b)
            {
                if (a is null) return b;
                if (b is null) return a;
                var members = a.Members.Concat(b.Members.Where(m => a.Find(m.Name) is null)).ToList();
                return new SlangBlock(a.Binding, Math.Max(a.Size, b.Size), members);
            }
            var samplers = vertex.Samplers.Concat(fragment.Samplers).GroupBy(s => s.Name).Select(g => g.First()).ToList();
            return new SpirvReflection(Join(vertex.Uniforms, fragment.Uniforms), Join(vertex.PushConstants, fragment.PushConstants), samplers);
        }
    }
}
