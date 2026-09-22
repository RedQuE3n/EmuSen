using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT reads the C# Mars's state and writes back the same bytes, and names every field where the C# serializer does - see Mars_Native.md §5.1.
    [Collection("MarsStatics")]
    public class MarsNativeStateTests : IDisposable
    {
        // Where copies of real game states are looked for; absent, those cases pass without running.
        public const string StatesVariable = "EMUSEN_MARSRT_STATES";

        // A counter the processor keeps in RDRAM, so every frame changes the machine.
        private static readonly byte[] CountForever =
        {
            0x3C, 0x04, 0xA0, 0x10, 0x8C, 0x88, 0x00, 0x00, 0x25, 0x08, 0x00, 0x01,
            0xAC, 0x88, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFC, 0x00, 0x00, 0x00, 0x00,
        };

        private readonly ITestOutputHelper _output;
        private readonly string _rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, CountForever)));

        public MarsNativeStateTests(ITestOutputHelper output) => _output = output;

        public void Dispose()
        {
            try { File.Delete(_rom); } catch (IOException) { }
        }

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_running_machines_state_comes_back_from_rust_byte_for_byte(bool snapshot)
        {
            Assert.True(MarsMachine.Available, MarsNative.Report);
            var core = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
            core.LoadRom(_rom);
            core.Bus!.Save = new SaveChip(N64SaveType.Eeprom4k);
            core.Bus.Write32(MemoryMap.ViBase + VideoInterface.VerticalSync, 0x20D);
            core.Bus.Write32(MemoryMap.ViBase + VideoInterface.HorizontalSync, 0x40);
            for (int i = 0; i < 300; i++) core.RunFrame();

            byte[] state = Save(core, snapshot);
            Assert.True(core.Bus.Read32(0x0010_0000) > 10_000, "the machine never ran");
            _output.WriteLine($"{core.TotalFrames} frames, {core.Bus.Cycles} cycles, {core.Cpu!.Instructions} instructions, {state.Length} bytes");

            using var machine = new MarsMachine(MemoryBus.RdramSizeExpanded);
            machine.Load(state);
            Assert.Equal(MemoryBus.RdramSize, machine.RdramBytes);
            Assert.Equal(snapshot ? MarsMachine.SnapshotKind : MarsMachine.StateKind, machine.LoadedKind);
            AssertSameBytes(state, machine.Save(snapshot));
            string layout = machine.Layout(snapshot);
            AssertSameLayout(Layout(core, state, snapshot), layout);
            _output.WriteLine($"{layout.Count(c => c == '\n')} fields compared by name, offset, length and type");
        }

        // Noise in every serialized field, the save chip's device, three paks, stub registers and a transfer under way.
        [Theory]
        [InlineData(N64SaveType.Unknown, false)]
        [InlineData(N64SaveType.Eeprom4k, false)]
        [InlineData(N64SaveType.Eeprom16k, true)]
        [InlineData(N64SaveType.Sram256k, false)]
        [InlineData(N64SaveType.SramBanked768k, true)]
        [InlineData(N64SaveType.FlashRam, false)]
        [InlineData(N64SaveType.FlashRam, true)]
        [InlineData(N64SaveType.Sram1M, false)]
        public void Every_field_filled_with_noise_comes_back_from_rust_byte_for_byte(N64SaveType saveType, bool snapshot)
        {
            Assert.True(MarsMachine.Available, MarsNative.Report);
            var core = new MarsCore(expansionPak: saveType == N64SaveType.FlashRam, batteryRamDisabled: true) { SkipRendering = true };
            core.LoadRom(_rom);
            var noise = new Random(1996 + (int)saveType);
            MemoryBus bus = core.Bus!;
            Fill(core.Cpu!, noise, new HashSet<object>(ReferenceEqualityComparer.Instance));
            Fill(bus, noise, new HashSet<object>(ReferenceEqualityComparer.Instance));

            bus.Save = new SaveChip(saveType);
            if (((object?)bus.Save.Eeprom ?? (object?)bus.Save.Sram ?? bus.Save.Flash) is { } device) Fill(device, noise, new HashSet<object>(ReferenceEqualityComparer.Instance));
            for (int port = 0; port < 4; port++)
            {
                bus.Si.Controllers[port].Pak = port == 1 ? null : new ControllerPak(null);
                if (bus.Si.Controllers[port].Pak is { } pak) Fill(pak, noise, new HashSet<object>(ReferenceEqualityComparer.Instance));
            }

            var registers = (Dictionary<uint, uint>)typeof(MemoryBus).GetField("_registers", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(bus)!;
            for (int i = 0; i < 5; i++) registers[(uint)noise.Next() & 0x0FFF_FFFC] = (uint)noise.Next();
            bus.Si.Due = noise.NextInt64(1, long.MaxValue);
            bus.Si.PendingRead = noise.Next();

            byte[] state = Save(core, snapshot);

            using var machine = new MarsMachine(bus.Rdram.Length);
            machine.Load(state);
            AssertSameBytes(state, machine.Save(snapshot));
            AssertSameLayout(Layout(core, state, snapshot), machine.Layout(snapshot));
        }

        // The C# thread rarely stands with words unrun at a snapshot, so the tail is written by hand here.
        [Fact]
        public void A_snapshots_pending_display_processor_words_come_back_and_refuse_to_become_a_state()
        {
            Assert.True(MarsMachine.Available, MarsNative.Report);
            var core = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
            core.LoadRom(_rom);
            for (int i = 0; i < 5; i++) core.RunFrame();
            byte[] snapshot = Save(core, snapshot: true);
            byte[] idle = (byte[])snapshot.Clone();

            int tail = snapshot.Length - 4 - 8 * DpInterface.SnapshotWords;
            Assert.Equal(0, BitConverter.ToInt32(snapshot, tail));
            var noise = new Random(64);
            const int Words = 37;
            BitConverter.GetBytes(Words).CopyTo(snapshot, tail);
            for (int i = 0; i < Words; i++) BitConverter.GetBytes(noise.NextInt64()).CopyTo(snapshot, tail + 4 + 8 * i);

            using var machine = new MarsMachine(MemoryBus.RdramSize);
            machine.Load(snapshot);
            AssertSameBytes(snapshot, machine.Save(snapshot: true));
            Assert.Contains($"{tail + 4} {8 * Words} u64[{Words}] Bus.Dp.Pending.Words", machine.Layout(snapshot: true));
            Assert.Throws<InvalidDataException>(() => machine.Save(snapshot: false));

            // The C# reader seeks past the padding and so takes a snapshot cut short inside it; MarsRT refuses one - see Mars_Native.md §5.1.
            byte[] before = machine.Save(snapshot: true);
            core.LoadState(new MemoryStream(idle, 0, idle.Length - 8));
            Assert.Throws<InvalidDataException>(() => machine.Load(idle.AsSpan(0, idle.Length - 8)));

            BitConverter.GetBytes(DpInterface.SnapshotWords + 1).CopyTo(snapshot, tail);
            Assert.Throws<InvalidDataException>(() => machine.Load(snapshot));
            AssertSameBytes(before, machine.Save(snapshot: true));
        }

        // No C# writer makes such a byte, so this holds Rust to the C# reader's own reading of one, and so of the bytes it writes back.
        [Fact]
        public void A_bool_byte_that_is_neither_zero_nor_one_reads_as_the_csharp_reader_reads_it()
        {
            Assert.True(MarsMachine.Available, MarsNative.Report);
            var core = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
            core.LoadRom(_rom);
            for (int i = 0; i < 5; i++) core.RunFrame();
            core.Bus!.Si.Controllers[0].Pak!.Dirty = true;
            byte[] state = Save(core, snapshot: false);

            using var machine = new MarsMachine(MemoryBus.RdramSize);
            machine.Load(state);
            string layout = machine.Layout(snapshot: false);
            byte[] odd = (byte[])state.Clone();
            foreach (string field in new[] { "Cpu.InDelaySlot", "Bus.Vi._wasBlank", "Bus.Dp.Processor._edgeInvalid", "Bus.Si.Controllers[2].Present" }) odd[OffsetOf(layout, field)] = 2;

            var reader = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
            reader.LoadRom(_rom);
            reader.LoadState(new MemoryStream(odd));
            byte[] csharp = Save(reader, snapshot: false);
            Assert.False(csharp.AsSpan().SequenceEqual(state), "every patched bool was already true, so the test compared nothing");

            machine.Load(odd);
            AssertSameBytes(csharp, machine.Save(snapshot: false));
        }

        private static int OffsetOf(string layout, string path) =>
            int.Parse(layout.Split('\n').Single(line => line.EndsWith(" " + path, StringComparison.Ordinal)).Split(' ')[0]);

        [Theory]
        [InlineData("sm64.state")]
        [InlineData("oot.state")]
        [InlineData("ge-dam.state")]
        public void A_real_games_state_comes_back_from_rust_byte_for_byte(string name)
        {
            string? folder = Environment.GetEnvironmentVariable(StatesVariable);
            string path = Path.Combine(folder ?? "", name);
            if (folder is null || !File.Exists(path))
            {
                _output.WriteLine($"{name}: absent, not run");
                return;
            }

            Assert.True(MarsMachine.Available, MarsNative.Report);
            byte[] state = File.ReadAllBytes(path);
            using var machine = new MarsMachine(MemoryBus.RdramSize);
            machine.Load(state);
            bool snapshot = machine.LoadedKind == MarsMachine.SnapshotKind;
            AssertSameBytes(state, machine.Save(snapshot));
            _output.WriteLine($"{name}: {state.Length} bytes, version {machine.LoadedKind}, {machine.RdramBytes / (1024 * 1024)} MB, identical");
        }

        [Fact]
        public void A_state_that_is_not_marss_or_is_cut_short_is_refused_and_changes_nothing()
        {
            Assert.True(MarsMachine.Available, MarsNative.Report);
            var core = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
            core.LoadRom(_rom);
            byte[] state = Save(core, snapshot: false);

            using var machine = new MarsMachine(MemoryBus.RdramSize);
            machine.Load(state);
            Assert.Throws<InvalidDataException>(() => machine.Load(state.AsSpan(0, state.Length - 1)));
            Assert.Throws<InvalidDataException>(() => machine.Load(new byte[] { 0x4D, 0x41, 0x52, 0x54, 1, 0, 0, 0 }));
            byte[] wrongSize = (byte[])state.Clone();
            BitConverter.GetBytes(3 * 1024 * 1024).CopyTo(wrongSize, 8);
            Assert.Throws<InvalidDataException>(() => machine.Load(wrongSize));
            AssertSameBytes(state, machine.Save(snapshot: false));
            Assert.Throws<ArgumentOutOfRangeException>(() => new MarsMachine(1234));
        }

        private static byte[] Save(MarsCore core, bool snapshot)
        {
            using var stream = new MemoryStream();
            if (snapshot) core.SaveSnapshot(stream);
            else core.SaveState(stream);
            return stream.ToArray();
        }

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
            for (int i = 0; i < Math.Min(want.Length, got.Length); i++)
            {
                Assert.True(want[i] == got[i], $"field {i}: C# \"{want[i]}\", Rust \"{got[i]}\"");
            }
            Assert.Equal(want.Length, got.Length);
        }

        // The C# serializer's own walk as "offset length type path" lines, the counts of the two hand-written lists read from the bytes.
        private static string Layout(MarsCore core, byte[] state, bool snapshot)
        {
            var layout = new LayoutWalk();
            layout.Line("Magic", "u32", 4);
            layout.Line("Version", "i32", 4);
            layout.Line("Rdram.Length", "i32", 4);
            layout.Line("TotalFrames", "i64", 8);
            layout.Line("_lastFrameCycles", "i64", 8);
            layout.Walk("Cpu.", core.Cpu!);
            MemoryBus bus = core.Bus!;
            layout.Walk("Bus.", bus);

            int registers = BitConverter.ToInt32(state, layout.Offset);
            layout.Line("Bus._registers.Count", "i32", 4);
            for (int i = 0; i < registers; i++)
            {
                layout.Line($"Bus._registers[{i}].Key", "u32", 4);
                layout.Line($"Bus._registers[{i}].Value", "u32", 4);
            }

            layout.Line("Bus.Save.Type", "i32", 4);
            if (bus.Save.Eeprom is { } eeprom) layout.Walk("Bus.Save.Eeprom.", eeprom);
            if (bus.Save.Sram is { } sram) layout.Walk("Bus.Save.Sram.", sram);
            if (bus.Save.Flash is { } flash) layout.Walk("Bus.Save.Flash.", flash);

            for (int i = 0; i < bus.Si.Controllers.Length; i++)
            {
                layout.Line($"Bus.Si.Controllers[{i}].Pak", "bool", 1);
                if (bus.Si.Controllers[i].Pak is { } pak) layout.Walk($"Bus.Si.Controllers[{i}].Pak.", pak);
            }

            if (snapshot)
            {
                int pending = BitConverter.ToInt32(state, layout.Offset);
                layout.Line("Bus.Dp.Pending.Count", "i32", 4);
                layout.Line("Bus.Dp.Pending.Words", $"u64[{pending}]", 8 * pending);
                layout.Line("Bus.Dp.Pending.Padding", $"u64[{DpInterface.SnapshotWords - pending}]", 8 * (DpInterface.SnapshotWords - pending));
            }

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
                    else if (t.IsArray)
                    {
                        var array = (Array)value!;
                        for (int i = 0; i < array.Length; i++) Walk($"{path}[{i}].", array.GetValue(i)!);
                    }
                    else if (t.IsValueType) Walk(path + ".", value!);
                    else
                    {
                        Line(path, "class", 1);
                        if (value != null) Walk(path + ".", value);
                    }
                }
            }

            private static (string Name, int Size)? Primitive(Type t)
            {
                if (t.IsEnum) return ("i32", 4);
                if (t == typeof(byte)) return ("u8", 1);
                if (t == typeof(sbyte)) return ("i8", 1);
                if (t == typeof(bool)) return ("bool", 1);
                if (t == typeof(short)) return ("i16", 2);
                if (t == typeof(ushort)) return ("u16", 2);
                if (t == typeof(int)) return ("i32", 4);
                if (t == typeof(uint)) return ("u32", 4);
                if (t == typeof(long)) return ("i64", 8);
                if (t == typeof(ulong)) return ("u64", 8);
                if (t == typeof(float)) return ("f32", 4);
                if (t == typeof(char)) return ("char", 2);
                if (t == typeof(string)) return ("string", 0);
                return null;
            }
        }

        private static IEnumerable<FieldInfo> StateFields(Type type) => type
            .GetFields(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)
            .Where(f => f.GetCustomAttribute<SkipInStateAttribute>() == null)
            .Where(f => f.GetCustomAttribute<AliasOfSerializedFieldAttribute>() == null)
            .OrderBy(f => f.Name, StringComparer.Ordinal);

        // MarsSaveStateTests' noise, one distinct value per field, so a field read into its neighbour's place shows.
        private static void Fill(object target, Random noise, HashSet<object> seen)
        {
            if (!target.GetType().IsValueType && !seen.Add(target)) return;

            foreach (FieldInfo field in StateFields(target.GetType()))
            {
                Type t = field.FieldType;
                object? value = field.GetValue(target);

                if (value is byte[] bytes) noise.NextBytes(bytes);
                else if (t.IsArray && value is Array array)
                {
                    Type element = t.GetElementType()!;
                    for (int i = 0; i < array.Length; i++)
                    {
                        if (Random(element, noise) is { } scalar) array.SetValue(scalar, i);
                        else if (array.GetValue(i) is { } item)
                        {
                            Fill(item, noise, seen);
                            if (element.IsValueType) array.SetValue(item, i);
                        }
                    }
                }
                else if (Random(t, noise) is { } scalar) field.SetValue(target, scalar);
                else if (value != null)
                {
                    Fill(value, noise, seen);
                    if (t.IsValueType) field.SetValue(target, value);
                }
            }
        }

        private static object? Random(Type t, Random noise)
        {
            if (t == typeof(bool)) return noise.Next(2) == 1;
            if (t == typeof(byte)) return (byte)noise.Next(256);
            if (t == typeof(sbyte)) return (sbyte)noise.Next(256);
            if (t == typeof(short)) return (short)noise.Next();
            if (t == typeof(ushort)) return (ushort)noise.Next();
            if (t == typeof(int)) return noise.Next();
            if (t == typeof(uint)) return (uint)noise.Next();
            if (t == typeof(long)) return noise.NextInt64();
            if (t == typeof(ulong)) return (ulong)noise.NextInt64();
            if (t == typeof(float)) return (float)noise.NextDouble();
            if (t == typeof(char)) return (char)noise.Next(0x20, 0x7F);
            if (t.IsEnum) return Enum.ToObject(t, noise.Next(4));
            return null;
        }
    }
}
