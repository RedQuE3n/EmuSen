using System.Diagnostics;
using System.Security.Cryptography;
using System.Text.Json.Nodes;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Fixtures.RomRunner
{
    // A picture as 8-bit RGB, row by row, whatever the engine's own format - see Nephrite_Native.md §3.1.
    public sealed record RomPicture(int Width, int Height, byte[] Rgb)
    {
        public (byte R, byte G, byte B) At(int x, int y) => (Rgb[(y * Width + x) * 3], Rgb[(y * Width + x) * 3 + 1], Rgb[(y * Width + x) * 3 + 2]);

        public static RomPicture FromRgba(byte[] rgba, int width, int height)
        {
            var rgb = new byte[width * height * 3];
            for (int i = 0; i < width * height && i * 4 + 2 < rgba.Length; i++) (rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]) = (rgba[i * 4], rgba[i * 4 + 1], rgba[i * 4 + 2]);
            return new RomPicture(width, height, rgb);
        }

        // The probe's screen formats, as its manifest names them.
        public static RomPicture? FromProbe(byte[] data, int width, int height, string format)
        {
            var rgb = new byte[width * height * 3];
            for (int i = 0; i < width * height; i++)
            {
                switch (format)
                {
                    case "Xrgb8888" when i * 4 + 3 < data.Length:
                        (rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]) = (data[i * 4 + 2], data[i * 4 + 1], data[i * 4]);
                        break;
                    case "Rgb565" when i * 2 + 1 < data.Length:
                        int p = data[i * 2] | data[i * 2 + 1] << 8;
                        (rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]) = ((byte)((p >> 11) << 3), (byte)(((p >> 5) & 63) << 2), (byte)((p & 31) << 3));
                        break;
                    case "Bgr555" when i * 2 + 1 < data.Length:
                        int q = data[i * 2] | data[i * 2 + 1] << 8;
                        (rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]) = ((byte)((q & 31) << 3), (byte)(((q >> 5) & 31) << 3), (byte)(((q >> 10) & 31) << 3));
                        break;
                    default:
                        return null;
                }
            }
            return new RomPicture(width, height, rgb);
        }
    }

    // One engine's memories, by lower-case space name, and its picture after a frame.
    public sealed record RomSnapshot(int Frame, IReadOnlyDictionary<string, byte[]> Spaces, RomPicture? Picture);

    // A run's snapshots in frame order, and its sound from power-on as interleaved stereo.
    public sealed record RomRun(string Engine, IReadOnlyList<RomSnapshot> Snapshots, int AudioRate, short[] Audio)
    {
        public RomSnapshot? At(int frame) => Snapshots.FirstOrDefault(s => s.Frame == frame);
    }

    // Button held on port 0 from Frame for Frames frames, as the probe's --press counts them.
    public sealed record RomPress(int Frame, PadButton Button, int Frames = 4)
    {
        public static bool Held(IReadOnlyList<RomPress>? presses, PadButton b, int frame) =>
            presses?.Any(p => p.Button == b && frame >= p.Frame && frame < p.Frame + p.Frames) == true;
    }

    // Any engine, run from power-on to each of the frames asked for - see Nephrite_Native.md §3.
    public interface ITestRomEngine
    {
        string Name { get; }
        RomRun Run(string rom, IReadOnlyList<int> frames, IReadOnlyList<RomPress>? presses = null, bool audio = false);
    }

    // Any library on the core ABI v1, driven through its own descriptors: spaces by machine info's names, presses by the controller's canonical controls.
    public sealed class CoreAbiTestRomEngine : ITestRomEngine
    {
        private readonly CoreLibrary _library;
        private readonly Func<string, IReadOnlyList<(uint Which, byte[] Data)>> _files;

        public CoreAbiTestRomEngine(string libraryPath, Func<string, IReadOnlyList<(uint Which, byte[] Data)>>? files = null)
        {
            _library = CoreLibrary.Open(libraryPath);
            _files = files ?? (_ => Array.Empty<(uint, byte[])>());
        }

        public bool Available => _library.Available;
        public string Report => _library.Report;
        public string Name => _library.Available ? _library.Info.Name : Path.GetFileName(_library.Path);

        public RomRun Run(string rom, IReadOnlyList<int> frames, IReadOnlyList<RomPress>? presses = null, bool audio = false)
        {
            using var m = new CoreMachine(_library, File.ReadAllBytes(rom), "", _files(rom));
            var port0 = m.Info.Ports.FirstOrDefault(p => p.Port == 0)?.Controller;
            var buttons = _library.Info.Systems.SelectMany(s => s.Controllers).FirstOrDefault(c => c.Id == port0)?.Buttons ?? Array.Empty<CoreButton>();
            var shots = new List<RomSnapshot>();
            var sound = new List<short>();
            uint all = buttons.Aggregate(0u, (a, b) => a | 1u << (int)b.Bit);
            foreach (int target in frames.Order())
            {
                while (m.TotalFrames < target)
                {
                    int frame = (int)m.TotalFrames;
                    uint mask = buttons.Where(b => b.Control is { } c && RomPress.Held(presses, c, frame)).Aggregate(0u, (a, b) => a | 1u << (int)b.Bit);
                    m.SetButtons(0, mask, all);
                    m.Advance();
                    var (s, _) = m.DrainAudio(int.MaxValue);
                    if (audio) sound.AddRange(s);
                }
                var mem = new Dictionary<string, byte[]>();
                // A view holds nothing the other spaces do not, so a snapshot leaves it out - see EmuSen_CoreAPI.md §29.
                foreach (var space in m.Info.Spaces.Where(s => !s.View))
                {
                    var bytes = new byte[space.Size];
                    m.ReadSpace(space.Id, 0, bytes);
                    mem[space.Name.ToLowerInvariant()] = bytes;
                }
                var info = m.FrameInfo;
                var rgba = new byte[info.Bytes];
                m.CopyFrame(rgba);
                shots.Add(new RomSnapshot(target, mem, RomPicture.FromRgba(rgba, info.Width, info.Height)));
            }
            return new RomRun(Name, shots, m.AudioSampleRate, sound.ToArray());
        }
    }

    // A run the probe skipped because it needs firmware nobody supplied (its exit 4) - see Nephrite_Native.md §8.
    public sealed class FirmwareSkippedException(string message) : InvalidOperationException(message);

    // A libretro core run as a black box through the reference probe; each dump set is cached by ROM, frames, presses, options, core and probe build - see Nephrite_Plan.md §3.4.
    public sealed class LibretroProbeEngine : ITestRomEngine
    {
        public const string ProbeVariable = "EMUSEN_LIBRETRO_PROBE";

        private readonly string _core;
        private readonly string _cache;
        private readonly IReadOnlyList<KeyValuePair<string, string>> _options;
        private readonly string? _system;
        private readonly string? _systemDir;
        private readonly IReadOnlyCollection<string> _wordSwapped;

        public LibretroProbeEngine(string name, string corePath, string cacheRoot, IReadOnlyDictionary<string, string>? options = null, string? system = null, string? systemDir = null, IReadOnlyCollection<string>? wordSwapped = null)
        {
            _wordSwapped = wordSwapped ?? Array.Empty<string>();
            Name = name;
            _core = corePath;
            _cache = cacheRoot;
            _options = options?.OrderBy(o => o.Key, StringComparer.Ordinal).ToArray() ?? Array.Empty<KeyValuePair<string, string>>();
            _system = system;
            _systemDir = systemDir;
        }

        public string Name { get; }

        public static string ProbePath => Environment.GetEnvironmentVariable(ProbeVariable) is { Length: > 0 } p ? p
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "probe", "libretro", "probe");

        public bool Available => File.Exists(ProbePath) && File.Exists(_core);

        private static int Gcd(int a, int b) => b == 0 ? a : Gcd(b, a % b);

        // A space a core holds as little-endian words, put back in the 68000's byte order.
        public static byte[] SwapWords(byte[] b)
        {
            var o = (byte[])b.Clone();
            for (int i = 0; i + 1 < o.Length; i += 2) (o[i], o[i + 1]) = (b[i + 1], b[i]);
            return o;
        }

        public RomRun Run(string rom, IReadOnlyList<int> frames, IReadOnlyList<RomPress>? presses = null, bool audio = false)
        {
            var probe = new FileInfo(ProbePath);
            var core = new FileInfo(_core);
            string key = Convert.ToHexStringLower(MD5.HashData(System.Text.Encoding.UTF8.GetBytes(string.Join("|",
                Convert.ToHexStringLower(MD5.HashData(File.ReadAllBytes(rom))), string.Join(",", frames.Order()),
                string.Join(",", presses?.Select(p => $"{p.Frame}:{p.Button}:{p.Frames}") ?? Array.Empty<string>()), audio,
                string.Join(",", _options.Select(o => $"{o.Key}={o.Value}")), _system ?? "", _systemDir ?? "",
                core.Length, core.LastWriteTimeUtc.Ticks, probe.Length, probe.LastWriteTimeUtc.Ticks))));
            string dir = Path.Combine(_cache, Name, key);
            int first = frames.Min(), last = frames.Max();
            int stride = frames.Select(f => f - first).Aggregate(0, Gcd);
            if (!File.Exists(Path.Combine(dir, "done")))
            {
                Directory.CreateDirectory(dir);
                var psi = new ProcessStartInfo(ProbePath) { RedirectStandardOutput = true, RedirectStandardError = true, WorkingDirectory = dir };
                foreach (string a in new[] { rom, dir, first.ToString(), last.ToString(), Math.Max(1, stride).ToString() }) psi.ArgumentList.Add(a);
                psi.ArgumentList.Add("--core");
                psi.ArgumentList.Add(_core);
                foreach (var p in presses ?? Array.Empty<RomPress>())
                {
                    psi.ArgumentList.Add("--press");
                    psi.ArgumentList.Add($"{p.Frame}:{p.Button}:{p.Frames}");
                }
                if (audio)
                {
                    psi.ArgumentList.Add("--wav");
                    psi.ArgumentList.Add(Path.Combine(dir, "audio.wav"));
                }
                foreach (var o in _options)
                {
                    psi.ArgumentList.Add("--option");
                    psi.ArgumentList.Add($"{o.Key}={o.Value}");
                }
                if (_system is { Length: > 0 })
                {
                    psi.ArgumentList.Add("--system");
                    psi.ArgumentList.Add(_system);
                }
                if (_systemDir is { Length: > 0 })
                {
                    psi.ArgumentList.Add("--sysdir");
                    psi.ArgumentList.Add(_systemDir);
                }
                using var process = Process.Start(psi)!;
                string output = process.StandardOutput.ReadToEnd() + process.StandardError.ReadToEnd();
                process.WaitForExit();
                File.WriteAllText(Path.Combine(dir, "probe.log"), output);
                if (process.ExitCode == 4) throw new FirmwareSkippedException(output.Split('\n').FirstOrDefault(l => l.StartsWith("[SKIP]")) ?? output);
                if (process.ExitCode != 0) throw new InvalidOperationException($"the probe exited {process.ExitCode} on {rom}: {output}");
                File.WriteAllText(Path.Combine(dir, "done"), "");
            }
            var shots = new List<RomSnapshot>();
            foreach (int frame in frames.Order())
            {
                // The probe names its dumps by the core's own name, so the manifest is found by its frame.
                string? manifest = Directory.EnumerateFiles(dir, $"*_manifest_f{frame:D5}.json").FirstOrDefault();
                if (manifest is null) continue;
                var doc = JsonNode.Parse(File.ReadAllText(manifest))!;
                var mem = new Dictionary<string, byte[]>();
                foreach (var s in doc["spaces"]!.AsArray()) mem[s!["name"]!.GetValue<string>()] = File.ReadAllBytes(Path.Combine(dir, s["file"]!.GetValue<string>()));
                foreach (string name in _wordSwapped.Where(mem.ContainsKey)) mem[name] = SwapWords(mem[name]);
                RomPicture? picture = null;
                if (doc["screen"] is JsonObject screen)
                    picture = RomPicture.FromProbe(File.ReadAllBytes(Path.Combine(dir, screen["file"]!.GetValue<string>())), screen["width"]!.GetValue<int>(), screen["height"]!.GetValue<int>(), screen["format"]!.GetValue<string>());
                shots.Add(new RomSnapshot(frame, mem, picture));
            }
            return new RomRun(Name, shots, 0, Array.Empty<short>());
        }
    }

    // A fetched corpus outside the repository: its root from a variable, and its manifest, one ROM per MD5 - see Nephrite_Plan.md §3.
    public sealed record TestRomCorpus(string Variable)
    {
        public string? Root => Environment.GetEnvironmentVariable(Variable) is { } r && Directory.Exists(r) ? r : null;

        // unique-roms.txt: "md5 size [kind] path", the kind a word without a slash, the path relative to the root or absolute and free to hold spaces.
        public IReadOnlyList<(string Md5, string Path)> Unique() =>
            Root is not { } root || !File.Exists(System.IO.Path.Combine(root, "unique-roms.txt")) ? Array.Empty<(string, string)>()
            : File.ReadLines(System.IO.Path.Combine(root, "unique-roms.txt"))
                .Where(l => l.Length > 0 && !l.StartsWith('#'))
                .Select(l => System.Text.RegularExpressions.Regex.Match(l, @"^([0-9a-fA-F]{32})\s+\d+\s+(?:[^\s/]+\s+)?(.+)$"))
                .Where(m => m.Success)
                .Select(m => (m.Groups[1].Value, System.IO.Path.IsPathRooted(m.Groups[2].Value) ? m.Groups[2].Value : System.IO.Path.Combine(root, m.Groups[2].Value)))
                .ToList();
    }
}
