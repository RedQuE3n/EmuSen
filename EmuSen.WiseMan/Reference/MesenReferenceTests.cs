using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Etc;

namespace EmuSen.WiseMan.Reference
{
    // Differential tests against Mesen, run off dumps MesenProbe wrote at a
    // known frame. Neither the ROMs nor the dumps are in this repo (both are
    // derived from commercial game data), so every test here returns early
    // when they are absent - same shape as NecDspRealFirmwareTests. Producing
    // a dump set: EmuSen_Debugging_Tools_Reference_v5.md §3.39.
    public class MesenReferenceTests
    {
        // Reference/dumps/<romStem>/mesen_<space>_f<frame>.bin, beside this source.
        private static string DumpRoot =>
            Path.Combine(DianaOSSandbox.RootDirectory, "EmuSen.WiseMan", "Reference", "dumps");

        private static string RomPath(string stem) =>
            Path.Combine(DianaOSSandbox.UsrHomeDirectory, "Games", "SNES", stem + ".smc");

        private sealed record Fixture(string RomStem, long Frame, string DumpDir);

        // Every romStem/frame pair that has a complete dump set locally. Driven
        // off the filesystem rather than a hardcoded list so dropping a new
        // dump directory in is all it takes to extend the coverage.
        public static TheoryData<string, long> Available()
        {
            var data = new TheoryData<string, long>();
            if (!Directory.Exists(DumpRoot)) { data.Add(string.Empty, 0); return data; }

            foreach (string dir in Directory.GetDirectories(DumpRoot))
            {
                string stem = Path.GetFileName(dir);
                if (!File.Exists(RomPath(stem))) continue;

                foreach (string vram in Directory.GetFiles(dir, "mesen_vram_f*.bin"))
                {
                    string tag = Path.GetFileNameWithoutExtension(vram).Split("_f")[^1];
                    if (long.TryParse(tag, out long frame)) data.Add(stem, frame);
                }
            }

            // xUnit rejects an empty TheoryData, and "no dumps present" has to
            // be a pass rather than a failure on a clean checkout.
            if (data.Count == 0) data.Add(string.Empty, 0);
            return data;
        }

        private static Fixture? Resolve(string romStem, long frame)
        {
            if (romStem.Length == 0) return null;
            string dir = Path.Combine(DumpRoot, romStem);
            return Directory.Exists(dir) && File.Exists(RomPath(romStem))
                ? new Fixture(romStem, frame, dir)
                : null;
        }

        private static byte[]? Reference(Fixture fixture, string space) =>
            File.Exists(Path.Combine(fixture.DumpDir, $"mesen_{space}_f{fixture.Frame:D5}.bin"))
                ? File.ReadAllBytes(Path.Combine(fixture.DumpDir, $"mesen_{space}_f{fixture.Frame:D5}.bin"))
                : null;

        private static SnesDebugTarget RunTo(Fixture fixture)
        {
            var core = new VenusCore(headless: true);
            core.LoadRom(RomPath(fixture.RomStem));
            for (long i = 0; i < fixture.Frame; i++) core.RunFrame();
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        private static byte[]? Ours(SnesDebugTarget target, string spaceName, int length)
        {
            var space = target.GetMemorySpaces()
                .FirstOrDefault(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
            if (space == null) return null;

            var bytes = new byte[Math.Min(length, space.Size)];
            for (int i = 0; i < bytes.Length; i++) bytes[i] = space.Read(i);
            return bytes;
        }

        // Reports the first differing offset rather than just "not equal" -
        // which block differs is most of the diagnosis, see Venus_SuperFX.md §9.
        private static void AssertSame(string label, byte[] expected, byte[] actual)
        {
            int shared = Math.Min(expected.Length, actual.Length);
            for (int i = 0; i < shared; i++)
            {
                if (expected[i] == actual[i]) continue;

                int differing = 0;
                for (int j = 0; j < shared; j++) if (expected[j] != actual[j]) differing++;
                Assert.Fail($"{label} differs from Mesen at ${i:X4}: expected ${expected[i]:X2}, got ${actual[i]:X2} "
                          + $"({differing} of {shared} bytes differ)");
            }
            Assert.Equal(expected.Length, actual.Length);
        }

        // VRAM and CGRAM are byte-exact against Mesen, which is what makes the
        // rest of this file diagnostic: a frame that matches here but looks
        // wrong is a renderer bug, and one that differs is an upload bug. That
        // split is what turned the Yoshi's Island missing-sprites hunt from a
        // guess into a bisection - see Venus_SuperFX.md §9.
        [Theory]
        [MemberData(nameof(Available))]
        public void Video_memory_matches_mesen_at_the_same_frame(string romStem, long frame)
        {
            Fixture? fixture = Resolve(romStem, frame);
            if (fixture == null) return;

            var target = RunTo(fixture);
            foreach (string space in new[] { "VRAM", "CGRAM" })
            {
                byte[]? expected = Reference(fixture, space.ToLowerInvariant());
                if (expected == null) continue;

                byte[]? actual = Ours(target, space, expected.Length);
                Assert.NotNull(actual);
                AssertSame($"{romStem} frame {frame} {space}", expected, actual!);
            }
        }

        // OAM is NOT byte-exact yet - sprite coordinates still drift a few
        // pixels from Mesen's at the same frame (Venus_SuperFX.md §9, residual
        // GSU divergence). What this pins is the thing that was actually
        // broken: how many sprites the frame puts on screen at all, which the
        // BGE/BLT swap drove to zero while Mesen drew 58.
        [Theory]
        [MemberData(nameof(Available))]
        public void The_same_number_of_sprites_are_on_screen_as_mesen(string romStem, long frame)
        {
            Fixture? fixture = Resolve(romStem, frame);
            if (fixture == null) return;

            byte[]? expected = Reference(fixture, "oam");
            if (expected == null) return;

            byte[]? actual = Ours(RunTo(fixture), "OAM", expected.Length);
            Assert.NotNull(actual);

            // Y = $F0 is how this game parks an entry off-screen; counting
            // those is a stabler comparison than any single coordinate.
            static int Visible(byte[] oam)
            {
                int n = 0;
                for (int i = 0; i + 1 < Math.Min(512, oam.Length); i += 4) if (oam[i + 1] != 0xF0) n++;
                return n;
            }

            Assert.Equal(Visible(expected), Visible(actual!));
        }

        // Game Pak RAM the GSU writes itself, which on Yoshi's Island carries
        // the OAM table the CPU then DMAs - see Venus_SuperFX.md §9. Also not
        // byte-exact yet, so this asserts the property that failed rather than
        // full equality: the table is built, not left as the "hide all" fill.
        [Theory]
        [MemberData(nameof(Available))]
        public void The_gsu_builds_its_sprite_table_in_work_ram(string romStem, long frame)
        {
            Fixture? fixture = Resolve(romStem, frame);
            if (fixture == null) return;

            byte[]? expected = Reference(fixture, "gsuram");
            if (expected == null) return;

            byte[]? actual = Ours(RunTo(fixture), "GSURAM", expected.Length);
            if (actual == null) return; // not a SuperFX cartridge

            static bool AllParked(byte[] ram, int start, int end)
            {
                for (int i = start; i + 1 < end; i += 4) if (ram[i + 1] != 0xF0) return false;
                return true;
            }

            // $0A00 is where Yoshi's Island stages the table the OAM DMA reads.
            const int TableStart = 0x0A00, TableEnd = 0x0C20;
            if (AllParked(expected, TableStart, TableEnd)) return; // Mesen has nothing on screen either
            Assert.False(AllParked(actual!, TableStart, TableEnd),
                $"{romStem} frame {frame}: every GSU sprite entry is parked off-screen, but Mesen's is not.");
        }
    }
}
