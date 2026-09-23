using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.Cores.Nintendo.Mercury.Video;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MercuryRT reads the C# Mercury's state and writes back the same bytes, naming every field where the C# serializer does - see Mercury_Native.md §3.2.
    public class MercuryRtStateTests
    {
        // A directory of Game Boy images to run and round-trip; absent, that case passes without running.
        public const string RomsVariable = "EMUSEN_MERCURYRT_ROMS";

        // Enables cart RAM, keys a pulse note, then counts in WRAM, copies the count to cart RAM and scrolls by it forever.
        private static readonly byte[] Busy =
        {
            0x3E, 0x0A, 0xEA, 0x00, 0x00, 0x3E, 0xF0, 0xE0, 0x12, 0x3E, 0x87, 0xE0, 0x14,
            0x21, 0x00, 0xC0, 0x34, 0x7E, 0xEA, 0x00, 0xA0, 0xE0, 0x43, 0x18, 0xF4,
        };

        // Every board, with and without RAM, battery, clock and rumble, on both consoles.
        public static TheoryData<byte, byte, byte> Boards => new()
        {
            { 0x00, 0x00, 0x00 }, { 0x08, 0x02, 0x00 }, { 0x03, 0x02, 0x00 }, { 0x03, 0x03, 0x80 },
            { 0x06, 0x00, 0x00 }, { 0x06, 0x00, 0xC0 }, { 0x10, 0x03, 0x00 }, { 0x13, 0x03, 0xC0 },
            { 0x1B, 0x03, 0x80 }, { 0x1E, 0x04, 0x00 }, { 0x19, 0x00, 0xC0 },
        };

        private readonly ITestOutputHelper _output;

        public MercuryRtStateTests(ITestOutputHelper output) => _output = output;

        [Theory]
        [MemberData(nameof(Boards))]
        public void A_running_machines_state_comes_back_from_rust_byte_for_byte(byte kind, byte ramCode, byte cgb)
        {
            Assert.True(MercuryMachine.Available, MercuryNative.Report);
            byte[] rom = SyntheticGbRom.Build(romBanks: 4, cartridgeType: kind, ramSizeCode: ramCode, cgbFlag: cgb, patches: (0, Busy));
            MercuryCore core = Load(rom);
            for (int i = 0; i < 300; i++) core.RunFrame();
            Assert.True(core.Bus!.Wram[0] != 0 || core.TotalFrames == 300, "the machine never ran");

            byte[] state = Save(core);
            using var machine = new MercuryMachine(rom, null);
            machine.Load(state);
            AssertSameBytes(state, machine.Save());
            string layout = machine.Layout();
            AssertSameLayout(Layout(core, state), layout);
            _output.WriteLine($"type ${kind:X2}: {state.Length} bytes, {layout.Count(c => c == '\n')} fields compared by name, offset, length and type");
        }

        // One distinct value per field, so a field read into its neighbour's place shows; the save path non-ASCII and long enough for a two-byte length.
        [Theory]
        [MemberData(nameof(Boards))]
        public void Every_field_filled_with_noise_comes_back_from_rust_byte_for_byte(byte kind, byte ramCode, byte cgb)
        {
            Assert.True(MercuryMachine.Available, MercuryNative.Report);
            byte[] rom = SyntheticGbRom.Build(romBanks: 4, cartridgeType: kind, ramSizeCode: ramCode, cgbFlag: cgb);
            MercuryCore core = Load(rom);
            var noise = new Random(1989 + kind + cgb);
            var seen = new HashSet<object>(ReferenceEqualityComparer.Instance);
            Fill(core.Cart!, noise, seen);
            Fill(core.Cart!.Mapper, noise, seen);
            Fill(core.Cpu!, noise, seen);
            Fill(core.Bus!, noise, seen);
            typeof(Cartridge).GetField("_savePath", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(core.Cart, "/sävé/" + new string('é', 70) + ".srm");
            typeof(MercuryCore).GetField("_cyclesIntoFrame", BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(core, noise.NextInt64());

            byte[] state = Save(core);
            using var machine = new MercuryMachine(rom, null);
            machine.Load(state);
            AssertSameBytes(state, machine.Save());
            AssertSameLayout(Layout(core, state), machine.Layout());
        }

        // No C# writer makes these, so Rust is held to the C# reader's own reading of them, and so of the bytes it writes back.
        [Fact]
        public void Odd_bools_and_an_unnamed_mode_read_as_the_csharp_reader_reads_them()
        {
            Assert.True(MercuryMachine.Available, MercuryNative.Report);
            byte[] rom = SyntheticGbRom.Build(cartridgeType: 0x13, ramSizeCode: 0x03, cgbFlag: 0x80, patches: (0, Busy));
            MercuryCore core = Load(rom);
            for (int i = 0; i < 10; i++) core.RunFrame();
            byte[] state = Save(core);

            using var machine = new MercuryMachine(rom, null);
            machine.Load(state);
            string layout = machine.Layout();
            byte[] odd = (byte[])state.Clone();
            foreach (string field in new[] { "Cpu.Ime", "Bus.DoubleSpeed", "Bus.Ppu.StatLine", "Bus.Apu.Pulse1.HasSweep", "Mapper._cart" }) odd[OffsetOf(layout, field)] = 2;
            BitConverter.GetBytes(7).CopyTo(odd, OffsetOf(layout, "Bus.Ppu.Mode"));

            MercuryCore reader = Load(rom);
            reader.LoadState(new MemoryStream(odd));
            Assert.Equal((PpuMode)7, reader.Bus!.Ppu.Mode);
            byte[] csharp = Save(reader);
            Assert.False(csharp.AsSpan().SequenceEqual(state), "the patches changed nothing, so the test compared nothing");

            machine.Load(odd);
            AssertSameBytes(csharp, machine.Save());
        }

        [Fact]
        public void A_state_that_is_not_mercurys_or_is_cut_short_is_refused_and_changes_nothing()
        {
            Assert.True(MercuryMachine.Available, MercuryNative.Report);
            byte[] rom = SyntheticGbRom.Build(cartridgeType: 0x03, ramSizeCode: 0x02);
            byte[] state = Save(Load(rom));

            using var machine = new MercuryMachine(rom, null);
            machine.Load(state);
            Assert.Throws<InvalidDataException>(() => machine.Load(state.AsSpan(0, state.Length - 1)));
            Assert.Throws<InvalidDataException>(() => machine.Load(new byte[] { 0x4D, 0x41, 0x52, 0x54, 5, 0, 0, 0 }));
            byte[] version = (byte[])state.Clone();
            version[4] = 4;
            Assert.Throws<InvalidDataException>(() => machine.Load(version));
            AssertSameBytes(state, machine.Save());

            Assert.Throws<InvalidDataException>(() => new MercuryMachine(new byte[0x14F], null));
            Assert.Throws<NotSupportedException>(() => new MercuryMachine(SyntheticGbRom.Build(cartridgeType: 0x22), null));
        }

        // Real cartridges past their title screens, the Start and A presses of Mercury_Native.md §1.1's bench.
        [Fact]
        public void Real_games_states_come_back_from_rust_byte_for_byte()
        {
            string? folder = Environment.GetEnvironmentVariable(RomsVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{RomsVariable} unset, not run");
                return;
            }

            Assert.True(MercuryMachine.Available, MercuryNative.Report);
            foreach (string path in Directory.GetFiles(folder, "*.gb*").Order(StringComparer.Ordinal))
            {
                byte[] rom = File.ReadAllBytes(path);
                MercuryCore core = Load(rom);
                using var machine = new MercuryMachine(rom, null);
                for (int frame = 0; frame <= 900; frame++)
                {
                    int k = frame % 90;
                    if (k is 0 or 5) core.SetButton(0, PadButton.Start, k == 0);
                    if (k is 45 or 50) core.SetButton(0, PadButton.A, k == 45);
                    core.RunFrame();
                    if (frame % 300 != 0) continue;
                    byte[] state = Save(core);
                    machine.Load(state);
                    AssertSameBytes(state, machine.Save());
                    AssertSameLayout(Layout(core, state), machine.Layout());
                }
                _output.WriteLine($"{Path.GetFileName(path)}: identical at frames 0, 300, 600 and 900");
            }
        }

        private static MercuryCore Load(byte[] rom)
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticGbRom.WriteTemp(rom);
            try
            {
                var core = new MercuryCore { SkipRendering = true };
                core.LoadRom(path);
                return core;
            }
            finally
            {
                File.Delete(path);
            }
        }

        private static byte[] Save(MercuryCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        private static int OffsetOf(string layout, string path) =>
            int.Parse(layout.Split('\n').Single(line => line.EndsWith(" " + path, StringComparison.Ordinal)).Split(' ')[0]);

        private static void AssertSameBytes(byte[] expected, byte[] actual)
        {
            int length = Math.Min(expected.Length, actual.Length);
            int first = expected.AsSpan(0, length).CommonPrefixLength(actual.AsSpan(0, length));
            Assert.True(first == length && expected.Length == actual.Length,
                $"Rust wrote {actual.Length} bytes against C#'s {expected.Length}; the first difference is at {first}.");
        }

        private static void AssertSameLayout(string expected, string actual)
        {
            string[] want = expected.Split('\n', StringSplitOptions.RemoveEmptyEntries);
            string[] got = actual.Split('\n', StringSplitOptions.RemoveEmptyEntries);
            for (int i = 0; i < Math.Min(want.Length, got.Length); i++) Assert.True(want[i] == got[i], $"field {i}: C# \"{want[i]}\", Rust \"{got[i]}\"");
            Assert.Equal(want.Length, got.Length);
        }

        // The C# serializer's own walk as "offset length type path" lines, MercuryCore.SaveState's header written by hand.
        private static string Layout(MercuryCore core, byte[] state)
        {
            var layout = new LayoutWalk();
            layout.Line("Magic", "u32", 4);
            layout.Line("Version", "i32", 4);
            layout.Line("TotalFrames", "i64", 8);
            layout.Line("_cyclesIntoFrame", "i64", 8);
            layout.Walk("Cart.", core.Cart!);
            layout.Walk("Mapper.", core.Cart!.Mapper);
            layout.Walk("Cpu.", core.Cpu!);
            layout.Walk("Bus.", core.Bus!);
            Assert.Equal(state.Length, layout.Offset);
            return layout.Text;
        }

        private sealed class LayoutWalk
        {
            private readonly StringBuilder _text = new();

            public int Offset { get; private set; }

            public string Text => _text.ToString();

            public void Line(string path, string type, int length)
            {
                _text.Append(Offset).Append(' ').Append(length).Append(' ').Append(type).Append(' ').Append(path).Append('\n');
                Offset += length;
            }

            public void Walk(string prefix, object target)
            {
                foreach (FieldInfo field in StateFields(target.GetType()))
                {
                    Type t = field.FieldType;
                    object? value = field.GetValue(target);
                    string path = prefix + field.Name;

                    if (t == typeof(string)) Line(path, "string", StringLength((string?)value ?? ""));
                    else if (Primitive(t) is { } scalar) Line(path, scalar.Name, scalar.Size);
                    else if (t.IsArray && Primitive(t.GetElementType()!) is { } element)
                    {
                        int n = ((Array)value!).Length;
                        Line(path, $"{element.Name}[{n}]", element.Size * n);
                    }
                    else
                    {
                        Assert.False(t.IsArray || t.IsValueType, $"{path}: a {t.Name} field, which this walk does not model");
                        Line(path, "class", 1);
                        if (value != null) Walk(path + ".", value);
                    }
                }
            }

            private static int StringLength(string s)
            {
                int bytes = Encoding.UTF8.GetByteCount(s), prefix = 1;
                for (int n = bytes; n > 0x7F; n >>= 7) prefix++;
                return prefix + bytes;
            }

            private static (string Name, int Size)? Primitive(Type t)
            {
                if (t.IsEnum) return ("i32", 4);
                if (t == typeof(byte)) return ("u8", 1);
                if (t == typeof(bool)) return ("bool", 1);
                if (t == typeof(ushort)) return ("u16", 2);
                if (t == typeof(int)) return ("i32", 4);
                if (t == typeof(long)) return ("i64", 8);
                return null;
            }
        }

        private static IEnumerable<FieldInfo> StateFields(Type type) => type
            .GetFields(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)
            .Where(f => f.GetCustomAttribute<SkipInStateAttribute>() == null)
            .OrderBy(f => f.Name, StringComparer.Ordinal);

        private static void Fill(object target, Random noise, HashSet<object> seen)
        {
            if (!seen.Add(target)) return;
            foreach (FieldInfo field in StateFields(target.GetType()))
            {
                Type t = field.FieldType;
                object? value = field.GetValue(target);
                if (value is byte[] bytes) noise.NextBytes(bytes);
                else if (Random(t, noise) is { } scalar) field.SetValue(target, scalar);
                else if (value != null && t != typeof(string)) Fill(value, noise, seen);
            }
        }

        private static object? Random(Type t, Random noise)
        {
            if (t == typeof(bool)) return noise.Next(2) == 1;
            if (t == typeof(byte)) return (byte)noise.Next(256);
            if (t == typeof(ushort)) return (ushort)noise.Next();
            if (t == typeof(int)) return noise.Next();
            if (t == typeof(long)) return noise.NextInt64();
            if (t.IsEnum) return Enum.ToObject(t, noise.Next(4));
            return null;
        }
    }
}
