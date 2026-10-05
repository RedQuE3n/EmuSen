using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MoonRT reads the C# Moon's state and writes back the same bytes, naming every field where the C# serializer does - see Moon_Native.md §3.2.
    public class MoonRtStateTests
    {
        // A directory of NES images to run and round-trip; absent, that case passes without running.
        public const string RomsVariable = "EMUSEN_MOONRT_ROMS";

        // Turns rendering and a pulse note on, then counts in RAM, copies the count to PRG RAM and scrolls by it forever.
        public static readonly byte[] Busy =
        {
            0x78, 0xA9, 0x1E, 0x8D, 0x01, 0x20, 0xA9, 0x01, 0x8D, 0x15, 0x40, 0xA9, 0xBF, 0x8D, 0x00, 0x40,
            0xA9, 0x40, 0x8D, 0x03, 0x40, 0xE6, 0x10, 0xA5, 0x10, 0x8D, 0x00, 0x60, 0x8D, 0x05, 0x20, 0x8D,
            0x05, 0x20, 0x4C, 0x15, 0x80,
        };

        // Every board Moon implements, with CHR ROM and with CHR RAM.
        public static TheoryData<int, int> Boards
        {
            get
            {
                var data = new TheoryData<int, int>();
                foreach (int mapper in new[] { 0, 1, 2, 3, 4, 7, 9, 11, 64, 65, 66, 67, 68, 69, 71, 79 })
                {
                    data.Add(mapper, 8);
                    data.Add(mapper, 0);
                }
                return data;
            }
        }

        private readonly ITestOutputHelper _output;

        public MoonRtStateTests(ITestOutputHelper output) => _output = output;

        public static byte[] Rom(int mapper, int chrBanks, params (int Offset, byte[] Bytes)[] patches) =>
            SyntheticNesRom.Build(prgBanks: 8, chrBanks: chrBanks, mapper: mapper, battery: true, patches: patches);

        [Theory]
        [MemberData(nameof(Boards))]
        public void A_running_machines_state_comes_back_from_rust_byte_for_byte(int mapper, int chrBanks)
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            byte[] rom = Rom(mapper, chrBanks, (0, Busy));
            MoonCore core = Load(rom);
            for (int i = 0; i < 300; i++) core.RunFrame();

            byte[] state = Save(core);
            using var machine = new MoonMachine(rom);
            machine.Load(state);
            AssertSameBytes(state, machine.Save());
            string layout = machine.Layout();
            AssertSameLayout(Layout(core, state), layout);
            _output.WriteLine($"mapper {mapper}, {chrBanks} CHR banks: {state.Length} bytes, {layout.Count(c => c == '\n')} fields compared by name, offset, length and type");
        }

        // One distinct value per field, so a field read into its neighbour's place shows.
        [Theory]
        [MemberData(nameof(Boards))]
        public void Every_field_filled_with_noise_comes_back_from_rust_byte_for_byte(int mapper, int chrBanks)
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            byte[] rom = Rom(mapper, chrBanks);
            MoonCore core = Load(rom);
            var noise = new Random(1983 + mapper * 7 + chrBanks);
            var seen = new HashSet<object>(ReferenceEqualityComparer.Instance);
            Fill(core.Cart!, noise, seen);
            Fill(core.Cart!.Mapper, noise, seen);
            Fill(core.Cpu!, noise, seen);
            Fill(core.Bus!, noise, seen);
            Fill(core.Ppu!, noise, seen);
            Fill(core.Apu!, noise, seen);
            foreach (string name in new[] { "_lineStartClock", "_cpuBudget", "_masterClock", "_cpuRemainder" })
                typeof(MoonCore).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.SetValue(core, noise.NextInt64());

            byte[] state = Save(core);
            using var machine = new MoonMachine(rom);
            machine.Load(state);
            AssertSameBytes(state, machine.Save());
            AssertSameLayout(Layout(core, state), machine.Layout());
        }

        // No C# writer makes these, so Rust is held to the C# reader's own reading of them, and so of the bytes it writes back.
        [Fact]
        public void Odd_bools_and_class_flags_read_as_the_csharp_reader_reads_them()
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            byte[] rom = Rom(4, 8, (0, Busy));
            MoonCore core = Load(rom);
            for (int i = 0; i < 10; i++) core.RunFrame();
            byte[] state = Save(core);

            using var machine = new MoonMachine(rom);
            machine.Load(state);
            string layout = machine.Layout();
            byte[] odd = (byte[])state.Clone();
            foreach (string field in new[] { "Cpu.Jammed", "Cpu._bus", "Ppu.WriteToggle", "Apu.Pulse1._onesComplement", "Apu.Dmc", "Mapper._a12", "Mapper._wramEnabled" }) odd[OffsetOf(layout, field)] = 2;

            MoonCore reader = Load(rom);
            reader.LoadState(new MemoryStream(odd));
            byte[] csharp = Save(reader);
            Assert.False(csharp.AsSpan().SequenceEqual(state), "the patches changed nothing, so the test compared nothing");

            machine.Load(odd);
            AssertSameBytes(csharp, machine.Save());
        }

        // A flag of 0 makes the C# reader leave the object as it was and read on, so every later field is read out of place.
        [Fact]
        public void A_class_flag_of_zero_is_skipped_as_the_csharp_reader_skips_it()
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            byte[] rom = Rom(4, 8, (0, Busy));
            MoonCore core = Load(rom);
            for (int i = 0; i < 10; i++) core.RunFrame();
            byte[] state = Save(core);

            using var machine = new MoonMachine(rom);
            machine.Load(state);
            string layout = machine.Layout();
            byte[] absent = (byte[])state.Clone();
            foreach (string field in new[] { "Cpu._bus", "Apu.Dmc" }) absent[OffsetOf(layout, field)] = 0;

            MoonCore reader = Load(rom);
            for (int i = 0; i < 10; i++) reader.RunFrame();
            reader.LoadState(new MemoryStream(absent));
            byte[] csharp = Save(reader);
            Assert.False(csharp.AsSpan().SequenceEqual(state), "the patches changed nothing, so the test compared nothing");

            machine.Load(absent);
            AssertSameBytes(csharp, machine.Save());
        }

        [Fact]
        public void A_state_that_is_not_moons_or_is_cut_short_is_refused_and_changes_nothing()
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            byte[] rom = Rom(1, 0);
            byte[] state = Save(Load(rom));

            using var machine = new MoonMachine(rom);
            machine.Load(state);
            Assert.Throws<InvalidDataException>(() => machine.Load(state.AsSpan(0, state.Length - 1)));
            MoonCore later = Load(rom);
            for (int i = 0; i < 10; i++) later.RunFrame();
            byte[] moved = Save(later);
            Assert.False(moved.AsSpan().SequenceEqual(state), "ten frames changed nothing, so a partial read would go unseen");
            Assert.Throws<InvalidDataException>(() => machine.Load(moved.AsSpan(0, moved.Length - 1)));
            AssertSameBytes(state, machine.Save());
            Assert.Throws<InvalidDataException>(() => machine.Load(new byte[] { 0x4D, 0x45, 0x52, 0x43, 3, 0, 0, 0 }));
            byte[] version = (byte[])state.Clone();
            version[4] = 2;
            Assert.Throws<InvalidDataException>(() => machine.Load(version));
            AssertSameBytes(state, machine.Save());

            Assert.Throws<InvalidDataException>(() => new MoonMachine(new byte[] { 0x4E, 0x45, 0x53 }));
            byte[] unmarked = SyntheticNesRom.Build(mapper: 0);
            unmarked[0] = (byte)'M';
            Exception csharp = Assert.ThrowsAny<Exception>(() => Load(unmarked));
            Assert.Equal(csharp.GetType(), Assert.ThrowsAny<Exception>(() => new MoonMachine(unmarked)).GetType());
            Assert.Throws<NotSupportedException>(() => new MoonMachine(SyntheticNesRom.Build(mapper: 5)));
        }

        // An image with no PRG throws from inside C#'s LoadRom, and MoonRT refuses it with the same exception type - Moon_Native.md §6.2, D4.
        [Theory]
        [InlineData(0)]
        [InlineData(1)]
        [InlineData(4)]
        public void An_image_without_prg_is_refused_with_the_csharp_exception(int mapper)
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            byte[] rom = SyntheticNesRom.Build(mapper: mapper);
            byte[] empty = rom.AsSpan(0, 16).ToArray().Concat(rom.AsSpan(16 + SyntheticNesRom.PrgBankSize).ToArray()).ToArray();
            empty[4] = 0;
            Exception csharp = Assert.ThrowsAny<Exception>(() => Load(empty));
            Exception rust = Assert.ThrowsAny<Exception>(() => new MoonMachine(empty));
            Assert.Equal(csharp.GetType(), rust.GetType());
        }

        // Version 3 has neither the DMA tail nor the mixer, version 4 no mixer, version 5 no RenderingSince; both engines load each alike and write it back as version 6.
        [Theory]
        [InlineData(0, 3)]
        [InlineData(4, 3)]
        [InlineData(0, 4)]
        [InlineData(4, 4)]
        [InlineData(0, 5)]
        [InlineData(4, 5)]
        public void An_older_state_loads_in_both_engines_alike(int mapper, int version)
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            byte[] rom = Rom(mapper, 8, (0, Busy));
            MoonCore core = Load(rom);
            for (int i = 0; i < 30; i++) core.RunFrame();
            byte[] v6 = Save(core);
            int cut = RenderingSinceBytes + (version <= 4 ? MixerBytes : 0) + (version == 3 ? DmaTailBytes : 0);
            byte[] old = v6.AsSpan(0, v6.Length - cut).ToArray();
            old[4] = (byte)version;

            MoonCore csharp = Load(rom);
            csharp.LoadState(new MemoryStream(old));
            using var machine = new MoonMachine(rom);
            machine.Load(old);
            byte[] expected = Save(csharp);
            Assert.Equal(6, BitConverter.ToInt32(expected, 4));
            AssertSameBytes(expected, machine.Save());
            for (int i = 0; i < 10; i++) csharp.RunFrame();
            for (int i = 0; i < 10; i++) machine.RunFrame();
            AssertSameBytes(Save(csharp), machine.Save());
        }

        // Version 4's tail after the walks, version 5's mixer after it (three doubles and an int, then five filter doubles), then version 6's RenderingSince.
        public const int DmaTailBytes = 8, MixerBytes = 8 + 4 + 8 + 5 * 8, RenderingSinceBytes = 8;

        // A machine loaded with another's state at frame 300 sounds exactly as the first from then on, in both engines - see Moon_Native.md §3.13.
        [Theory]
        [InlineData(0)]
        [InlineData(4)]
        public void A_state_loaded_into_a_second_machine_resumes_its_sound_exactly(int mapper)
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            byte[] rom = Rom(mapper, 8, (0, Busy));
            foreach (Func<ICore> make in new Func<ICore>[] { () => Load(rom), () => LoadRt(rom) })
            {
                ICore a = make();
                for (int f = 0; f < 300; f++) { Press(a, f); a.RunFrame(); a.DequeueAudioSamples(int.MaxValue); }
                var state = new MemoryStream();
                a.SaveState(state);
                ICore b = make();
                b.LoadState(new MemoryStream(state.ToArray()));
                for (int f = 300; f < 600; f++)
                {
                    Press(a, f);
                    Press(b, f);
                    a.RunFrame();
                    b.RunFrame();
                    Assert.Equal(a.DequeueAudioSamples(int.MaxValue), b.DequeueAudioSamples(int.MaxValue));
                }
                var sa = new MemoryStream();
                var sb = new MemoryStream();
                a.SaveState(sa);
                b.SaveState(sb);
                AssertSameBytes(sa.ToArray(), sb.ToArray());
                (a as IDisposable)?.Dispose();
                (b as IDisposable)?.Dispose();
            }
        }

        private static void Press(ICore core, int frame)
        {
            core.SetButton(0, PadButton.Start, frame % 90 < 5);
            core.SetButton(0, PadButton.A, frame % 90 is >= 45 and < 50);
        }

        private static ICore LoadRt(byte[] rom)
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticNesRom.WriteTemp(rom);
            try
            {
                var core = new MoonRtCore();
                core.LoadRom(path);
                return core;
            }
            finally { File.Delete(path); }
        }

        // Real cartridges past their title screens, with the Start and A presses of Moon_Native.md §1.1's bench.
        [Fact]
        public void Real_games_states_come_back_from_rust_byte_for_byte()
        {
            string? folder = Environment.GetEnvironmentVariable(RomsVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{RomsVariable} unset, not run");
                return;
            }

            Assert.True(MoonMachine.Available, MoonNative.Report);
            foreach (string path in Directory.GetFiles(folder, "*.nes").Order(StringComparer.Ordinal))
            {
                byte[] rom = File.ReadAllBytes(path);
                MoonCore core = Load(rom);
                using var machine = new MoonMachine(rom);
                for (int frame = 0; frame <= 900; frame++)
                {
                    int k = frame % 90;
                    core.SetButton(0, PadButton.Start, k < 5);
                    core.SetButton(0, PadButton.A, k >= 45 && k < 50);
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

        public static MoonCore Load(byte[] rom)
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticNesRom.WriteTemp(rom);
            try
            {
                var core = new MoonCore();
                core.LoadRom(path);
                return core;
            }
            finally
            {
                File.Delete(path);
            }
        }

        public static byte[] Save(MoonCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        private static int OffsetOf(string layout, string path) =>
            int.Parse(layout.Split('\n').Single(line => line.EndsWith(" " + path, StringComparison.Ordinal)).Split(' ')[0]);

        public static void AssertSameBytes(byte[] expected, byte[] actual)
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

        // The C# serializer's own walk as "offset length type path" lines, MoonCore.SaveState's header, version 4's tail, version 5's mixer and version 6's RenderingSince written by hand.
        public static string Layout(MoonCore core, byte[] state)
        {
            var layout = new LayoutWalk();
            layout.Line("Magic", "u32", 4);
            layout.Line("Version", "i32", 4);
            layout.Line("TotalFrames", "i64", 8);
            layout.Line("_lineStartClock", "i64", 8);
            layout.Line("_cpuBudget", "i64", 8);
            layout.Line("_masterClock", "i64", 8);
            layout.Line("_cpuRemainder", "i64", 8);
            layout.Walk("Cart.", core.Cart!);
            layout.Walk("Mapper.", core.Cart!.Mapper);
            layout.Walk("Cpu.", core.Cpu!);
            layout.Walk("Bus.", core.Bus!);
            layout.Walk("Ppu.", core.Ppu!);
            layout.Walk("Apu.", core.Apu!);
            layout.Line("Apu.Dmc.SampleBuffer", "u8", 1);
            layout.Line("Apu.Dmc.BufferFull", "bool", 1);
            layout.Line("Apu.Dmc.LoadDelay", "i32", 4);
            layout.Line("Bus.OamDmaPending", "bool", 1);
            layout.Line("Bus.OamDmaPage", "u8", 1);
            layout.Line("Apu._sampleAccumulator", "f64", 8);
            layout.Line("Apu._sampleCount", "i32", 4);
            layout.Line("Apu._cycleFraction", "f64", 8);
            foreach (string filter in new[] { "_hp90", "_hp90Prev", "_hp440", "_hp440Prev", "_lp14k" }) layout.Line("Apu." + filter, "f64", 8);
            layout.Line("Ppu.RenderingSince", "i64", 8);
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

                    if (Primitive(t) is { } scalar) Line(path, scalar.Name, scalar.Size);
                    else if (t.IsArray && Primitive(t.GetElementType()!) is { } element)
                    {
                        int n = ((Array)value!).Length;
                        Line(path, $"{element.Name}[{n}]", element.Size * n);
                    }
                    else
                    {
                        Assert.False(t.IsArray || t.IsValueType || t == typeof(string), $"{path}: a {t.Name} field, which this walk does not model");
                        Line(path, "class", 1);
                        if (value != null) Walk(path + ".", value);
                    }
                }
            }

            private static (string Name, int Size)? Primitive(Type t)
            {
                if (t.IsEnum) return ("i32", 4);
                if (t == typeof(byte)) return ("u8", 1);
                if (t == typeof(bool)) return ("bool", 1);
                if (t == typeof(ushort)) return ("u16", 2);
                if (t == typeof(int)) return ("i32", 4);
                if (t == typeof(long)) return ("i64", 8);
                if (t == typeof(ulong)) return ("u64", 8);
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
                else if (value is int[] ints) for (int i = 0; i < ints.Length; i++) ints[i] = noise.Next();
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
            if (t == typeof(ulong)) return (ulong)noise.NextInt64();
            if (t.IsEnum) return Enum.ToObject(t, noise.Next(5));
            return null;
        }
    }
}
