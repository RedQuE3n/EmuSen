using System.Collections.Concurrent;
using System.Text.RegularExpressions;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Reference
{
    // One dumped frame of a set: which ROM, which reference emulator, which frame.
    public sealed record ReferenceDumpFixture(string RomStem, string Backend, string System, long Frame, string DumpDir);

    // Replays each dump set once and samples it at every dumped frame - see EmuSen_Debugging_Tools_Reference_v5.md §3.55.
    public static class ReferenceDumpSnapshots
    {
        public static string DumpRoot =>
            Path.Combine(DianaOSSandbox.InstallDirectory, "EmuSen.WiseMan", "Reference", "dumps");

        // <backend>_<space>_f<frame>.bin - the backend names itself, so a dump set
        // is never tied to one emulator having produced it.
        public static readonly Regex BlobName = new(@"^(?<backend>.+?)_(?<space>[a-z]+)_f(?<frame>\d+)\.bin$");

        // The probe's space names on the left, ours on the right.
        private static readonly (string Dump, string Ours)[] NesSpaces =
        {
            ("ram", MoonCore.SpaceRam), ("nametable", MoonCore.SpaceCiram), ("oam", MoonCore.SpaceOam),
            ("palette", MoonCore.SpacePalette), ("chr", MoonCore.SpaceChr), ("work", MoonCore.SpacePrgRam),
        };

        private static readonly string[] SnesSpaces = { "VRAM", "CGRAM", "OAM", "GSURAM" };

        // Discovery walks the tree and reads every manifest, and Resolve used to do it per test.
        private static readonly Lazy<IReadOnlyList<ReferenceDumpFixture>> Fixtures =
            new(() => Scan().ToArray(), LazyThreadSafetyMode.ExecutionAndPublication);

        private static readonly ConcurrentDictionary<(string RomStem, string Backend),
            Lazy<IReadOnlyDictionary<long, IReadOnlyDictionary<string, byte[]>>>> Sets = new();

        public static IReadOnlyList<ReferenceDumpFixture> Discover() => Fixtures.Value;

        public static ReferenceDumpFixture? Resolve(string romStem, string backend, long frame)
        {
            if (romStem.Length == 0) return null;
            return Discover().FirstOrDefault(f => f.RomStem == romStem && f.Backend == backend && f.Frame == frame);
        }

        // Driven off the filesystem rather than a hardcoded list, so dropping a new
        // dump directory in is all it takes to extend the coverage.
        private static IEnumerable<ReferenceDumpFixture> Scan()
        {
            if (!Directory.Exists(DumpRoot)) yield break;

            foreach (string dir in Directory.GetDirectories(DumpRoot))
            {
                var seen = new HashSet<(string, long)>();
                foreach (string file in Directory.GetFiles(dir, "*.bin"))
                {
                    Match match = BlobName.Match(Path.GetFileName(file));
                    if (!match.Success) continue;

                    string backend = match.Groups["backend"].Value;
                    long frame = long.Parse(match.Groups["frame"].Value);
                    if (!seen.Add((backend, frame))) continue;

                    string system = SystemOf(dir, backend, frame);
                    if (system.Length > 0) yield return new ReferenceDumpFixture(Path.GetFileName(dir), backend, system, frame, dir);
                }
            }
        }

        // The manifest is authoritative; the fallback reads the console off which
        // spaces exist, which is all the pre-manifest dump sets can tell us.
        private static string SystemOf(string dir, string backend, long frame)
        {
            string manifest = Path.Combine(dir, $"{backend}_manifest_f{frame:D5}.json");
            if (File.Exists(manifest))
            {
                Match match = Regex.Match(File.ReadAllText(manifest), "\"system\"\\s*:\\s*\"(?<s>[^\"]*)\"");
                if (match.Success) return match.Groups["s"].Value;
            }

            if (File.Exists(Path.Combine(dir, $"{backend}_vram_f{frame:D5}.bin"))) return "snes";
            if (File.Exists(Path.Combine(dir, $"{backend}_nametable_f{frame:D5}.bin"))) return "nes";
            return "";
        }

        public static bool HasManifest(ReferenceDumpFixture fixture) =>
            File.Exists(Path.Combine(fixture.DumpDir, $"{fixture.Backend}_manifest_f{fixture.Frame:D5}.json"));

        public static string? RomPath(ReferenceDumpFixture fixture)
        {
            string manifest = Path.Combine(fixture.DumpDir, $"{fixture.Backend}_manifest_f{fixture.Frame:D5}.json");
            if (File.Exists(manifest))
            {
                Match match = Regex.Match(File.ReadAllText(manifest), "\"rom\"\\s*:\\s*\"(?<r>[^\"]*)\"");
                if (match.Success && File.Exists(match.Groups["r"].Value)) return match.Groups["r"].Value;
                if (match.Success)
                {
                    // The dump may have moved between machines; the library is the answer.
                    string name = Path.GetFileName(match.Groups["r"].Value);
                    return RealRom.Find(fixture.System == "nes" ? "NES" : "SNES", name);
                }
            }

            // The manifest branch above maps the console; this one used to assume SNES.
            return fixture.System == "nes"
                ? RealRom.Find("NES", fixture.RomStem + ".nes")
                : RealRom.Find("SNES", fixture.RomStem + ".smc");
        }

        // Our memory spaces at this fixture's frame, sampled from the set's single replay.
        public static IReadOnlyDictionary<string, byte[]>? Ours(ReferenceDumpFixture fixture)
        {
            string? romPath = RomPath(fixture);
            if (romPath == null) return null;

            var set = Sets.GetOrAdd((fixture.RomStem, fixture.Backend),
                _ => new Lazy<IReadOnlyDictionary<long, IReadOnlyDictionary<string, byte[]>>>(
                    () => Replay(fixture.RomStem, fixture.Backend, fixture.System, romPath),
                    LazyThreadSafetyMode.ExecutionAndPublication)).Value;

            return set.TryGetValue(fixture.Frame, out IReadOnlyDictionary<string, byte[]>? spaces) ? spaces : null;
        }

        // Boots a core of its own and runs straight to one frame - what Ours did for every test before §3.55.
        public static IReadOnlyDictionary<string, byte[]>? FreshBoot(ReferenceDumpFixture fixture)
        {
            string? romPath = RomPath(fixture);
            if (romPath == null) return null;

            return Replay(fixture.RomStem, fixture.Backend, fixture.System, romPath, onlyFrame: fixture.Frame)
                .TryGetValue(fixture.Frame, out IReadOnlyDictionary<string, byte[]>? spaces) ? spaces : null;
        }

        // One core, run forward once, sampled each time it reaches a frame the reference dumped.
        private static IReadOnlyDictionary<long, IReadOnlyDictionary<string, byte[]>> Replay(
            string romStem, string backend, string system, string romPath, long? onlyFrame = null)
        {
            long[] frames = onlyFrame is long single
                ? new[] { single }
                : Discover().Where(f => f.RomStem == romStem && f.Backend == backend)
                            .Select(f => f.Frame).Distinct().OrderBy(frame => frame).ToArray();

            var captured = new Dictionary<long, IReadOnlyDictionary<string, byte[]>>();
            if (frames.Length == 0) return captured;

            // A .srm beside an SRAM cartridge would feed a previous run's state into a reference
            // comparison, which is the failure that cost three days of measurements - see §3.55.
            CoreOptions.BatteryRamDisabled = true;

            if (system == "nes")
            {
                var core = new MoonCore();
                core.LoadRom(romPath);
                var target = new MoonDebugTarget(core);

                long ran = 0;
                foreach (long frame in frames)
                {
                    while (ran < frame) { core.RunFrame(); ran++; }
                    var spaces = new Dictionary<string, byte[]>(StringComparer.OrdinalIgnoreCase);
                    foreach ((string dump, string ours) in NesSpaces)
                    {
                        byte[]? bytes = Read(target.GetMemorySpaces(), ours);
                        if (bytes != null) spaces[dump] = bytes;
                    }
                    captured[frame] = spaces;
                }
                return captured;
            }

            if (system == "snes")
            {
                var core = new VenusCore(headless: true);
                core.LoadRom(romPath);
                var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);

                long ran = 0;
                foreach (long frame in frames)
                {
                    while (ran < frame) { core.RunFrame(); ran++; }
                    var spaces = new Dictionary<string, byte[]>(StringComparer.OrdinalIgnoreCase);
                    foreach (string name in SnesSpaces)
                    {
                        byte[]? bytes = Read(target.GetMemorySpaces(), name);
                        if (bytes != null) spaces[name.ToLowerInvariant()] = bytes;
                    }
                    captured[frame] = spaces;
                }
                return captured;
            }

            return captured;
        }

        private static byte[]? Read(IEnumerable<IDebugMemorySpace> spaces, string name)
        {
            var space = spaces.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase));
            if (space == null) return null;

            var bytes = new byte[space.Size];
            for (int i = 0; i < bytes.Length; i++) bytes[i] = space.Read(i);
            return bytes;
        }
    }
}
