using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Globalization;

namespace EmuSen.Cores.Native
{
    // A state read by its core's own listing, `offset length type path` a line, so a port's shim reads a field by path - see EmuSen_NativeCores.md §3.14.
    public sealed class StateLayout
    {
        private readonly record struct Field(int Offset, int Length, string Type, int Count);

        private readonly Dictionary<string, Field> _fields = new(StringComparer.Ordinal);
        private readonly List<string> _paths = new();

        public StateLayout(string listing)
        {
            foreach (string line in listing.Split('\n'))
            {
                string[] p = line.TrimEnd('\r').Split(' ', 4);
                if (p.Length < 4) continue;
                string type = p[2];
                int count = 1, bracket = type.IndexOf('[');
                if (bracket >= 0)
                {
                    count = int.Parse(type[(bracket + 1)..^1], CultureInfo.InvariantCulture);
                    type = type[..bracket];
                }
                if (_fields.TryAdd(p[3], new Field(int.Parse(p[0], CultureInfo.InvariantCulture), int.Parse(p[1], CultureInfo.InvariantCulture), type, count))) _paths.Add(p[3]);
            }
        }

        // Every path in the order the state writes it.
        public IReadOnlyList<string> Paths => _paths;

        public bool Has(string path) => _fields.ContainsKey(path);

        // A scalar or an array element as a number, signed or not as its type is; a class flag reads as 0 or 1.
        public long Int(ReadOnlySpan<byte> state, string path, int index = 0)
        {
            Field f = _fields.TryGetValue(path, out var found) ? found : throw new KeyNotFoundException($"the state has no field {path}");
            if (index < 0 || index >= f.Count) throw new ArgumentOutOfRangeException(nameof(index), $"{path} has {f.Count} elements");
            int size = f.Length / Math.Max(1, f.Count);
            ReadOnlySpan<byte> b = state.Slice(f.Offset + index * size, size);
            return f.Type switch
            {
                "u8" or "bool" or "class" => b[0],
                "i8" => (sbyte)b[0],
                "u16" => BinaryPrimitives.ReadUInt16LittleEndian(b),
                "i16" => BinaryPrimitives.ReadInt16LittleEndian(b),
                "u32" => BinaryPrimitives.ReadUInt32LittleEndian(b),
                "i32" => BinaryPrimitives.ReadInt32LittleEndian(b),
                "u64" or "i64" => BinaryPrimitives.ReadInt64LittleEndian(b),
                _ => throw new InvalidOperationException($"{path} is a {f.Type}, not a number"),
            };
        }

        public bool Flag(ReadOnlySpan<byte> state, string path) => Int(state, path) != 0;

        public double Double(ReadOnlySpan<byte> state, string path) =>
            _fields.TryGetValue(path, out var f) && f.Type == "f64" ? BinaryPrimitives.ReadDoubleLittleEndian(state.Slice(f.Offset, 8)) : throw new KeyNotFoundException($"the state has no double {path}");

        public ReadOnlySpan<byte> Bytes(ReadOnlySpan<byte> state, string path) =>
            _fields.TryGetValue(path, out var f) ? state.Slice(f.Offset, f.Length) : throw new KeyNotFoundException($"the state has no field {path}");

        // A field's bits as the debugger shows them: eight per byte of a scalar, one for a flag.
        public int Bits(string path) => _fields.TryGetValue(path, out var f) ? f.Type is "bool" or "class" ? 1 : f.Length / Math.Max(1, f.Count) * 8 : 0;

        public int Count(string path) => _fields.TryGetValue(path, out var f) ? f.Count : 0;

        public bool IsNumber(string path) => _fields.TryGetValue(path, out var f) && f.Type is "u8" or "bool" or "class" or "i8" or "u16" or "i16" or "u32" or "i32" or "u64" or "i64";
    }
}
