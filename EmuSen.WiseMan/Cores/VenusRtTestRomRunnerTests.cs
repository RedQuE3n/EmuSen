using System.Collections.Concurrent;
using System.Runtime.InteropServices;
using System.Text;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.VenusRT;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Fixtures.Snes;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // The SNES test-ROM runner over any engine, and the baseline it records - see VenusRT_Native.md §3.
    [Collection(TestCollections.ProcessGlobals)]
    public class VenusRtTestRomRunnerTests
    {
        public const string ReportVariable = "EMUSEN_VENUSRT_REPORT";

        private readonly ITestOutputHelper _output;

        public VenusRtTestRomRunnerTests(ITestOutputHelper output) => _output = output;

        private static SnesRun RunWith(string path, byte[] vram, byte[]? cgram = null, byte[]? earlier = null)
        {
            var shots = new List<SnesSnapshot>();
            if (earlier is not null) shots.Add(new SnesSnapshot(1800, new Dictionary<string, byte[]> { ["vram"] = earlier }, null));
            shots.Add(new SnesSnapshot(3600, new Dictionary<string, byte[]> { ["vram"] = vram, ["cgram"] = cgram ?? new byte[512] }, null));
            return new SnesRun("synthetic", shots, 0, Array.Empty<short>());
        }

        private static byte[] Vram(params (int Word, string Text)[] texts)
        {
            var v = new byte[0x10000];
            foreach (var (word, text) in texts)
                for (int i = 0; i < text.Length; i++) v[(word + i) * 2] = (byte)text[i];
            return v;
        }

        [Fact]
        public void Each_suites_protocol_reads_its_pass_its_failure_and_its_silence()
        {
            string gilyon = "/corpus/roms/gilyon-snes-tests-v1.4/cputest/cputest-full.sfc";
            Assert.Equal(new SnesVerdict(SnesOutcome.Passed, "gilyon", "last=0649"), SnesTestRomGrader.Grade(gilyon, RunWith(gilyon, Vram((0x32, "Success"), (0x6E, "0649")))));
            Assert.Equal(new SnesVerdict(SnesOutcome.Failed, "gilyon", "test=0024"), SnesTestRomGrader.Grade(gilyon, RunWith(gilyon, Vram((0x32, "Failed"), (0x6E, "0024")))));
            Assert.Equal(SnesOutcome.Incomplete, SnesTestRomGrader.Grade(gilyon, RunWith(gilyon, Vram())).Outcome);

            string blargg = "/corpus/src/higan-snes-test-roms/blargg-spc-6/spc_smp.sfc";
            Assert.Equal(SnesOutcome.Passed, SnesTestRomGrader.Grade(blargg, RunWith(blargg, Vram(), new byte[] { 0x00, 0x7C })).Outcome);
            Assert.Equal(SnesOutcome.Failed, SnesTestRomGrader.Grade(blargg, RunWith(blargg, Vram(), new byte[] { 0x1F, 0x00 })).Outcome);
            Assert.Equal(SnesOutcome.Incomplete, SnesTestRomGrader.Grade(blargg, RunWith(blargg, Vram())).Outcome);

            string text = "/corpus/src/higan-snes-test-roms/jonasquinn-test-roms/test_x.sfc";
            Assert.Equal(SnesOutcome.Passed, SnesTestRomGrader.Grade(text, RunWith(text, Vram((0x40, "Timer test"), (0x60, "Passed")))).Outcome);
            Assert.Equal(SnesOutcome.Failed, SnesTestRomGrader.Grade(text, RunWith(text, Vram((0x40, "Passed 1"), (0x60, "Failed 2")))).Outcome);
            Assert.Equal(SnesOutcome.Done, SnesTestRomGrader.Grade(text, RunWith(text, Vram((0x40, "A=1234 Done")))).Outcome);
            Assert.Equal(SnesOutcome.Visual, SnesTestRomGrader.Grade(text, RunWith(text, Vram((0x40, "Press start")))).Outcome);
            Assert.Equal(SnesOutcome.NoDump, SnesTestRomGrader.Grade(text, new SnesRun("none", Array.Empty<SnesSnapshot>(), 0, Array.Empty<short>())).Outcome);
        }

        // A 512x478 reference frame holds each line twice; a 512-wide picture is compared with it column for column, a 256-wide one pixel for pair.
        [Fact]
        public void A_hi_res_reference_frame_is_compared_line_for_doubled_line()
        {
            var wide = new ushort[512 * 478];
            for (int y = 0; y < 478; y++) for (int x = 0; x < 512; x++) wide[y * 512 + x] = (ushort)((y / 2 - 7) * 600 + x & 0x7FFF);
            var ours = new ushort[512 * 224];
            for (int y = 0; y < 224; y++) for (int x = 0; x < 512; x++) ours[y * 512 + x] = (ushort)(y * 600 + x & 0x7FFF);
            Assert.Equal((512 * 224, 0), SnesDifferential.Pictures(new SnesPicture(512, 224, ours), new SnesPicture(512, 478, wide), SnesDifferential.MesenRowOffset));
            ours[5 * 512 + 9] ^= 1;
            Assert.Equal((512 * 224, 1), SnesDifferential.Pictures(new SnesPicture(512, 224, ours), new SnesPicture(512, 478, wide), SnesDifferential.MesenRowOffset));
            var narrow = new ushort[256 * 224];
            for (int y = 0; y < 224; y++) for (int x = 0; x < 256; x++) narrow[y * 256 + x] = (ushort)(y * 600 + 2 * x & 0x7FFF);
            Assert.Equal((512 * 224, 256 * 224), SnesDifferential.Pictures(new SnesPicture(256, 224, narrow), new SnesPicture(512, 478, wide), SnesDifferential.MesenRowOffset));
            var tall = new ushort[512 * 448];
            for (int y = 0; y < 448; y++) for (int x = 0; x < 512; x++) tall[y * 512 + x] = (ushort)((y / 2) * 600 + x & 0x7FFF);
            Assert.Equal((512 * 448, 0), SnesDifferential.Pictures(new SnesPicture(512, 448, tall), new SnesPicture(512, 478, wide), SnesDifferential.MesenRowOffset));
            tall[3 * 512] ^= 1;
            Assert.Equal((512 * 448, 1), SnesDifferential.Pictures(new SnesPicture(512, 448, tall), new SnesPicture(512, 478, wide), SnesDifferential.MesenRowOffset));
        }

        // byuu's ROMs pass on a blue backdrop and fail on a red one, where their folder's source writes both colours.
        [Fact]
        public void A_backdrop_verdict_counts_only_where_the_source_writes_it()
        {
            string dir = Path.Combine(Path.GetTempPath(), $"backdrop-{Guid.NewGuid():N}", "jonasquinn-test-roms", "t");
            Directory.CreateDirectory(dir);
            try
            {
                string rom = Path.Combine(dir, "test_nmi.smc"), other = Path.Combine(Path.GetDirectoryName(dir)!, "u", "x.smc");
                File.WriteAllText(Path.Combine(dir, "test_nmi.asm"), "pass() {\n  lda #$00 : sta $2122\n  lda #$7c : sta $2122\n}\nfail() {\n  lda #$1f : sta $2122\n}\n");
                Assert.Equal(SnesOutcome.Passed, SnesTestRomGrader.Grade(rom, RunWith(rom, Vram(), new byte[] { 0x00, 0x7C })).Outcome);
                Assert.Equal(SnesOutcome.Failed, SnesTestRomGrader.Grade(rom, RunWith(rom, Vram(), new byte[] { 0x1F, 0x00 })).Outcome);
                Assert.Equal(SnesOutcome.Visual, SnesTestRomGrader.Grade(rom, RunWith(rom, Vram(), new byte[] { 0x10, 0x42 })).Outcome);
                Assert.Equal(SnesOutcome.Visual, SnesTestRomGrader.Grade(other, RunWith(other, Vram(), new byte[] { 0x00, 0x7C })).Outcome);
            }
            finally
            {
                Directory.Delete(Path.GetDirectoryName(Path.GetDirectoryName(dir))!, true);
            }
        }

        // PeterLemon's tests pass by showing PASS on a screen that has stopped changing, and fail by showing FAIL at all.
        [Fact]
        public void A_peterlemon_test_passes_only_on_a_steady_screen()
        {
            string dir = Path.Combine(Path.GetTempPath(), $"venusrt-grader-{Environment.ProcessId}");
            Directory.CreateDirectory(dir);
            try
            {
                string rom = Path.Combine(dir, "CPUADC.sfc");
                File.WriteAllText(Path.Combine(dir, "CPUADC.asm"), "PrintText(Pass, 8, 32)\n");
                byte[] pass = Vram((0x100, "PASS")), fail = Vram((0x100, "PASS"), (0x120, "FAIL"));
                Assert.Equal(SnesOutcome.Passed, SnesTestRomGrader.Grade(rom, RunWith(rom, pass, earlier: pass)).Outcome);
                Assert.Equal(SnesOutcome.Incomplete, SnesTestRomGrader.Grade(rom, RunWith(rom, pass, earlier: Vram())).Outcome);
                Assert.Equal(SnesOutcome.Failed, SnesTestRomGrader.Grade(rom, RunWith(rom, fail, earlier: fail)).Outcome);
            }
            finally { Directory.Delete(dir, true); }
        }

        [Fact]
        public void The_differential_counts_every_differing_byte_and_pixel_and_nothing_else()
        {
            var px = new ushort[256 * 225];
            px[256 * 3 + 7] = 0x001F;
            var shifted = new ushort[256 * 224];
            shifted[256 * 2 + 7] = 0x801F;
            var a = new SnesRun("a", new[] { new SnesSnapshot(1, new Dictionary<string, byte[]> { ["vram"] = new byte[] { 1, 2, 3 } }, new SnesPicture(256, 224, shifted)) }, 0, Array.Empty<short>());
            var b = new SnesRun("b", new[] { new SnesSnapshot(1, new Dictionary<string, byte[]> { ["vram"] = new byte[] { 1, 9, 3 } }, new SnesPicture(256, 225, px)) }, 0, Array.Empty<short>());
            var d = SnesDifferential.Compare(a, b, secondRowOffset: 1).Single();
            Assert.Equal(1, d.SpaceBytes["vram"]);
            Assert.Equal((256 * 224, 0), (d.PixelsCompared, d.PixelsDiffering));
            Assert.Equal(2, SnesDifferential.Compare(a, b).Single().PixelsDiffering);
            Assert.True(SnesDifferential.Compare(a, a).Single().Identical);

            short[] tone = Enumerable.Range(0, 32000 * 2).Select(i => (short)((i / 2) % 3200 < 1600 && (i / 2) / 3200 % 2 == 0 ? 8000 : 0)).ToArray();
            short[] later = new short[533 * 2 * 5].Concat(tone).ToArray()[..tone.Length];
            var s = SnesDifferential.Audio(new SnesRun("a", Array.Empty<SnesSnapshot>(), 32000, tone), new SnesRun("b", Array.Empty<SnesSnapshot>(), 32000, later));
            Assert.True(s.Correlation > 0.95, $"{s}");
            Assert.InRange(s.LagWindows, 4, 6);
            Assert.True(SnesDifferential.Audio(new SnesRun("a", Array.Empty<SnesSnapshot>(), 32000, new short[64000]), new SnesRun("b", Array.Empty<SnesSnapshot>(), 32000, tone)).FirstSilent);

            short[] Chirp(int rate, int seconds, double amp, int delay) => Enumerable.Range(0, rate * seconds).SelectMany(i => { double t = (double)(i - delay) / rate; short v = (short)(i < delay ? 0 : amp * Math.Sin(2 * Math.PI * (100 * t + 90 * t * t))); return new[] { v, v }; }).ToArray();
            var at32 = new SnesRun("a", Array.Empty<SnesSnapshot>(), 32000, Chirp(32000, 4, 8000, 0));
            var at48 = new SnesRun("b", Array.Empty<SnesSnapshot>(), 48000, Chirp(48000, 4, 4000, 30));
            var w = SnesDifferential.Samples(at32, at48);
            Assert.True(w.Correlation > 0.999 && w.Least > 0.99 && w.RelativeError < 0.05, $"{w}");
            Assert.Equal((20, 20), (w.LagFrom, w.LagTo));
            Assert.InRange(w.Gain, 1.98, 2.02);
        }

        [Fact]
        public void VenusRTs_library_claims_nothing_it_does_not_export_and_its_state_is_its_own()
        {
            Assert.True(VenusNative.Available, VenusNative.Report);
            Assert.Equal(("venusrt", "EMUSEN_VENUS_NATIVE", "venusrt_crash"), (VenusNative.Library.Crate, VenusNative.Library.Variable, VenusNative.Library.CrashLogStem));
            Assert.Equal(0ul, VenusNative.Library.Capabilities);
            nint handle = NativeLibrary.Load(Path.Combine(AppContext.BaseDirectory, VenusNative.Library.FileName));
            foreach (string name in NativeInterface.Required) Assert.True(NativeLibrary.TryGetExport(handle, name, out _), name);
            foreach (var (_, name, exports) in NativeInterface.Optional)
                foreach (string export in exports) Assert.False(NativeLibrary.TryGetExport(handle, export, out _), $"{name}: {export}");

            using var m = new VenusMachine(new byte[0x8000 + 512]);
            for (int i = 0; i < 3; i++) m.Advance();
            Assert.Equal(3, m.TotalFrames);
            Assert.Equal((256, 224), (m.FrameInfo.Width, m.FrameInfo.Height));
            Assert.Equal(32000, m.AudioSampleRate);
            Assert.Equal(new[] { 0x10000, 0x200, 0x220, 0x20000, 0x10000 }, SnesEngine.Spaces.Select(n => m.ReadSpace(n).Length));
            byte[] state = m.Save();
            Assert.Equal("VNRT"u8.ToArray(), state[..4]);
            state[0] = (byte)'S'; state[1] = (byte)'N'; state[2] = (byte)'E'; state[3] = (byte)'S';
            Assert.Throws<InvalidDataException>(() => m.Load(state));
            Assert.Throws<InvalidDataException>(() => new VenusMachine(new byte[0x4000]));
        }

        [Fact]
        public void The_runner_drives_VenusRT_to_the_frames_asked_for()
        {
            string rom = Path.Combine(Path.GetTempPath(), $"venusrt-runner-{Environment.ProcessId}.sfc");
            File.WriteAllBytes(rom, SyntheticRom.Build());
            try
            {
                var run = new VenusRtSnesEngine().Run(rom, new[] { 20, 10 }, new[] { new SnesPress(2, EmuSen.Galaxia.Input.PadButton.Start) }, audio: true);
                Assert.Equal(new[] { 10, 20 }, run.Snapshots.Select(s => s.Frame));
                Assert.Equal(SnesOutcome.Visual, SnesTestRomGrader.Grade(rom, run).Outcome);
                Assert.True(SnesDifferential.Compare(run, run).All(d => d.Identical));
            }
            finally { File.Delete(rom); }
        }

        // A boot image handed to the runner is the one the SPC700 starts from: the open program writes $AA and $BB through $F4 and $F5 to the RAM beneath, a synthetic loop at its own vector does not.
        [Fact]
        public void The_runner_starts_VenusRT_from_a_boot_image_when_given_one()
        {
            string rom = Path.Combine(Path.GetTempPath(), $"venusrt-boot-{Environment.ProcessId}.sfc");
            File.WriteAllBytes(rom, SyntheticRom.Build());
            try
            {
                byte[] loop = new byte[64];
                (loop[0], loop[1], loop[62], loop[63]) = (0x2F, 0xFE, 0xC0, 0xFF);
                byte[] own = new VenusRtSnesEngine(dspFirmware: false, boot: null).Run(rom, new[] { 10 }).Snapshots[0].Spaces["apuram"];
                byte[] given = new VenusRtSnesEngine(dspFirmware: false, boot: loop).Run(rom, new[] { 10 }).Snapshots[0].Spaces["apuram"];
                Assert.Equal((0xAA, 0xBB), (own[0xF4], own[0xF5]));
                Assert.NotEqual((0xAA, 0xBB), (given[0xF4], given[0xF5]));
            }
            finally { File.Delete(rom); }
        }

        // Mesen's picture rows against a 224-line engine's: the offset that makes steady pictures agree is the one the differential uses.
        [Fact]
        public void The_picture_offset_is_the_one_steady_pictures_agree_at()
        {
            string? root = SnesTestRomCorpus.Root;
            if (root is null || !MesenProbeSnesEngine.Available)
            {
                _output.WriteLine($"{SnesTestRomCorpus.Variable} unset or no probe, not run");
                return;
            }
            CoreOptions.BatteryRamDisabled = true;
            var mesen = new MesenProbeSnesEngine(Path.Combine(root, "runs", "mesen"));
            var roms = new[] { "roms/240pSuite-SNES-1.03/240pSuite.sfc", "roms/gilyon-snes-tests-v1.4/spctest/spctest.sfc", "src/higan-snes-test-roms/PeterLemon/SNES-CPUTest-CPU/ADC/CPUADC.sfc" };
            var best = new List<int>();
            foreach (string rom in roms.Where(r => File.Exists(Path.Combine(root, r))))
            {
                string path = Path.Combine(root, rom);
                var a = ICoreSnesEngine.Venus().Run(path, new[] { 1800, 3600 });
                var b = mesen.Run(path, new[] { 1800, 3600 });
                var scores = Enumerable.Range(0, 16).Select(o => SnesDifferential.Pictures(a.Snapshots[1].Picture, b.Snapshots[1].Picture, o)).ToList();
                int at = scores.Select((s, o) => (s.Differing, o)).Min().o;
                _output.WriteLine($"{rom}: best offset {at}, {scores[at].Differing} of {scores[at].Compared} differ; offset 0: {scores[0].Differing}");
                best.Add(at);
            }
            Assert.NotEmpty(best);
            Assert.All(best, o => Assert.Equal(SnesDifferential.MesenRowOffset, o));
        }

        // Opt-in: each game in EMUSEN_VENUSRT_GAMES against Mesen every tenth frame, and where each space and the picture first part.
        [Fact]
        public void The_games_pictures_and_spaces_against_Mesen()
        {
            string? dir = Environment.GetEnvironmentVariable("EMUSEN_VENUSRT_GAMES");
            string? root = SnesTestRomCorpus.Root;
            if (dir is null || root is null || !MesenProbeSnesEngine.Available)
            {
                _output.WriteLine("EMUSEN_VENUSRT_GAMES, the corpus or the probe unset, not run");
                return;
            }
            int until = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_VENUSRT_GAMES_FRAMES"), out int l) ? l : 600;
            var frames = Enumerable.Range(1, until / 10).Select(i => i * 10).ToArray();
            var mesen = new MesenProbeSnesEngine(Path.Combine(root, "runs", "mesen"));
            var table = new List<string> { "game\tmesen_lit\tvenusrt_lit\tvram\tcgram\toam\tpicture\tpictures_differing\taudio\tsamples\tat_last" };
            foreach (string rom in Directory.GetFiles(dir).Order(StringComparer.Ordinal))
            {
                SnesRun theirs = mesen.Run(rom, frames, audio: true), ours = new VenusRtSnesEngine().Run(rom, frames, audio: true);
                var sound = SnesDifferential.Audio(ours, theirs);
                var wave = SnesDifferential.Samples(ours, theirs);
                var d = SnesDifferential.Compare(ours, theirs, SnesDifferential.MesenRowOffset);
                static string Lit(SnesRun r) => r.Snapshots.FirstOrDefault(s => s.Picture is { } p && p.Pixels.Any(x => (x & 0x7FFF) != 0))?.Frame.ToString() ?? "never";
                string First(Func<SnesFrameDiff, int> n) => d.FirstOrDefault(f => n(f) != 0)?.Frame.ToString() ?? "never";
                var last = d[^1];
                table.Add(string.Join('\t', Path.GetFileName(rom), Lit(theirs), Lit(ours), First(f => f.SpaceBytes.GetValueOrDefault("vram")), First(f => f.SpaceBytes.GetValueOrDefault("cgram")),
                    First(f => f.SpaceBytes.GetValueOrDefault("oam")), First(f => f.PixelsDiffering), $"{d.Count(f => f.PixelsDiffering != 0)} of {d.Count}", $"r {sound.Correlation:F3} lag {sound.LagWindows}{(sound.FirstSilent ? " venusrt silent" : "")}{(sound.SecondSilent ? " mesen silent" : "")}", $"r {wave.Correlation:F4} least {wave.Least:F4} lag {wave.LagFrom}..{wave.LagTo} gain {wave.Gain:F3} error {wave.RelativeError:F4} over {wave.Seconds}s",
                    $"vram {last.SpaceBytes.GetValueOrDefault("vram")}B cgram {last.SpaceBytes.GetValueOrDefault("cgram")}B oam {last.SpaceBytes.GetValueOrDefault("oam")}B picture {last.PixelsDiffering}/{last.PixelsCompared}px"));
            }
            foreach (string line in table) _output.WriteLine(line);
            if (Environment.GetEnvironmentVariable(ReportVariable) is { } report) File.WriteAllLines(report, table);
        }

        // The corpus over C# Venus and Mesen, graded by each suite's protocol, against the committed baseline table.
        [Fact]
        public void The_corpus_reproduces_the_recorded_baseline()
        {
            string? root = SnesTestRomCorpus.Root;
            if (root is null)
            {
                _output.WriteLine($"{SnesTestRomCorpus.Variable} unset, not run");
                return;
            }
            CoreOptions.BatteryRamDisabled = true;
            bool mesen = MesenProbeSnesEngine.Available && Environment.GetEnvironmentVariable("EMUSEN_VENUSRT_NO_MESEN") != "1";
            var engines = new List<ISnesEngine> { ICoreSnesEngine.Venus() };
            if (mesen) engines.Add(new MesenProbeSnesEngine(Path.Combine(root, "runs", "mesen")));
            int threads = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_VENUSRT_THREADS"), out int t) ? t : 4;
            // VenusRT as a third column when asked for; its verdicts are reported, not compared with the baseline.
            bool venusRt = Environment.GetEnvironmentVariable("EMUSEN_VENUSRT_ENGINE") == "1";

            var roms = SnesTestRomCorpus.Unique(root);
            var rows = new ConcurrentDictionary<string, string>();
            Parallel.ForEach(roms, new ParallelOptions { MaxDegreeOfParallelism = threads }, rom =>
            {
                string path = Path.Combine(root, rom.Path);
                var frames = SnesTestRomCorpus.Frames(rom.Path);
                var cells = new List<string> { rom.Md5, rom.Path, SnesTestRomCorpus.Suite(rom.Path) };
                var runs = new List<SnesRun>();
                foreach (var engine in engines)
                {
                    try
                    {
                        var run = engine.Run(path, frames);
                        runs.Add(run);
                        var v = SnesTestRomGrader.Grade(path, run);
                        cells.Add(v.Outcome.ToString());
                        cells.Add(v.SelfGraded || v.Outcome == SnesOutcome.Done ? $"{v.Protocol} {v.Detail}".Trim() : "");
                    }
                    catch (Exception e)
                    {
                        cells.Add("Threw");
                        cells.Add(e.GetType().Name);
                    }
                }
                if (runs.Count == 2)
                {
                    var d = SnesDifferential.Compare(runs[0], runs[1], SnesDifferential.MesenRowOffset);
                    var last = d.LastOrDefault();
                    cells.Add(last is null ? "n/a" : last.SpaceBytes.GetValueOrDefault("vram") == 0 ? "same" : $"{last.SpaceBytes["vram"]}B");
                    var mesenRun = runs[1];
                    bool steady = mesenRun.Snapshots.Count == 2 && mesenRun.Snapshots[0].Spaces["vram"].AsSpan().SequenceEqual(mesenRun.Snapshots[1].Spaces["vram"]);
                    cells.Add(steady ? "steady" : "moving");
                }
                if (venusRt)
                {
                    try
                    {
                        var run = new VenusRtSnesEngine().Run(path, frames);
                        var v = SnesTestRomGrader.Grade(path, run);
                        cells.Add(v.Outcome.ToString());
                        cells.Add(v.SelfGraded || v.Outcome == SnesOutcome.Done ? $"{v.Protocol} {v.Detail}".Trim() : "");
                        var against = runs.Count == 2 ? SnesDifferential.Compare(run, runs[1], SnesDifferential.MesenRowOffset).LastOrDefault() : null;
                        foreach (string space in new[] { "vram", "cgram", "oam" })
                            cells.Add(against is null ? "n/a" : against.SpaceBytes.GetValueOrDefault(space) == 0 ? "same" : $"{against.SpaceBytes[space]}B");
                        cells.Add(against is null || against.PixelsCompared == 0 ? "n/a" : against.PixelsDiffering == 0 ? "same" : $"{against.PixelsDiffering}px");
                    }
                    catch (Exception e)
                    {
                        cells.Add("Threw");
                        cells.Add(e.GetType().Name);
                        cells.AddRange(new[] { "n/a", "n/a", "n/a", "n/a" });
                    }
                }
                rows[rom.Md5] = string.Join('\t', cells);
            });

            string header = "md5\trom\tsuite\tvenus\tvenus_detail" + (mesen ? "\tmesen\tmesen_detail\tvram_last\tmesen_vram" : "") + (venusRt ? "\tvenusrt\tvenusrt_detail\tvenusrt_vram\tvenusrt_cgram\tvenusrt_oam\tvenusrt_picture" : "");
            string table = header + "\n" + string.Join("\n", roms.Select(r => rows[r.Md5])) + "\n";
            if (Environment.GetEnvironmentVariable(ReportVariable) is { } report) File.WriteAllText(report, table);

            var lines = table.Split('\n', StringSplitOptions.RemoveEmptyEntries).Skip(1).Select(l => l.Split('\t')).ToList();
            foreach (var g in lines.GroupBy(c => c[2]).OrderBy(g => g.Key, StringComparer.Ordinal))
                _output.WriteLine($"{g.Key}: {g.Count()} ROMs, Venus {g.Count(c => c[3] == "Passed")} passed" + (mesen ? $", Mesen {g.Count(c => c[5] == "Passed")}" : ""));

            string baseline = Path.Combine(AppContext.BaseDirectory, "Cores", "VenusRtBaseline.tsv");
            if (!mesen || !File.Exists(baseline)) return;
            var want = File.ReadAllLines(baseline).Skip(1).Select(l => l.Split('\t')).ToDictionary(c => c[0]);
            var differences = lines.Where(c => !want.TryGetValue(c[0], out var w) || w[3] != c[3] || w[5] != c[5] || (c[3] != "Visual" && w[4] != c[4]) || (c[5] != "Visual" && w[6] != c[6]))
                .Select(c => $"{c[1]}: {c[3]}/{c[5]}").ToList();
            Assert.True(differences.Count == 0, string.Join("\n", differences));
        }
    }
}
