using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;

namespace EmuSen.WiseMan.Fixtures
{
    // A Mercury state read and written field by field through a committed layout listing, with no machine behind it - see Mercury_Native.md §14.5.
    public sealed class MercuryStateLayout
    {
        // Set to 1 to write a listing missing for a board, console and RAM size from MercuryRT's state_layout instead of failing.
        public const string RecordVariable = "EMUSEN_MERCURY_LAYOUT_RECORD";

        // The state version the committed listings describe; a new version records them again.
        public const int StateVersion = 11;

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

        public static string Directory([CallerFilePath] string here = "") =>
            Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(here)!)!, "Cores", "MercuryGoldens", "layouts");

        // The board a cartridge type selects, as both engines' Cartridge.FromImage does; the listings are named by it.
        public static string Board(byte cartridgeType) => cartridgeType switch
        {
            0x00 or 0x08 or 0x09 => "none",
            >= 0x01 and <= 0x03 => "mbc1",
            0x05 or 0x06 => "mbc2",
            >= 0x0F and <= 0x13 => "mbc3",
            >= 0x19 and <= 0x1E => "mbc5",
            _ => throw new ArgumentException($"no board for cartridge type ${cartridgeType:X2}"),
        };

        public static string FileName(string board, bool colourConsole, int ramLength) => $"{board}-{(colourConsole ? "gbc" : "gb")}-ram{ramLength}.layout";

        // The listing a state of this cartridge follows: the board from the header, the console from the state's own header, the RAM size from its length.
        public static MercuryStateLayout For(byte[] rom, ReadOnlySpan<byte> state)
        {
            string board = Board(rom[0x147]);
            bool colour = state[8] != 0;
            string prefix = $"{board}-{(colour ? "gbc" : "gb")}-";
            lock (Chosen)
                if (Chosen.TryGetValue((prefix, state.Length), out MercuryStateLayout? known)) return known;
            string dir = Directory();
            foreach (string path in System.IO.Directory.Exists(dir) ? System.IO.Directory.GetFiles(dir, prefix + "*.layout") : Array.Empty<string>())
            {
                var layout = Load(path);
                if (layout.Length != state.Length) continue;
                lock (Chosen) Chosen[(prefix, state.Length)] = layout;
                return layout;
            }
            throw new InvalidDataException($"No committed layout for a {board} board on the {(colour ? "Game Boy Color" : "Game Boy")} with a {state.Length}-byte state; record one with {RecordVariable}=1.");
        }

        private static readonly Dictionary<string, MercuryStateLayout> Loaded = new(StringComparer.Ordinal);
        private static readonly Dictionary<(string, int), MercuryStateLayout> Chosen = new();

        public static MercuryStateLayout Load(string path)
        {
            lock (Loaded)
            {
                if (!Loaded.TryGetValue(path, out MercuryStateLayout? layout))
                    Loaded[path] = layout = new MercuryStateLayout(Path.GetFileName(path), File.ReadAllText(path));
                return layout;
            }
        }

        public static MercuryStateLayout FromText(string name, string text) => new(name, text);

        // A listing written from an engine's state_layout, named for its board, console and RAM size; the commit that adds it says why.
        public static string Record(byte cartridgeType, string text)
        {
            var layout = new MercuryStateLayout("", text);
            Field ram = layout["Cart.Ram"];
            Field console = layout["Console"];
            Assert.True(console.Length == 1, "a listing without the console byte");
            string name = FileName(Board(cartridgeType), text.Contains("u8[16384] Bus.Vram", StringComparison.Ordinal), ram.Length);
            System.IO.Directory.CreateDirectory(Directory());
            string path = Path.Combine(Directory(), name);
            File.WriteAllText(path, text);
            lock (Loaded) Loaded.Remove(path);
            lock (Chosen) Chosen.Clear();
            return name;
        }

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
