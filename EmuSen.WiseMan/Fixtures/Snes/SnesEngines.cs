using System.Diagnostics;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.VenusRT;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Fixtures.Snes
{
    // A picture as 15-bit BGR words, row by row - see VenusRT_Native.md §3.2.
    public sealed record SnesPicture(int Width, int Height, ushort[] Pixels)
    {
        public static SnesPicture FromRgba(byte[] rgba, int width, int height)
        {
            var px = new ushort[width * height];
            for (int i = 0; i < px.Length && i * 4 + 2 < rgba.Length; i++)
                px[i] = (ushort)((rgba[i * 4] >> 3) | (rgba[i * 4 + 1] >> 3) << 5 | (rgba[i * 4 + 2] >> 3) << 10);
            return new SnesPicture(width, height, px);
        }
    }

    // One engine's memories, by the probe's lower-case names, and its picture after a frame.
    public sealed record SnesSnapshot(int Frame, IReadOnlyDictionary<string, byte[]> Spaces, SnesPicture? Picture);

    // A run's snapshots in frame order, and its sound from power-on as interleaved stereo.
    public sealed record SnesRun(string Engine, IReadOnlyList<SnesSnapshot> Snapshots, int AudioRate, short[] Audio)
    {
        public SnesSnapshot? At(int frame) => Snapshots.FirstOrDefault(s => s.Frame == frame);
    }

    // Button held on port 0 from Frame for Frames frames, as the probe's --press counts them.
    public sealed record SnesPress(int Frame, PadButton Button, int Frames = 4);

    // Any SNES engine, run from power-on to each of the frames asked for - see VenusRT_Native.md §3.
    public interface ISnesEngine
    {
        string Name { get; }
        SnesRun Run(string rom, IReadOnlyList<int> frames, IReadOnlyList<SnesPress>? presses = null, bool audio = false);
    }

    public static class SnesEngine
    {
        public static readonly string[] Spaces = { "vram", "cgram", "oam", "wram", "apuram" };

        // The probe's button names, in its --press spelling.
        public static string ProbeName(PadButton b) => b.ToString();

        internal static bool Held(IReadOnlyList<SnesPress>? presses, PadButton b, int frame) =>
            presses?.Any(p => p.Button == b && frame >= p.Frame && frame < p.Frame + p.Frames) == true;

        internal static readonly PadButton[] Buttons = Enumerable.Range(0, 12).Select(i => (PadButton)i).ToArray();
    }

    // An engine behind ICore and its debug target, as a black box - C# Venus for the baseline.
    public sealed class ICoreSnesEngine : ISnesEngine
    {
        private readonly Func<string, CoreBundle> _load;

        public ICoreSnesEngine(string name, Func<string, CoreBundle> load)
        {
            Name = name;
            _load = load;
        }

        public string Name { get; }

        public static ICoreSnesEngine Venus() => new("Venus (C#)", rom => CoreFactory.Load(rom, headless: true));

        public SnesRun Run(string rom, IReadOnlyList<int> frames, IReadOnlyList<SnesPress>? presses = null, bool audio = false)
        {
            var bundle = _load(rom);
            var core = bundle.Core;
            var spaces = bundle.DebugTarget.GetMemorySpaces().ToDictionary(s => s.Name.ToLowerInvariant());
            var shots = new List<SnesSnapshot>();
            var sound = new List<short>();
            int frame = 0;
            foreach (int target in frames.Order())
            {
                while (frame < target)
                {
                    foreach (var b in SnesEngine.Buttons) core.SetButton(0, b, SnesEngine.Held(presses, b, frame));
                    core.RunFrame();
                    short[] s = core.DequeueAudioSamples(int.MaxValue);
                    if (audio) sound.AddRange(s);
                    frame++;
                }
                var mem = new Dictionary<string, byte[]>();
                foreach (string name in SnesEngine.Spaces)
                {
                    if (!spaces.TryGetValue(name, out var space)) continue;
                    var bytes = new byte[space.Size];
                    for (int i = 0; i < bytes.Length; i++) bytes[i] = space.Read(i);
                    mem[name] = bytes;
                }
                shots.Add(new SnesSnapshot(target, mem, SnesPicture.FromRgba(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight)));
            }
            int rate = core.AudioSampleRate;
            (core as IDisposable)?.Dispose();
            return new SnesRun(Name, shots, rate, sound.ToArray());
        }
    }

    // VenusRT through the common interface, before it has an ICore shim (stage 6).
    public sealed class VenusRtSnesEngine : ISnesEngine
    {
        public string Name => "VenusRT";

        private readonly byte[]? _ipl;

        public VenusRtSnesEngine(byte[]? ipl = null) => _ipl = ipl ?? Ipl();

        // The SPC700 boot ROM from the corpus's firmware folder or the firmware library, never shipped (VenusRT_Native.md §21).
        public static byte[]? Ipl()
        {
            foreach (string? dir in new[] { SnesTestRomCorpus.Root is { } r ? Path.Combine(r, "firmware") : null, EmuSen.Common.Firmware.FirmwareLibrary.Directory })
                if (dir is not null && File.Exists(Path.Combine(dir, "spc700.rom"))) return File.ReadAllBytes(Path.Combine(dir, "spc700.rom"));
            return null;
        }

        // The NEC DSP firmware C# Venus's public FirmwareRequirements names for the ROM, whole or as a program and data pair, from the same folders.
        public static byte[]? DspFirmware(string rom)
        {
            foreach (var request in EmuSen.Cores.Nintendo.Venus.Memory.Cartridge.FirmwareRequirements(rom))
                foreach (string? dir in new[] { SnesTestRomCorpus.Root is { } r ? Path.Combine(r, "firmware") : null, EmuSen.Common.Firmware.FirmwareLibrary.Directory })
                {
                    if (dir is null) continue;
                    string whole = Path.Combine(dir, request.FileName), stem = Path.Combine(dir, Path.GetFileNameWithoutExtension(request.FileName));
                    if (File.Exists(whole) && new FileInfo(whole).Length == request.Size) return File.ReadAllBytes(whole);
                    if (File.Exists(stem + ".program.rom") && File.Exists(stem + ".data.rom")) return File.ReadAllBytes(stem + ".program.rom").Concat(File.ReadAllBytes(stem + ".data.rom")).ToArray();
                }
            return null;
        }

        public SnesRun Run(string rom, IReadOnlyList<int> frames, IReadOnlyList<SnesPress>? presses = null, bool audio = false)
        {
            using var m = new VenusMachine(File.ReadAllBytes(rom), _ipl, dspFirmware: DspFirmware(rom));
            var shots = new List<SnesSnapshot>();
            var sound = new List<short>();
            foreach (int target in frames.Order())
            {
                // The machine's own count: a DMA longer than a frame is one step, and one Advance then ends two frames on.
                while (m.TotalFrames < target)
                {
                    int frame = (int)m.TotalFrames;
                    uint mask = 0;
                    foreach (var b in SnesEngine.Buttons) if (SnesEngine.Held(presses, b, frame)) mask |= 1u << (int)b;
                    m.SetButtons(0, mask, 0xFFF);
                    m.Advance();
                    short[] s = m.DrainAudio(int.MaxValue);
                    if (audio) sound.AddRange(s);
                }
                var mem = SnesEngine.Spaces.ToDictionary(n => n, n => m.ReadSpace(n));
                var info = m.FrameInfo;
                var rgba = new byte[info.Bytes];
                m.CopyFrame(rgba);
                shots.Add(new SnesSnapshot(target, mem, SnesPicture.FromRgba(rgba, info.Width, info.Height)));
            }
            return new SnesRun(Name, shots, m.AudioSampleRate, sound.ToArray());
        }
    }

    // Mesen, run as a black box through the reference probe; each dump set is cached by ROM, frames, presses and probe build.
    public sealed class MesenProbeSnesEngine : ISnesEngine
    {
        public string Name => "Mesen";

        public static string Probe => Environment.GetEnvironmentVariable("EMUSEN_MESEN_PROBE")
            ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache/emusen/probe/mesen/probe");

        public static string Checkout => Environment.GetEnvironmentVariable("EMUSEN_MESEN_CHECKOUT")
            ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "Projects/mesen-reference");

        public static bool Available => File.Exists(Probe) && File.Exists(Path.Combine(Checkout, "bin/pgohelperlib.so"));

        private readonly string _cache;

        public MesenProbeSnesEngine(string cache) => _cache = cache;

        public SnesRun Run(string rom, IReadOnlyList<int> frames, IReadOnlyList<SnesPress>? presses = null, bool audio = false)
        {
            var sorted = frames.Order().ToArray();
            int start = sorted[0], end = sorted[^1];
            int stride = sorted.Length > 1 ? sorted[1] - sorted[0] : 1;
            for (int i = 1; i < sorted.Length; i++)
                if (sorted[i] - sorted[i - 1] != stride) throw new ArgumentException("the probe reports at a fixed stride", nameof(frames));

            var args = new List<string> { rom, "", start.ToString(), end.ToString(), stride.ToString() };
            foreach (var p in presses ?? Array.Empty<SnesPress>()) { args.Add("--press"); args.Add($"{p.Frame}:{SnesEngine.ProbeName(p.Button)}:{p.Frames}"); }
            var probe = new FileInfo(Probe);
            byte[]? firmware = VenusRtSnesEngine.DspFirmware(rom);
            string key = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(
                string.Join("\n", args.Skip(2)) + $"\n{audio}\n{probe.Length}:{probe.LastWriteTimeUtc.Ticks}\n" + Convert.ToHexString(MD5.HashData(File.ReadAllBytes(rom)))
                + (firmware is null ? "" : "\nfirmware " + Convert.ToHexString(MD5.HashData(firmware))))))[..20];
            string dir = Path.Combine(_cache, key);
            string log = Path.Combine(dir, "probe.log");
            if (!File.Exists(Path.Combine(dir, "done")))
            {
                Directory.CreateDirectory(dir);
                args[1] = dir;
                // Mesen reads a DSP's firmware as home/Firmware/<name>.rom, program and data in one file (measured 2026-10-02).
                if (firmware is not null)
                    foreach (var request in EmuSen.Cores.Nintendo.Venus.Memory.Cartridge.FirmwareRequirements(rom))
                    {
                        Directory.CreateDirectory(Path.Combine(dir, "mesenhome", "Firmware"));
                        File.WriteAllBytes(Path.Combine(dir, "mesenhome", "Firmware", request.FileName), firmware);
                    }
                if (audio) { args.Add("--wav"); args.Add(Path.Combine(dir, "audio.wav")); }
                var psi = new ProcessStartInfo(Probe) { WorkingDirectory = Checkout, RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (string a in args) psi.ArgumentList.Add(a);
                using var p = Process.Start(psi)!;
                var err = p.StandardError.ReadToEndAsync();
                string output = p.StandardOutput.ReadToEnd();
                p.WaitForExit();
                File.WriteAllText(log, output + err.Result + $"\nexit={p.ExitCode}\n");
                string home = Path.Combine(dir, "mesenhome");
                if (Directory.Exists(home)) Directory.Delete(home, true);
                if (p.ExitCode != 0) throw new InvalidOperationException($"the probe exited {p.ExitCode} on {rom}: {err.Result}");
                File.WriteAllText(Path.Combine(dir, "done"), "");
            }

            // The probe prints the PPU's output size once per report, in report order.
            var sizes = Regex.Matches(File.ReadAllText(log), @"PPU (\d+)x(\d+):").Select(m => (int.Parse(m.Groups[1].Value), int.Parse(m.Groups[2].Value))).ToList();
            var shots = new List<SnesSnapshot>();
            for (int i = 0; i < sorted.Length; i++)
            {
                int f = sorted[i];
                var mem = new Dictionary<string, byte[]>();
                foreach (string name in SnesEngine.Spaces)
                {
                    string file = Path.Combine(dir, $"mesen_{name}_f{f:D5}.bin");
                    if (File.Exists(file)) mem[name] = File.ReadAllBytes(file);
                }
                SnesPicture? pic = null;
                string screen = Path.Combine(dir, $"mesen_screen_f{f:D5}.bin");
                if (File.Exists(screen) && i < sizes.Count)
                {
                    var (w, h) = sizes[i];
                    byte[] raw = File.ReadAllBytes(screen);
                    // A hi-res or interlaced frame fills the probe's whole 512x478 buffer, every line twice when not interlaced; its log still says 256x239.
                    bool wide = false;
                    for (int j = w * h * 2; j + 1 < raw.Length && !wide; j += 2) wide = ((raw[j] | raw[j + 1] << 8) & 0x7FFF) != 0;
                    if (wide) (w, h) = (512, 478);
                    var px = new ushort[w * h];
                    for (int j = 0; j < px.Length && j * 2 + 1 < raw.Length; j++) px[j] = (ushort)(raw[j * 2] | raw[j * 2 + 1] << 8);
                    pic = new SnesPicture(w, h, px);
                }
                shots.Add(new SnesSnapshot(f, mem, pic));
            }
            var (rate, samples) = audio ? ReadWav(Path.Combine(dir, "audio.wav")) : (0, Array.Empty<short>());
            return new SnesRun(Name, shots, rate, samples);
        }

        // A 16-bit PCM WAV's rate and samples; a mono file is widened to stereo.
        public static (int Rate, short[] Samples) ReadWav(string path)
        {
            if (!File.Exists(path)) return (0, Array.Empty<short>());
            byte[] b = File.ReadAllBytes(path);
            int rate = 0, channels = 2, at = 12;
            while (at + 8 <= b.Length)
            {
                string id = Encoding.ASCII.GetString(b, at, 4);
                int size = BitConverter.ToInt32(b, at + 4);
                if (id == "fmt ") { channels = BitConverter.ToInt16(b, at + 10); rate = BitConverter.ToInt32(b, at + 12); }
                if (id == "data")
                {
                    int n = Math.Min(size, b.Length - at - 8) / 2;
                    var s = new short[n];
                    Buffer.BlockCopy(b, at + 8, s, 0, n * 2);
                    return (rate, channels == 1 ? s.SelectMany(x => new[] { x, x }).ToArray() : s);
                }
                at += 8 + size + (size & 1);
            }
            return (rate, Array.Empty<short>());
        }
    }
}
