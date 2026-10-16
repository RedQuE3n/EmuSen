using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Cpu.Core;
using EmuSen.Galaxia.Input;

namespace EmuSen.MercuryLegacy
{
    // C# Mercury and a MercuryRT library side by side, compared after every frame or instruction: state, picture, sound and serial - see Mercury_Native.md §14.7.
    public sealed unsafe class Differential : IDisposable
    {
        // The MercuryRT library to compare against, built from main; a test fails when it is unset.
        public const string LibraryVariable = "EMUSEN_MERCURYRT_LIBRARY";

        // What couples this project to main, each checked by Pins before anything is compared.
        public const int StateVersion = 11;
        public const uint AbiMajor = 1;
        public static readonly string[] Extensions =
            { "mercuryrt_core_set_sample_rate", "mercuryrt_core_rom_patch", "mercuryrt_core_step", "mercuryrt_core_tick", "mercuryrt_core_stop", "mercuryrt_core_serial" };

        // The order of a mask's bits, MercuryRT's and the goldens' scripts'.
        public static readonly PadButton[] MaskOrder = { PadButton.Right, PadButton.Left, PadButton.Up, PadButton.Down, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        public static CoreLibrary Library()
        {
            string? path = Environment.GetEnvironmentVariable(LibraryVariable);
            Assert.True(path is { Length: > 0 } && File.Exists(path), $"{LibraryVariable} must name a MercuryRT library built from main.");
            CoreLibrary library = CoreLibrary.Open(path!);
            Assert.True(library.Available, library.Report);
            return library;
        }

        public readonly MercuryCore Csharp = new();
        public readonly CoreEngine Rust;
        private readonly CoreLibrary _library;
        private readonly string _rom;
        public int Frames { get; private set; }
        public long Samples { get; private set; }

        public Differential(byte[] image, GbModel model = GbModel.Auto, bool skipRendering = false)
        {
            CoreOptions.BatteryRamDisabled = true;
            _library = Library();
            _rom = Path.Combine(Path.GetTempPath(), $"mercury_legacy_{Guid.NewGuid():N}.gb");
            File.WriteAllBytes(_rom, image);
            Csharp.Model = model;
            Csharp.LoadRom(_rom);
            Csharp.SkipRendering = skipRendering;
            Rust = new CoreEngine(_library, new Dictionary<string, string> { [MercuryCore.ModelKey] = MercuryCore.ModelName(model) });
            Rust.LoadRom(_rom);
            Rust.SkipRendering = skipRendering;
            // MercuryRtCore.Loaded's coefficients: the division and Math.Pow evaluated in C#, so both mixers use the same doubles.
            double cyclesPerSample = 4194304.0 / 44100;
            Call<double, double>("mercuryrt_core_set_sample_rate", cyclesPerSample, Math.Pow(EmuSen.Cores.Nintendo.Mercury.Audio.Apu.HighPassSeed, cyclesPerSample));
            CompareState("load");
        }

        public void Dispose()
        {
            Rust.Dispose();
            try { File.Delete(_rom); } catch (IOException) { }
        }

        private int Call(string name)
        {
            lock (Rust.Machine.Gate) return ((delegate* unmanaged<nint, int>)_library.Export(name))(Rust.Machine.Handle);
        }

        private int Call<T1, T2>(string name, T1 a, T2 b) where T1 : unmanaged where T2 : unmanaged
        {
            lock (Rust.Machine.Gate) return ((delegate* unmanaged<nint, T1, T2, int>)_library.Export(name))(Rust.Machine.Handle, a, b);
        }

        public byte[] RustSerial()
        {
            lock (Rust.Machine.Gate)
            {
                var serial = (delegate* unmanaged<nint, byte*, nuint, long>)_library.Export("mercuryrt_core_serial");
                long n = serial(Rust.Machine.Handle, null, 0);
                var log = new byte[n];
                fixed (byte* data = log) serial(Rust.Machine.Handle, data, (nuint)log.Length);
                return log;
            }
        }

        public void Press(uint mask)
        {
            for (int b = 0; b < MaskOrder.Length; b++)
            {
                Csharp.SetButton(0, MaskOrder[b], (mask & (1u << b)) != 0);
                Rust.SetButton(0, MaskOrder[b], (mask & (1u << b)) != 0);
            }
        }

        // One frame on each; false when both stopped on the same illegal opcode.
        public bool Frame(bool soundAndPicture = true)
        {
            string? a = Try(Csharp.RunFrame), b = Try(Rust.RunFrame);
            Assert.True((a is null) == (b is null), $"frame {Frames + 1}: C# {a ?? "ran"}, MercuryRT {b ?? "ran"}");
            Frames++;
            Assert.True(Csharp.Bus!.SerialLog.SequenceEqual(RustSerial()), $"frame {Frames}: the serial logs differ");
            if (soundAndPicture)
            {
                short[] x = Csharp.DequeueAudioSamples(int.MaxValue), y = Rust.DequeueAudioSamples(int.MaxValue);
                int same = x.AsSpan().CommonPrefixLength(y);
                Assert.True(same == x.Length && x.Length == y.Length, $"frame {Frames}: {x.Length} C# samples, {y.Length} MercuryRT, first difference at {same}");
                Samples += x.Length;
                int pixel = Csharp.GetFrameBufferRgba().AsSpan().CommonPrefixLength(Rust.GetFrameBufferRgba());
                Assert.True(pixel == 160 * 144 * 4, $"frame {Frames}: the pictures differ first at byte {pixel}");
            }
            CompareState($"frame {Frames}");
            return a is null;
        }

        private static string? Try(Action run)
        {
            try { run(); return null; }
            catch (NotSupportedException e) { return e.GetType().Name; }
            catch (InvalidOperationException e) { return e.GetType().Name; }
        }

        // One instruction on each, with the frame loop's acknowledge and stall; false when both stopped on the same illegal opcode.
        public bool Step()
        {
            bool illegal = false;
            try
            {
                MemoryBusStep();
            }
            catch (NotSupportedException) { illegal = true; }
            int rust = Call("mercuryrt_core_step");
            Assert.True(illegal == (rust == -20), $"C# {(illegal ? "stopped" : "stepped")}, MercuryRT returned {rust}");
            if (!illegal) CompareState($"pc ${Csharp.Cpu!.PC:X4}");
            return !illegal;
        }

        private void MemoryBusStep()
        {
            var bus = Csharp.Bus!;
            Csharp.Cpu!.Step(bus.InterruptEnable, bus.InterruptFlags, out int serviced);
            if (serviced >= 0) bus.InterruptFlags &= (byte)~(1 << serviced);
            int stall = bus.TakePendingStall();
            if (stall > 0) bus.Tick(stall);
        }

        // The machine without its CPU, and the speed switch, on both.
        public void Tick(int cycles)
        {
            Csharp.Bus!.Tick(cycles);
            lock (Rust.Machine.Gate) ((delegate* unmanaged<nint, int, int>)_library.Export("mercuryrt_core_tick"))(Rust.Machine.Handle, cycles);
            CompareState($"a tick of {cycles}");
        }

        public void Stop()
        {
            Csharp.Bus!.Stop();
            Call("mercuryrt_core_stop");
            CompareState("a stop");
        }

        public byte[] CsharpState()
        {
            using var stream = new MemoryStream();
            Csharp.SaveState(stream);
            return stream.ToArray();
        }

        public byte[] RustState()
        {
            using var stream = new MemoryStream();
            Rust.SaveState(stream);
            return stream.ToArray();
        }

        public void CompareState(string when)
        {
            byte[] want = CsharpState(), got = RustState();
            int first = want.AsSpan().CommonPrefixLength(got);
            if (first == want.Length && want.Length == got.Length) return;
            string field = Rust.Machine.Layout().Split('\n', StringSplitOptions.RemoveEmptyEntries).Select(l => l.Split(' ')).LastOrDefault(p => int.Parse(p[0]) <= first) is { } line ? line[^1] : "?";
            Assert.Fail($"{when}: the states differ first at byte {first}, in {field}");
        }

        // The repository's root, for the committed goldens the pins read.
        public static string Root([CallerFilePath] string here = "") => Path.GetDirectoryName(Path.GetDirectoryName(here)!)!;
    }
}
