using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Native;
using Xunit.Abstractions;
using RdpProcessor = EmuSen.Cores.Nintendo.Mars.Rdp.Rdp;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT's display processor alone, through the test-only exports of ffi_rdp.rs - see Mars_Native.md §5.3.
    internal sealed unsafe class NativeRdp : IDisposable
    {
        private static readonly delegate* unmanaged<nint> New = (delegate* unmanaged<nint>)MarsNative.Export("mars_rdp_new");
        private static readonly delegate* unmanaged<nint, void> Free = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_rdp_free");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int> LoadState = (delegate* unmanaged<nint, byte*, nuint, int>)MarsNative.Export("mars_rdp_load_state");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SaveState = (delegate* unmanaged<nint, byte*, nuint, long>)MarsNative.Export("mars_rdp_save_state");
        private static readonly delegate* unmanaged<nint, byte*> TextureMemoryOf = (delegate* unmanaged<nint, byte*>)MarsNative.Export("mars_rdp_texture_memory");
        private static readonly delegate* unmanaged<nint, ulong*, nuint, byte*, nuint, byte*, nuint, uint*, long> AcceptWords =
            (delegate* unmanaged<nint, ulong*, nuint, byte*, nuint, byte*, nuint, uint*, long>)MarsNative.Export("mars_rdp_accept");

        private nint _handle;

        public static bool Available => New != null && AcceptWords != null;

        public NativeRdp()
        {
            if (!Available) throw new InvalidOperationException($"MarsRT's display processor is not in use: {MarsNative.Report}");
            _handle = New();
        }

        public void Load(byte[] state)
        {
            int status;
            fixed (byte* data = state) status = LoadState(_handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"MarsRT refused the display processor's state: status {status}.");
        }

        public byte[] Save()
        {
            long size = SaveState(_handle, null, 0);
            var state = new byte[size];
            fixed (byte* data = state) SaveState(_handle, data, (nuint)state.Length);
            return state;
        }

        public ReadOnlySpan<byte> TextureMemory => new(TextureMemoryOf(_handle), 0x1000);

        // Runs words up to and including the first full sync; returns how many it took.
        public int Accept(ReadOnlySpan<ulong> words, byte[] rdram, byte[] hidden, out bool sync)
        {
            uint synced;
            long taken;
            fixed (ulong* w = words)
            fixed (byte* r = rdram)
            fixed (byte* h = hidden)
                taken = AcceptWords(_handle, w, (nuint)words.Length, r, (nuint)rdram.Length, h, (nuint)hidden.Length, &synced);
            sync = synced != 0;
            return (int)taken;
        }

        public void Dispose()
        {
            if (_handle != 0) Free(_handle);
            _handle = 0;
        }
    }

    // The C# processor's state as the serializer writes it, and where the two processors first differ.
    internal static class RdpCompare
    {
        public static byte[] StateOf(RdpProcessor rdp)
        {
            using var stream = new MemoryStream();
            using (var w = new BinaryWriter(stream, Encoding.UTF8, leaveOpen: true)) StateSerializer.Write(w, rdp);
            return stream.ToArray();
        }

        public static void Load(RdpProcessor rdp, byte[] state)
        {
            using var r = new BinaryReader(new MemoryStream(state), Encoding.UTF8);
            StateSerializer.Read(r, rdp);
            rdp.Refresh();
        }

        public static int FirstDifference(ReadOnlySpan<byte> a, ReadOnlySpan<byte> b)
        {
            int length = Math.Min(a.Length, b.Length);
            int at = a[..length].CommonPrefixLength(b[..length]);
            return at < length || a.Length != b.Length ? at : -1;
        }

        public static string? Difference(RdpProcessor cs, byte[] csRdram, byte[] csHidden, NativeRdp rust, byte[] rustRdram, byte[] rustHidden)
        {
            int at = FirstDifference(csRdram, rustRdram);
            if (at >= 0) return $"RDRAM first differs at {at:X6}: C# {csRdram[at]:X2}, Rust {rustRdram[at]:X2}";
            at = FirstDifference(csHidden, rustHidden);
            if (at >= 0) return $"hidden RDRAM first differs at {at:X6}: C# {csHidden[at]:X2}, Rust {rustHidden[at]:X2}";
            at = FirstDifference(cs.TextureMemory, rust.TextureMemory);
            if (at >= 0) return $"texture memory first differs at {at:X3}";
            byte[] a = StateOf(cs), b = rust.Save();
            at = FirstDifference(a, b);
            if (at >= 0) return $"state first differs at byte {at} of {a.Length} (Rust {b.Length}), in {FieldAt(at)}";
            return null;
        }

        // The serialized field a byte offset falls in, walked as the serializer walks the processor.
        public static string FieldAt(int offset)
        {
            int at = 0;
            string? found = null;
            Walk(typeof(RdpProcessor), "", ref at, offset, ref found);
            return found ?? "no field";
        }

        private static void Walk(Type type, string path, ref int at, int offset, ref string? found)
        {
            var fields = type.GetFields(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)
                .Where(f => f.GetCustomAttribute<SkipInStateAttribute>() == null && f.GetCustomAttribute<AliasOfSerializedFieldAttribute>() == null)
                .OrderBy(f => f.Name, StringComparer.Ordinal);
            var sample = type == typeof(RdpProcessor) ? (object)new RdpProcessor(new MemoryBus()) : Activator.CreateInstance(type)!;

            foreach (FieldInfo field in fields)
            {
                if (found != null) return;
                Type t = field.FieldType;
                string name = path + field.Name;
                if (t.IsArray && t.GetElementType()!.IsPrimitive)
                {
                    var array = (Array)field.GetValue(sample)!;
                    int size = array.Length * System.Runtime.InteropServices.Marshal.SizeOf(t.GetElementType() == typeof(bool) ? typeof(byte) : t.GetElementType()!);
                    if (offset < at + size) found = $"{name}[{(offset - at) / (size / array.Length)}]";
                    at += size;
                }
                else if (t.IsArray)
                {
                    int length = ((Array)field.GetValue(sample)!).Length;
                    for (int i = 0; i < length && found == null; i++) Walk(t.GetElementType()!, $"{name}[{i}].", ref at, offset, ref found);
                }
                else if (t.IsValueType && !t.IsPrimitive)
                {
                    Walk(t, name + ".", ref at, offset, ref found);
                }
                else
                {
                    int size = t == typeof(bool) ? 1 : System.Runtime.InteropServices.Marshal.SizeOf(t);
                    if (offset < at + size) found = name;
                    at += size;
                }
            }
        }
    }

    // MarsRT's processor run beside a bus's own on every word the bus runs inline, from the C# processor's state at the first word.
    internal sealed class RdpTwin : DpInterface.IWordWatcher, IDisposable
    {
        private readonly MemoryBus _bus;
        private readonly NativeRdp _rust = new();
        private readonly byte[] _rdram, _hidden;
        private bool _attached, _rustSync;

        public int Words, Syncs;
        public readonly List<string> Differences = new();

        public RdpTwin(MemoryBus bus)
        {
            _bus = bus;
            _rdram = new byte[bus.Rdram.Length];
            _hidden = new byte[bus.RdramHidden.Length];
            bus.Dp.Watcher = this;
        }

        // The CPU's writes since the last word reach the twin's memory; its own state is never copied after the first word.
        public void Before(ulong word)
        {
            if (!_attached) _rust.Load(RdpCompare.StateOf(_bus.Dp.Processor));
            _attached = true;
            _bus.Rdram.CopyTo(_rdram, 0);
            _bus.RdramHidden.CopyTo(_hidden, 0);
            _rust.Accept(new[] { word }, _rdram, _hidden, out _rustSync);
        }

        public void After(ulong word, bool fullSync)
        {
            Words++;
            if (fullSync) Syncs++;
            if (Differences.Count >= 5) return;
            string? difference = fullSync != _rustSync
                ? $"C# {(fullSync ? "synced" : "did not sync")}, Rust {(_rustSync ? "synced" : "did not")}"
                : RdpCompare.Difference(_bus.Dp.Processor, _bus.Rdram, _bus.RdramHidden, _rust, _rdram, _hidden);
            if (difference != null) Differences.Add($"word {Words} ({word:X16}): {difference}");
        }

        public void Dispose() => _rust.Dispose();
    }

    // Every MarsRdpTests case again, each bus's list also run through MarsRT and compared after every word - see Mars_Native.md §5.3.
    public class MarsNativeRdpTwinTests : MarsRdpTests, IDisposable
    {
        private readonly List<RdpTwin> _twins = new();
        private readonly ITestOutputHelper _output;

        public MarsNativeRdpTwinTests(ITestOutputHelper output) => _output = output;

        protected override MemoryBus NewBus()
        {
            Assert.True(NativeRdp.Available, MarsNative.Report);
            var bus = new MemoryBus();
            _twins.Add(new RdpTwin(bus));
            return bus;
        }

        public void Dispose()
        {
            string[] differences = _twins.SelectMany(t => t.Differences).ToArray();
            _output.WriteLine($"{_twins.Count} buses, {_twins.Sum(t => t.Words)} words and {_twins.Sum(t => t.Syncs)} full syncs compared after every word");
            foreach (RdpTwin twin in _twins) twin.Dispose();
            Assert.True(differences.Length == 0, string.Join("\n", differences));
        }
    }

    // A command stream recorded from a game: the memories and the processor's state before it, every word, and where each frame ended.
    internal sealed class RdpStream
    {
        private const uint Magic = 0x5344_5052;

        public byte[] Rdram = Array.Empty<byte>(), Hidden = Array.Empty<byte>(), State = Array.Empty<byte>();
        public int[] FrameEnds = Array.Empty<int>();
        public ulong[] Words = Array.Empty<ulong>();

        public void Write(string path)
        {
            using var w = new BinaryWriter(File.Create(path));
            w.Write(Magic);
            w.Write(Rdram.Length);
            w.Write(Hidden.Length);
            w.Write(State.Length);
            w.Write(FrameEnds.Length);
            w.Write(Words.Length);
            w.Write(Rdram);
            w.Write(Hidden);
            w.Write(State);
            foreach (int end in FrameEnds) w.Write(end);
            foreach (ulong word in Words) w.Write(word);
        }

        public static RdpStream Read(string path)
        {
            using var r = new BinaryReader(File.OpenRead(path));
            if (r.ReadUInt32() != Magic) throw new InvalidDataException($"{path} is not a display processor stream");
            int rdram = r.ReadInt32(), hidden = r.ReadInt32(), state = r.ReadInt32(), frames = r.ReadInt32(), words = r.ReadInt32();
            var stream = new RdpStream { Rdram = r.ReadBytes(rdram), Hidden = r.ReadBytes(hidden), State = r.ReadBytes(state) };
            stream.FrameEnds = new int[frames];
            for (int i = 0; i < frames; i++) stream.FrameEnds[i] = r.ReadInt32();
            stream.Words = new ulong[words];
            for (int i = 0; i < words; i++) stream.Words[i] = r.ReadUInt64();
            return stream;
        }

        private sealed class Recorder : DpInterface.IWordWatcher
        {
            public readonly List<ulong> Words = new();

            public void Before(ulong word) => Words.Add(word);

            public void After(ulong word, bool fullSync) { }
        }

        // The game run from its state with the list inline and one processor, as ThreadedRdp off and RdpWorkers 1 run it.
        public static RdpStream Record(string rom, string state, int frames)
        {
            var core = new MarsCore(expansionPak: true, batteryRamDisabled: true) { SkipRendering = true };
            core.LoadRom(rom);
            core.LoadState(state);
            core.ThreadedRdp = false;
            core.RdpWorkers = 1;
            MemoryBus bus = core.Bus!;
            Assert.False(bus.Dp.Threaded);

            var stream = new RdpStream { Rdram = (byte[])bus.Rdram.Clone(), Hidden = (byte[])bus.RdramHidden.Clone(), State = RdpCompare.StateOf(bus.Dp.Processor) };
            var recorder = new Recorder();
            bus.Dp.Watcher = recorder;
            var ends = new List<int>();
            for (int i = 0; i < frames; i++)
            {
                core.RunFrame();
                ends.Add(recorder.Words.Count);
            }
            bus.Dp.Watcher = null;

            stream.FrameEnds = ends.ToArray();
            stream.Words = recorder.Words.ToArray();
            return stream;
        }
    }

    // What a stream exercises, counted by decoding its commands the way the processor gathers them.
    internal sealed class RdpCensus
    {
        public readonly SortedDictionary<string, int> Counts = new(StringComparer.Ordinal);

        public RdpCensus(ulong[] words)
        {
            ulong modes = 0;
            var tiles = new (int Format, int Size)[8];
            string[] cycles = { "1-cycle", "2-cycle", "copy", "fill" };
            string[] formats = { "RGBA", "YUV", "CI", "IA", "I", "5", "6", "7" };

            for (int i = 0; i < words.Length;)
            {
                ulong word = words[i];
                uint id = RdpProcessor.Id(word);
                int length = RdpProcessor.Length(id);
                if (i + length > words.Length) break;
                i += length;

                string cycle = cycles[(int)(modes >> 52) & 3];
                bool depth = ((modes >> 4) & 1) != 0 || ((modes >> 5) & 1) != 0;
                switch (id)
                {
                    case >= 0x08 and <= 0x0F:
                        Add($"triangle {cycle}{((id & 4) != 0 ? " shaded" : "")}{((id & 2) != 0 ? " textured" : "")}{((id & 1) != 0 ? " z" : "")}");
                        if (depth && (id & 1) != 0) Add("depth-tested or depth-written triangles");
                        if ((id & 2) != 0) Texture((int)(word >> 48) & 7);
                        break;
                    case 0x24 or 0x25:
                        Add($"texture rectangle {cycle}{(id == 0x25 ? " flipped" : "")}");
                        Texture((int)(word >> 24) & 7);
                        break;
                    case 0x36:
                        Add($"fill rectangle {cycle}");
                        break;
                    case 0x2F:
                        modes = word;
                        break;
                    case 0x35:
                        tiles[(word >> 24) & 7] = ((int)(word >> 53) & 7, (int)(word >> 51) & 3);
                        break;
                    case 0x30:
                        Add("load palette");
                        break;
                    case 0x33:
                        Add("load block");
                        break;
                    case 0x34:
                        Add("load tile");
                        break;
                    case 0x29:
                        Add("full sync");
                        break;
                }

                void Texture(int tile)
                {
                    (int format, int size) = tiles[tile];
                    Add($"texel format {formats[format]}{4 << size}{(((modes >> 47) & 1) != 0 ? " through a palette" : "")}{(((modes >> 43) & 1) != 0 ? ", filtered" : "")}");
                }
            }
        }

        private void Add(string key) => Counts[key] = Counts.GetValueOrDefault(key) + 1;
    }

    // MarsRT's display processor against the C# one on the command streams of three games - see Mars_Native.md §5.3.
    public class MarsNativeRdpTests
    {
        // A folder holding copies of the ROMs and states; absent, the game cases pass without running.
        public const string GamesVariable = "EMUSEN_MARSRT_RDP";

        private const int Frames = 120;

        private readonly ITestOutputHelper _output;

        public MarsNativeRdpTests(ITestOutputHelper output) => _output = output;

        [Theory]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void A_games_command_stream_draws_the_same_bytes_in_rust_at_every_full_sync(string rom, string state)
        {
            RdpStream? stream = Stream(rom, state);
            if (stream is null) return;

            var bus = new MemoryBus(expansionPak: stream.Rdram.Length > MemoryBus.RdramSize);
            stream.Rdram.CopyTo(bus.Rdram, 0);
            stream.Hidden.CopyTo(bus.RdramHidden, 0);
            RdpProcessor cs = bus.Dp.Processor;
            RdpCompare.Load(cs, stream.State);
            Assert.Equal(stream.State, RdpCompare.StateOf(cs));

            using var rust = new NativeRdp();
            rust.Load(stream.State);
            byte[] rdram = (byte[])stream.Rdram.Clone(), hidden = (byte[])stream.Hidden.Clone();
            Assert.Null(RdpCompare.Difference(cs, bus.Rdram, bus.RdramHidden, rust, rdram, hidden));

            ulong[] words = stream.Words;
            int position = 0, syncs = 0, compared = 0;
            for (int i = 0; i < words.Length; i++)
            {
                bool sync = cs.Accept(words[i]);
                if (!sync && i != words.Length - 1) continue;

                bool rustSync = false;
                while (position <= i)
                {
                    position += rust.Accept(words.AsSpan(position, i + 1 - position), rdram, hidden, out rustSync);
                    Assert.True(!rustSync || position == i + 1, $"Rust synced at word {position - 1}, C# at {i}");
                }

                Assert.True(sync == rustSync, $"word {i}: C# sync {sync}, Rust sync {rustSync}");
                if (sync) syncs++;
                compared++;
                string? difference = RdpCompare.Difference(cs, bus.Rdram, bus.RdramHidden, rust, rdram, hidden);
                Assert.True(difference is null, $"{state}, after word {i} of {words.Length} (frame {FrameOf(stream, i)}): {difference}");
            }

            WriteExpected(StreamPath(state) + ".expected", bus.Rdram, bus.RdramHidden, RdpCompare.StateOf(cs));
            int changed = CountChanged(stream.Rdram, bus.Rdram);
            _output.WriteLine($"{state}: {words.Length} words, {stream.FrameEnds.Length} frames, {syncs} full syncs, {compared} comparisons, all identical; {changed} RDRAM bytes drawn");
            foreach (var (key, count) in new RdpCensus(words).Counts) _output.WriteLine($"  {count,8} {key}");
        }

        // Each processor runs the whole stream from the same start, three rounds interleaved after one to warm up; no comparisons are timed.
        [Theory]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void Rust_and_csharp_run_a_games_command_stream_in_measured_time(string rom, string state)
        {
            RdpStream? stream = Stream(rom, state);
            if (stream is null) return;

            var bus = new MemoryBus(expansionPak: stream.Rdram.Length > MemoryBus.RdramSize);
            using var rust = new NativeRdp();
            byte[] rdram = new byte[stream.Rdram.Length], hidden = new byte[stream.Hidden.Length];

            double RunCSharp()
            {
                stream.Rdram.CopyTo(bus.Rdram, 0);
                stream.Hidden.CopyTo(bus.RdramHidden, 0);
                RdpProcessor cs = bus.Dp.Processor;
                RdpCompare.Load(cs, stream.State);
                ulong[] words = stream.Words;
                var watch = Stopwatch.StartNew();
                for (int i = 0; i < words.Length; i++) cs.Accept(words[i]);
                return watch.Elapsed.TotalMilliseconds;
            }

            double RunRust()
            {
                stream.Rdram.CopyTo(rdram, 0);
                stream.Hidden.CopyTo(hidden, 0);
                rust.Load(stream.State);
                ReadOnlySpan<ulong> words = stream.Words;
                var watch = Stopwatch.StartNew();
                for (int at = 0; at < words.Length;) at += rust.Accept(words[at..], rdram, hidden, out _);
                return watch.Elapsed.TotalMilliseconds;
            }

            RunCSharp();
            RunRust();
            var times = new List<(double CSharp, double Rust)>();
            for (int round = 0; round < 3; round++)
            {
                double c = RunCSharp();
                double r = RunRust();
                times.Add((c, r));
                _output.WriteLine($"{state} round {round + 1}: C# {c:F1} ms, Rust {r:F1} ms, ratio {c / r:F2}");
            }

            Assert.Null(RdpCompare.Difference(bus.Dp.Processor, bus.Rdram, bus.RdramHidden, rust, rdram, hidden));
            double perFrameC = times.Average(t => t.CSharp) / stream.FrameEnds.Length, perFrameR = times.Average(t => t.Rust) / stream.FrameEnds.Length;
            _output.WriteLine($"{state}: {stream.Words.Length} words over {stream.FrameEnds.Length} frames; mean C# {perFrameC:F3} ms/frame, Rust {perFrameR:F3} ms/frame");
        }

        // Random modes, tiles, loads and colours over well-formed primitives, both processors from one start, compared after each list - see Mars_Native.md §5.3.
        [Fact]
        public void Random_modes_over_well_formed_primitives_draw_the_same_bytes_in_rust()
        {
            Assert.True(NativeRdp.Available, MarsNative.Report);
            const int Lists = 3000;

            var bus = new MemoryBus();
            RdpProcessor cs = bus.Dp.Processor;
            var noise = new Random(64);
            noise.NextBytes(bus.Rdram.AsSpan(0x0010_0000, 0x0008_0000));
            noise.NextBytes(bus.Rdram.AsSpan(0x0020_0000, 0x0000_4000));
            noise.NextBytes(bus.RdramHidden.AsSpan(0x0008_0000, 0x0004_0000));

            using var rust = new NativeRdp();
            rust.Load(RdpCompare.StateOf(cs));
            byte[] rdram = (byte[])bus.Rdram.Clone(), hidden = (byte[])bus.RdramHidden.Clone();
            byte[] start = (byte[])rdram.Clone();

            long words = 0, primitives = 0, changed = 0;
            var cycles = new int[4];
            for (int list = 0; list < Lists; list++)
            {
                ulong[] commands = RandomList(new Random(1000 + list), cycles, ref primitives);
                int csSyncs = 0, rustSyncs = 0;
                foreach (ulong word in commands) if (cs.Accept(word)) csSyncs++;
                for (int at = 0; at < commands.Length;)
                {
                    at += rust.Accept(commands.AsSpan(at), rdram, hidden, out bool sync);
                    if (sync) rustSyncs++;
                }

                words += commands.Length;
                changed += CountChanged(start.AsSpan(0x0010_0000, 0x0008_0000), rdram.AsSpan(0x0010_0000, 0x0008_0000));
                rdram.AsSpan(0x0010_0000, 0x0008_0000).CopyTo(start.AsSpan(0x0010_0000));
                string? difference = csSyncs != rustSyncs ? $"C# {csSyncs} syncs, Rust {rustSyncs}" : RdpCompare.Difference(cs, bus.Rdram, bus.RdramHidden, rust, rdram, hidden);
                Assert.True(difference is null, $"list {list} (seed {1000 + list}): {difference}\n{string.Join(", ", commands.Select(w => w.ToString("X16")))}");
            }

            _output.WriteLine($"{Lists} lists, {words} words, {primitives} primitives ({cycles[0]} one-cycle, {cycles[1]} two-cycle, {cycles[2]} copy, {cycles[3]} fill), {changed} RDRAM bytes changed, all identical");
        }

        // The edges of five triangles the other tests draw, and the shade, depth and texture blocks two of them carry.
        private static readonly ulong[][] Edges =
        {
            new[] { 0x0A980075_001B0006UL, 0x001DC000_FFFEF777UL, 0x00022C65_00002735UL, 0xFFFFA186_00053CF4UL },
            new[] { 0x0D800071_0027000AUL, 0x001BC000_FFFEE7C9UL, 0x00032AE0_00002A41UL, 0x00018F73_0003611AUL },
            new[] { 0x0D000077_00460005UL, 0x0004C000_00019783UL, 0x001C898B_FFFFD9D3UL, 0x001CDD8A_FFFE89D9UL },
            new[] { 0x0A000070_00200008UL, 0x00040000_0000E666UL, 0x001A0000_FFFFD89EUL, 0x001A0000_FFFC5555UL },
            new[] { 0x08800068_00280008UL, 0x00140000_FFFF2000UL, 0x00040000_00001555UL, 0x00040000_00020000UL },
        };

        private static readonly ulong[] Shade =
        {
            0x00FE0013_00380103UL, 0xFFF80008_FFFFFFFDUL, 0x13E20775_B2F4B7ECUL, 0x3D02D62A_3BC17C71UL,
            0xFFF70001_0006FFF6UL, 0xFFF90000_0006FFF6UL, 0xD83CF116_9A199028UL, 0x202F7BB8_BA7DFA60UL,
        };

        private static readonly ulong[] Texture =
        {
            0xFFF3FF97_7FFF0000UL, 0x0046FFDF_FD270000UL, 0xE4531025_FFFF0000UL, 0xC1371E00_D82E0000UL,
            0x001800D1_FE9D0000UL, 0x000D00D6_FF0D0000UL, 0x375ADFB6_C31D0000UL, 0x6141E8F4_480F0000UL,
        };

        private static readonly ulong[] Depth = { 0x0F392A41_02F31F10UL, 0x018DAB7F_011109C9UL };

        private static ulong[] RandomList(Random r, int[] cycles, ref long primitives)
        {
            var w = new List<ulong>();
            ulong Bits(int bits) => (ulong)r.NextInt64() & ((1UL << bits) - 1);
            ulong Command(uint id, ulong operand) => ((ulong)id << 56) | (operand & 0x00FF_FFFF_FFFF_FFFFUL);
            ulong Block(ulong word) => r.Next(3) switch { 0 => word, 1 => word ^ Bits(16) ^ (Bits(16) << 32), _ => (ulong)r.NextInt64() ^ ((ulong)r.Next() << 63) };

            int size = new[] { 0, 1, 2, 2, 2, 3, 3 }[r.Next(7)];
            ulong width = (ulong)r.Next(16, 48);
            w.Add(Command(0x3F, (Bits(3) << 53) | ((ulong)size << 51) | ((width - 1) << 32) | (0x0010_0000UL + (ulong)r.Next(8))));
            w.Add(Command(0x3E, 0x0014_0000UL + (ulong)r.Next(4) * 2));
            ulong field = r.Next(8) == 0 ? (2UL | Bits(1)) << 24 : 0;
            w.Add(Command(0x2D, ((ulong)r.Next(0, 24) << 44) | ((ulong)r.Next(0, 24) << 32) | field | ((ulong)r.Next(80, (int)width * 4 + 24) << 12) | (ulong)r.Next(80, 170)));

            w.Add(Command(0x3D, (Bits(3) << 53) | ((ulong)r.Next(4) << 51) | ((ulong)r.Next(3, 40) << 32) | (0x0020_0000UL + (ulong)r.Next(0x200) * 8 + (ulong)r.Next(8))));
            for (int n = r.Next(1, 4); n > 0; n--)
            {
                w.Add(Command(0x35, (Bits(3) << 53) | (Bits(2) << 51) | ((ulong)r.Next(9) << 41) | (Bits(9) << 32) | (Bits(3) << 24) | Bits(24)));
                ulong sl = (ulong)r.Next(0, 24), tl = (ulong)r.Next(0, 24), tile = Bits(3);
                w.Add(r.Next(3) switch
                {
                    0 => Command(0x34, (sl << 44) | (tl << 32) | (tile << 24) | ((sl + (ulong)r.Next(0, 96)) << 12) | (tl + (ulong)r.Next(0, 64))),
                    1 => Command(0x33, (sl << 44) | (tl << 32) | (tile << 24) | ((ulong)r.Next(0, 0x200) << 12) | Bits(12)),
                    _ => Command(0x30, (sl << 44) | (tl << 32) | (tile << 24) | ((sl + (ulong)r.Next(0, 0x400)) << 12) | tl),
                });
            }

            for (int n = r.Next(1, 3); n > 0; n--)
            {
                ulong sl = (ulong)r.Next(0, 32), tl = (ulong)r.Next(0, 32);
                w.Add(Command(0x32, (sl << 44) | (tl << 32) | (Bits(3) << 24) | ((sl + (ulong)r.Next(0, 160)) << 12) | (tl + (ulong)r.Next(0, 160))));
            }

            w.Add(Command(0x3C, Bits(56)));
            w.Add(Command(0x3A, Bits(48)));
            w.Add(Command(0x3B, Bits(32)));
            w.Add(Command(0x39, Bits(32)));
            w.Add(Command(0x38, Bits(32)));
            w.Add(Command(0x2A, Bits(56)));
            w.Add(Command(0x2B, Bits(28)));
            w.Add(Command(0x2C, Bits(54)));
            w.Add(Command(0x2E, Bits(32)));
            w.Add(Command(0x37, Bits(32)));

            for (int n = r.Next(1, 5); n > 0; n--)
            {
                int cycle = new[] { 0, 0, 0, 1, 1, 1, 2, 3 }[r.Next(8)];
                ulong modes = (Bits(56) & ~(3UL << 52)) | ((ulong)cycle << 52);
                if (r.Next(4) == 0) w.Add(Command(0x3C, Bits(56)));

                // A keyed combine as a key is used, (A - centre) x scale, with widths small enough that distances land in the alpha's range.
                if (r.Next(3) == 0)
                {
                    modes = (modes | (1UL << 40)) & ~(1UL << 13);
                    ulong a = new ulong[] { 1, 2, 3, 4, 5 }[r.Next(5)];
                    ulong cleared = Bits(56) & ~((0xFUL << 37) | (0x1FUL << 32) | (0xFUL << 24) | (7UL << 6));
                    w.Add(Command(0x3C, cleared | (a << 37) | (6UL << 32) | (6UL << 24) | (7UL << 6)));
                    w.Add(Command(0x2A, ((ulong)r.Next(0x40) << 44) | ((ulong)r.Next(0x40) << 32) | (Bits(8) << 24) | ((ulong)r.Next(0x20) << 16) | (Bits(8) << 8) | (ulong)r.Next(0x20)));
                    w.Add(Command(0x2B, ((ulong)r.Next(0x40) << 16) | (Bits(8) << 8) | (ulong)r.Next(0x20)));
                }

                w.Add(Command(0x2F, modes));
                cycles[cycle]++;
                primitives++;

                switch (r.Next(6))
                {
                    case 0:
                        ulong x = (ulong)r.Next(0, 120), y = (ulong)r.Next(0, 120);
                        w.Add(Command(0x36, ((x + (ulong)r.Next(0, 80)) << 44) | ((y + (ulong)r.Next(0, 80)) << 32) | (x << 12) | y));
                        break;
                    case 1:
                        ulong left = (ulong)r.Next(0, 120), top = (ulong)r.Next(0, 120);
                        ulong steps = r.Next(3) == 0 ? Bits(32) : ((ulong)(ushort)r.Next(-0x2000, 0x2000) << 16) | (ushort)r.Next(-0x2000, 0x2000);
                        w.Add(Command((uint)(0x24 + r.Next(2)), ((left + (ulong)r.Next(0, 90)) << 44) | ((top + (ulong)r.Next(0, 90)) << 32) | (Bits(3) << 24) | (left << 12) | top));
                        w.Add((Bits(32) << 32) | steps);
                        break;
                    default:
                        ulong[] edges = Edges[r.Next(Edges.Length)];
                        uint id = (uint)(0x08 + r.Next(8));
                        ulong flags = r.Next(2) == 0 ? (edges[0] >> 48) & 0xFF : Bits(8);
                        w.Add(((ulong)id << 56) | (flags << 48) | (edges[0] & 0x0000_FFFF_FFFF_FFFFUL));
                        for (int k = 1; k < 4; k++) w.Add(r.Next(4) == 0 ? edges[k] ^ Bits(12) : edges[k]);
                        if ((id & 4) != 0) foreach (ulong word in Shade) w.Add(Block(word));
                        if ((id & 2) != 0) foreach (ulong word in Texture) w.Add(Block(word));
                        if ((id & 1) != 0) foreach (ulong word in Depth) w.Add(Block(word));
                        break;
                }
            }

            w.Add(0x27UL << 56);
            w.Add(0x29UL << 56);
            return w.ToArray();
        }

        private static string StreamPath(string state) => Path.Combine(Environment.GetEnvironmentVariable(GamesVariable) ?? "", state) + $".{Frames}.rdp";

        // The C# processor's memories and state at the stream's end, which MarsRT's cargo test replays against - see Mars_Native.md §5.3.
        private static void WriteExpected(string path, byte[] rdram, byte[] hidden, byte[] state)
        {
            using var w = new BinaryWriter(File.Create(path));
            w.Write(0x5844_5052u);
            w.Write(rdram.Length);
            w.Write(hidden.Length);
            w.Write(state.Length);
            w.Write(rdram);
            w.Write(hidden);
            w.Write(state);
        }

        private RdpStream? Stream(string rom, string state)
        {
            string? folder = Environment.GetEnvironmentVariable(GamesVariable);
            string romPath = Path.Combine(folder ?? "", rom), statePath = Path.Combine(folder ?? "", state), streamPath = StreamPath(state);
            if (folder is null || !File.Exists(streamPath) && (!File.Exists(romPath) || !File.Exists(statePath)))
            {
                _output.WriteLine($"{state}: absent, not run");
                return null;
            }

            Assert.True(NativeRdp.Available, MarsNative.Report);
            if (File.Exists(streamPath)) return RdpStream.Read(streamPath);

            var watch = Stopwatch.StartNew();
            RdpStream stream = RdpStream.Record(romPath, statePath, Frames);
            stream.Write(streamPath);
            _output.WriteLine($"{state}: recorded {stream.Words.Length} words over {Frames} frames in {watch.Elapsed.TotalSeconds:F1} s");
            return stream;
        }

        private static int FrameOf(RdpStream stream, int word)
        {
            int frame = Array.FindIndex(stream.FrameEnds, end => end > word);
            return frame < 0 ? stream.FrameEnds.Length : frame;
        }

        private static int CountChanged(ReadOnlySpan<byte> before, ReadOnlySpan<byte> after)
        {
            int changed = 0;
            for (int i = 0; i < before.Length; i++) if (before[i] != after[i]) changed++;
            return changed;
        }
    }
}
