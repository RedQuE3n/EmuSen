using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // A MercuryRT state read and written field by field through a state_layout listing, with no machine behind it - see Mercury_Native.md §14.5.
    public sealed class MercuryStateLayout
    {
        public readonly record struct Field(int Offset, int Length, string Type, string Path);

        private readonly Dictionary<string, Field> _byPath;

        private MercuryStateLayout(string name, string text)
        {
            Name = name;
            Text = text;
            Fields = text.Split('\n', StringSplitOptions.RemoveEmptyEntries).Select(Parse).ToArray();
            _byPath = Fields.ToDictionary(f => f.Path, StringComparer.Ordinal);
            Length = Fields.Count == 0 ? 0 : Fields[^1].Offset + Fields[^1].Length;
        }

        public string Name { get; }
        public string Text { get; }
        public IReadOnlyList<Field> Fields { get; }
        public int Length { get; }

        public static MercuryStateLayout FromText(string name, string text) => new(name, text);

        public Field this[string path] => _byPath.TryGetValue(path, out Field f) ? f : throw new KeyNotFoundException($"{Name} has no field {path}");

        public bool Has(string path) => _byPath.ContainsKey(path);

        // A scalar as a signed integer: bools 0 or 1, enums their number, doubles their bits.
        public long Get(ReadOnlySpan<byte> state, string path) => Get(state, this[path]);

        public static long Get(ReadOnlySpan<byte> state, Field f)
        {
            ReadOnlySpan<byte> b = state.Slice(f.Offset, f.Length);
            return f.Type switch
            {
                "u8" => b[0],
                "bool" => b[0] != 0 ? 1 : 0,
                "class" => b[0],
                "u16" => BinaryPrimitives.ReadUInt16LittleEndian(b),
                "i32" => BinaryPrimitives.ReadInt32LittleEndian(b),
                "u32" => BinaryPrimitives.ReadUInt32LittleEndian(b),
                "i64" or "f64" => BinaryPrimitives.ReadInt64LittleEndian(b),
                _ => throw new InvalidOperationException($"{f.Path} is a {f.Type}, not a scalar"),
            };
        }

        public double GetDouble(ReadOnlySpan<byte> state, string path) => BitConverter.Int64BitsToDouble(Get(state, path));

        public void Set(Span<byte> state, string path, long value)
        {
            Field f = this[path];
            Span<byte> b = state.Slice(f.Offset, f.Length);
            switch (f.Type)
            {
                case "u8" or "bool": b[0] = (byte)value; break;
                case "u16": BinaryPrimitives.WriteUInt16LittleEndian(b, (ushort)value); break;
                case "i32": BinaryPrimitives.WriteInt32LittleEndian(b, (int)value); break;
                case "u32": BinaryPrimitives.WriteUInt32LittleEndian(b, (uint)value); break;
                case "i64" or "f64": BinaryPrimitives.WriteInt64LittleEndian(b, value); break;
                default: throw new InvalidOperationException($"{path} is a {f.Type}, not a scalar");
            }
        }

        // An array field's bytes in place.
        public Span<byte> Bytes(Span<byte> state, string path)
        {
            Field f = this[path];
            if (!f.Type.StartsWith("u8[", StringComparison.Ordinal)) throw new InvalidOperationException($"{path} is a {f.Type}, not a byte array");
            return state.Slice(f.Offset, f.Length);
        }

        // The fields two states of this layout differ in, each with both values, for a failure message.
        public IEnumerable<string> Differences(ReadOnlySpan<byte> a, ReadOnlySpan<byte> b)
        {
            var found = new List<string>();
            foreach (Field f in Fields)
            {
                if (a.Slice(f.Offset, f.Length).SequenceEqual(b.Slice(f.Offset, f.Length))) continue;
                if (f.Type.StartsWith("u8[", StringComparison.Ordinal))
                {
                    int at = a.Slice(f.Offset, f.Length).CommonPrefixLength(b.Slice(f.Offset, f.Length));
                    found.Add($"{f.Path}[{at}]: {a[f.Offset + at]:X2} against {b[f.Offset + at]:X2}");
                }
                else found.Add($"{f.Path}: {Get(a, f)} against {Get(b, f)}");
            }
            return found;
        }

        private static Field Parse(string line)
        {
            string[] p = line.Split(' ', 4);
            return new Field(int.Parse(p[0], CultureInfo.InvariantCulture), int.Parse(p[1], CultureInfo.InvariantCulture), p[2], p[3]);
        }
    }
}
