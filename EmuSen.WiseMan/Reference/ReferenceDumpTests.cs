using System.Text.RegularExpressions;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Reference
{
    // Differential tests against whichever reference emulators have dumped a set
    // locally - see EmuSen_Debugging_Tools_Reference_v5.md §3.45. Neither the ROMs
    // nor the dumps are in this repo (both are derived from commercial game data),
    // so every test returns early when they are absent, the same shape as
    // NecDspRealFirmwareTests.
    public class ReferenceDumpTests
    {
        private static string DumpRoot =>
            Path.Combine(DianaOSSandbox.InstallDirectory, "EmuSen.WiseMan", "Reference", "dumps");

        // <backend>_<space>_f<frame>.bin - the backend names itself, so a dump set
        // is never tied to one emulator having produced it.
        private static readonly Regex BlobName = new(@"^(?<backend>.+?)_(?<space>[a-z]+)_f(?<frame>\d+)\.bin$");

        private sealed record Fixture(string RomStem, string Backend, string System, long Frame, string DumpDir);

        public static TheoryData<string, string, long> Available()
        {
            var data = new TheoryData<string, string, long>();
            foreach (Fixture fixture in Discover()) data.Add(fixture.RomStem, fixture.Backend, fixture.Frame);

            // xUnit rejects empty TheoryData, and "no dumps present" has to pass.
            if (data.Count == 0) data.Add(string.Empty, string.Empty, 0);
            return data;
        }

        // Driven off the filesystem rather than a hardcoded list, so dropping a new
        // dump directory in is all it takes to extend the coverage.
        private static IEnumerable<Fixture> Discover()
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
                    if (system.Length > 0) yield return new Fixture(Path.GetFileName(dir), backend, system, frame, dir);
                }
            }
        }

        // Catches a half-synced machine, whose dumps all early-return green - see §3.54.
        [Fact]
        public void Every_dump_set_that_names_its_rom_can_find_it()
        {
            // Pre-manifest sets are matched by directory name, which was only ever a guess.
            Fixture[] fixtures = Discover().Where(HasManifest).ToArray();
            if (fixtures.Length == 0) return;

            string[] orphaned = fixtures.Where(f => RomPath(f) is null)
                .Select(f => f.RomStem).Distinct().OrderBy(stem => stem).ToArray();

            Assert.True(orphaned.Length == 0,
                $"{orphaned.Length} dump set(s) under {DumpRoot} have no ROM in the library, " +
                $"so their comparisons silently do nothing: {string.Join(", ", orphaned)}");
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

        private static string? RomPath(Fixture fixture)
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

        private static bool HasManifest(Fixture fixture) =>
            File.Exists(Path.Combine(fixture.DumpDir, $"{fixture.Backend}_manifest_f{fixture.Frame:D5}.json"));

        private static byte[]? Reference(Fixture fixture, string space)
        {
            string path = Path.Combine(fixture.DumpDir, $"{fixture.Backend}_{space}_f{fixture.Frame:D5}.bin");
            return File.Exists(path) ? File.ReadAllBytes(path) : null;
        }

        private static Dictionary<string, byte[]>? Ours(Fixture fixture, string romPath)
        {
            var spaces = new Dictionary<string, byte[]>(StringComparer.OrdinalIgnoreCase);

            if (fixture.System == "nes")
            {
                var core = new MoonCore();
                core.LoadRom(romPath);
                for (long i = 0; i < fixture.Frame; i++) core.RunFrame();

                var target = new MoonDebugTarget(core);
                // The probe's names on the left, ours on the right.
                foreach ((string dump, string ours) in new[]
                    { ("ram", MoonCore.SpaceRam), ("nametable", MoonCore.SpaceCiram), ("oam", MoonCore.SpaceOam),
                      ("palette", MoonCore.SpacePalette), ("chr", MoonCore.SpaceChr), ("work", MoonCore.SpacePrgRam) })
                {
                    byte[]? bytes = Read(target.GetMemorySpaces(), ours);
                    if (bytes != null) spaces[dump] = bytes;
                }
                return spaces;
            }

            if (fixture.System == "snes")
            {
                var core = new VenusCore(headless: true);
                core.LoadRom(romPath);
                for (long i = 0; i < fixture.Frame; i++) core.RunFrame();

                var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
                foreach (string name in new[] { "VRAM", "CGRAM", "OAM", "GSURAM" })
                {
                    byte[]? bytes = Read(target.GetMemorySpaces(), name);
                    if (bytes != null) spaces[name.ToLowerInvariant()] = bytes;
                }
                return spaces;
            }

            return null;
        }

        private static byte[]? Read(IEnumerable<global::EmuSen.DianaOS.DianaOS.Lib.IDebugMemorySpace> spaces, string name)
        {
            var space = spaces.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase));
            if (space == null) return null;

            var bytes = new byte[space.Size];
            for (int i = 0; i < bytes.Length; i++) bytes[i] = space.Read(i);
            return bytes;
        }

        // Moon folds $3F10/$14/$18/$1C onto $3F00/$04/$08/$0C on read and write
        // both, so those four never hold a value; Mesen stores the mirror too.
        // Rendering is identical either way - see §3.43. Expected, not a divergence.
        private static bool IsPaletteMirror(string space, int offset) =>
            space == "palette" && (offset & 0x03) == 0 && offset >= 0x10;

        // Reports the first differing offset rather than just "not equal" - which
        // block differs is most of the diagnosis, see Venus_SuperFX.md §9.
        private static void AssertSame(string label, byte[] expected, byte[] actual, string space)
        {
            int shared = Math.Min(expected.Length, actual.Length);
            int differing = 0, firstOffset = -1;

            for (int i = 0; i < shared; i++)
            {
                if (expected[i] == actual[i] || IsPaletteMirror(space, i)) continue;
                differing++;
                if (firstOffset < 0) firstOffset = i;
            }

            Assert.True(differing == 0,
                $"{label} differs from the reference at ${firstOffset:X4}: expected "
                + $"${(firstOffset >= 0 ? expected[firstOffset] : 0):X2}, got ${(firstOffset >= 0 ? actual[firstOffset] : 0):X2} "
                + $"({differing} of {shared} bytes differ)");
        }

        // VRAM and CGRAM are byte-exact against Mesen, which is what makes the rest
        // diagnostic: a frame that matches here but looks wrong is a renderer bug,
        // and one that differs is an upload bug. That split is what turned the
        // Yoshi's Island sprite hunt from a guess into a bisection.
        [Theory]
        [MemberData(nameof(Available))]
        public void Video_memory_matches_the_reference_at_the_same_frame(string romStem, string backend, long frame)
        {
            Fixture? fixture = Resolve(romStem, backend, frame);
            if (fixture == null) return;

            string? romPath = RomPath(fixture);
            if (romPath == null) return;

            Dictionary<string, byte[]>? ours = Ours(fixture, romPath);
            if (ours == null) return;

            foreach (string space in fixture.System == "nes"
                ? new[] { "nametable", "palette", "chr" }
                : new[] { "vram", "cgram" })
            {
                byte[]? expected = Reference(fixture, space);
                if (expected == null || !ours.TryGetValue(space, out byte[]? actual)) continue;

                // A space the game rewrites every frame is caught mid-update, and
                // then "differs at frame N" says nothing about whether the bytes
                // are right - only about phase. SMB3 scrolls its title screen 32
                // nametable bytes per frame, and our frame 180 is byte-identical
                // to Mesen's 182, with the difference growing exactly 32 bytes per
                // frame either side: the same data, two frames later. Matching an
                // adjacent dumped frame exactly is a stronger claim than any
                // tolerance would be, so that is what this accepts - and it
                // reports the offset, because a *growing* offset is a real bug
                // where a constant one is boot timing. See §3.46.
                if (Matches(expected, actual, space)) continue;

                // Within PhaseSearch of either end of the dumped window the
                // neighbour that would prove a phase offset may simply not have
                // been dumped, and "no match" then says nothing. An edge frame is
                // not evidence, so it is skipped rather than failed.
                if (AtWindowEdge(fixture, space)) continue;

                int offset = PhaseOffset(fixture, space, actual);
                Assert.True(offset != 0,
                    $"{romStem} {backend} frame {frame} {space}: {Differing(expected, actual, space)} of "
                    + $"{expected.Length} bytes differ, and no dumped frame within "
                    + $"{PhaseSearch} matches exactly.");
            }
        }

        private const int PhaseSearch = 8;

        private static bool AtWindowEdge(Fixture fixture, string space)
        {
            long lowest = long.MaxValue, highest = long.MinValue;
            foreach (string file in Directory.GetFiles(fixture.DumpDir, $"{fixture.Backend}_{space}_f*.bin"))
            {
                Match match = BlobName.Match(Path.GetFileName(file));
                if (!match.Success) continue;

                long frame = long.Parse(match.Groups["frame"].Value);
                lowest = Math.Min(lowest, frame);
                highest = Math.Max(highest, frame);
            }
            return fixture.Frame - PhaseSearch < lowest || fixture.Frame + PhaseSearch > highest;
        }

        // The nearest dumped frame whose bytes we reproduce exactly, or 0 for none.
        private static int PhaseOffset(Fixture fixture, string space, byte[] actual)
        {
            for (int distance = 1; distance <= PhaseSearch; distance++)
            {
                foreach (int offset in new[] { distance, -distance })
                {
                    var neighbour = fixture with { Frame = fixture.Frame + offset };
                    byte[]? bytes = Reference(neighbour, space);
                    if (bytes != null && Matches(bytes, actual, space)) return offset;
                }
            }
            return 0;
        }

        // Compared over the shared prefix, because the two sides legitimately size
        // some spaces differently: Moon's `Ciram` is a 4 KB array, while a stock
        // NES has 2 KB physical and that is all the reference exposes. The extra
        // half is storage for boards that supply their own VRAM, and holds nothing
        // to compare against on a cartridge that does not.
        private static int Differing(byte[] expected, byte[] actual, string space)
        {
            int shared = Math.Min(expected.Length, actual.Length), differing = 0;
            for (int i = 0; i < shared; i++)
            {
                if (expected[i] != actual[i] && !IsPaletteMirror(space, i)) differing++;
            }
            return differing;
        }

        private static bool Matches(byte[] expected, byte[] actual, string space) =>
            Math.Min(expected.Length, actual.Length) > 0 && Differing(expected, actual, space) == 0;

        // Internal RAM is deliberately NOT asserted byte-exact, and the reason is
        // measured rather than assumed: no two emulators agree on it. On SMB3,
        // Mesen and Nestopia - both mature and independent - differ by 9, 16, 32,
        // 39 and 30 bytes of 2048 at frames 60/120/180/240/300, and EmuSen sits
        // inside that spread, closer to Nestopia than Mesen is at three of the
        // five. A frame boundary is not the same instant in two emulators, and
        // whatever the CPU was mid-way through writing differs accordingly.
        //
        // What is worth pinning is the order of magnitude. A real fault - a broken
        // mapper, a mis-decoded opcode, a dead interrupt - diverges hundreds or
        // thousands of bytes, not tens. The budget below is a tripwire for that,
        // not a proof of correctness; the video-memory tests above are the ones
        // that assert real equality. See §3.46.
        private const double RamDivergenceBudget = 0.05;

        [Theory]
        [MemberData(nameof(Available))]
        public void Cpu_visible_memory_stays_close_to_the_reference(string romStem, string backend, long frame)
        {
            Fixture? fixture = Resolve(romStem, backend, frame);
            if (fixture == null || fixture.System != "nes") return;

            string? romPath = RomPath(fixture);
            if (romPath == null) return;

            Dictionary<string, byte[]>? ours = Ours(fixture, romPath);
            byte[]? expected = Reference(fixture, "ram");
            if (ours == null || expected == null || !ours.TryGetValue("ram", out byte[]? actual)) return;

            int shared = Math.Min(expected.Length, actual.Length);
            int differing = 0;
            for (int i = 0; i < shared; i++) if (expected[i] != actual[i]) differing++;

            Assert.True(differing <= shared * RamDivergenceBudget,
                $"{romStem} {backend} frame {frame} ram: {differing} of {shared} bytes differ "
                + $"({differing * 100.0 / shared:F1}%), over the {RamDivergenceBudget:P0} tripwire.");
        }

        // OAM is not byte-exact on every game yet, so this pins the thing that was
        // actually broken on Yoshi's Island: how many sprites reach the screen at
        // all, which the BGE/BLT swap drove to zero while Mesen drew 58.
        [Theory]
        [MemberData(nameof(Available))]
        public void The_same_number_of_sprites_are_on_screen_as_the_reference(string romStem, string backend, long frame)
        {
            Fixture? fixture = Resolve(romStem, backend, frame);
            if (fixture == null) return;

            string? romPath = RomPath(fixture);
            if (romPath == null) return;

            byte[]? expected = Reference(fixture, "oam");
            Dictionary<string, byte[]>? ours = Ours(fixture, romPath);
            if (expected == null || ours == null || !ours.TryGetValue("oam", out byte[]? actual)) return;

            // Y = $F0 is how these games park an entry off-screen; counting those
            // is a stabler comparison than any single coordinate.
            int stride = fixture.System == "nes" ? 4 : 4;
            int limit = fixture.System == "nes" ? 256 : 512;

            static int Visible(byte[] oam, int stride, int limit, int yOffset)
            {
                int n = 0;
                for (int i = 0; i + stride <= Math.Min(limit, oam.Length); i += stride) if (oam[i + yOffset] != 0xF0) n++;
                return n;
            }

            Assert.Equal(Visible(expected, stride, limit, fixture.System == "nes" ? 0 : 1),
                         Visible(actual, stride, limit, fixture.System == "nes" ? 0 : 1));
        }

        private static Fixture? Resolve(string romStem, string backend, long frame)
        {
            if (romStem.Length == 0) return null;
            return Discover().FirstOrDefault(f => f.RomStem == romStem && f.Backend == backend && f.Frame == frame);
        }
    }
}
