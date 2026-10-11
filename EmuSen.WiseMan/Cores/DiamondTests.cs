using System.Diagnostics;
using System.Text;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Psx;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Fixtures.RomRunner;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Diamond through the v1 adapter, discovery, the generic test-ROM runner with its TTY reader and the conformance kit, and the synthetic images it runs on - see Diamond_Native.md §2-§5 and §11.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamond_" + Guid.NewGuid().ToString("N"));

        public DiamondTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(true);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        public static string LibraryPath => CoreAdapterTests.LibraryPath("diamond");

        private string Write(string name, byte[] image)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        [Fact]
        public void The_library_is_on_the_core_abi_and_its_system_is_the_packs()
        {
            var library = CoreLibrary.Open(LibraryPath);
            Assert.True(library.Available, library.Report);
            Assert.Equal(("diamond", "Diamond", 0ul), (library.Info.Id, library.Info.DisplayName, library.Capabilities));
            var system = Assert.Single(library.Info.Systems);
            Assert.Equal((PsxSystem.Id, true), (system.Id, system.Development));
            Assert.Same(PsxSystem.Entry, SystemPacks.For(system.Id)!.Entry);
            Assert.Equal(PsxSystem.Entry.Extensions, system.Extensions);
            Assert.Empty(system.Extensions.Intersect(PsxSystem.DiscExtensions));
            Assert.Equal(PsxSystem.Bios.Select(b => (b.Name, b.Size)), system.Firmware.Select(f => (f.Name, f.Size)));
            Assert.All(system.Firmware, f => Assert.False(f.Required));
            Assert.All(system.Firmware, f => Assert.Equal("accuracy", f.Replacement?.Effect));
            Assert.Equal(new[] { (PsxSystem.DigitalPad, 14, 0), (PsxSystem.DualShock, 16, 4) }, system.Controllers.Select(c => (c.Id, c.Buttons.Count, c.Axes.Count)));
            Assert.Equal(new[] { "region", "gpu", "pad1", "pad2" }, library.Settings.Select(s => s.Key));

            Assert.Equal("scph5501.bin", library.FirmwareFor(SyntheticPsExe.Build()).Single().Name);
            Assert.Equal("scph5500.bin", library.FirmwareFor(SyntheticPsxDisc.Build("Japan").DataTrack).Single().Name);
            Assert.Equal("scph5502.bin", library.FirmwareFor(SyntheticPsxDisc.Build("Europe").Iso).Single().Name);
            Assert.Empty(library.FirmwareFor(new byte[4096]));
        }

        [Fact]
        public void A_machine_shows_the_packs_screen_and_runs_as_its_market()
        {
            var library = CoreLibrary.Open(LibraryPath);
            foreach (var (image, region, height, hz) in new (byte[], string, int, double)[]
            {
                (SyntheticPsExe.Build(), "ntsc", 240, 59.8261),
                (SyntheticPsxDisc.Build("Europe").DataTrack, "pal", 256, 49.7614),
                (SyntheticPsxDisc.Build("Japan", mode: 1).DataTrack, "ntsc", 240, 59.8261),
            })
            {
                using var machine = new CoreMachine(library, image, "", Array.Empty<(uint, byte[])>());
                var info = machine.Info;
                Assert.Equal(("psx", region, height), (info.System, info.Region, info.BaseHeight));
                Assert.Equal(PsxSystem.DisplayAspect, (double)info.AspectNum / info.AspectDen, 9);
                Assert.Equal(hz, info.FrameRateHz, 4);
                Assert.Equal(new[] { "R3000", "GTE" }, info.Processors.Select(p => p.Name));
                Assert.Equal(44100, info.AudioRate);
            }
            var refused = Assert.Throws<CoreRefusedException>(() => new CoreMachine(library, new byte[70_000], "", Array.Empty<(uint, byte[])>()));
            _output.WriteLine(refused.Message);
        }

        [Fact]
        public void The_stub_runs_a_program_and_each_disc_form_through_the_generic_runner()
        {
            var engine = new CoreAbiTestRomEngine(LibraryPath);
            Assert.True(engine.Available, engine.Report);
            var disc = SyntheticPsxDisc.Build();
            foreach (var (name, image) in new (string, byte[])[]
            {
                ("program.psexe", SyntheticPsExe.Build()),
                ("disc (Track 1).bin", disc.DataTrack),
                ("disc.iso", disc.Iso),
                ("mode1 (Track 1).bin", SyntheticPsxDisc.Build(mode: 1).DataTrack),
            })
            {
                var run = engine.Run(Write(name, image), new[] { 1, 60 }, new[] { new RomPress(10, PadButton.Start) });
                var last = run.At(60)!;
                Assert.Equal(new[] { "bios", "ram", "scratch", "spuram", "tty", "vram" }, last.Spaces.Keys.Order());
                Assert.Equal((0x20_0000, 0x400, 0x10_0000, 0x8_0000, 0x1_0000), (last.Spaces["ram"].Length, last.Spaces["scratch"].Length, last.Spaces["vram"].Length, last.Spaces["spuram"].Length, last.Spaces["tty"].Length));
                Assert.Equal((320, 240), (last.Picture!.Width, last.Picture.Height));
                Assert.All(last.Picture.Rgb, b => Assert.Equal(0, b));
                Assert.Equal(44100, run.AudioRate);
                Assert.True(TestRomDifferential.Steady(run, 1, 60));
                Assert.Equal(RomOutcome.Visual, new TestRomGrader(Array.Empty<RomProtocol>()).Grade(name, run).Outcome);
            }
        }

        // A developer asking for development cores opens its files; its system says it runs no game yet, so the console gets no tab or shelf - see EmuSen_CoreAPI.md §27.4.
        [Fact]
        public void A_program_opens_on_the_discovered_engine_when_development_cores_are_asked_for()
        {
            Assert.Contains(CoreDiscovery.Found, c => c.Info.Id == "diamond");
            string exe = Write("program.psexe", SyntheticPsExe.Build(market: "Europe"));
            Assert.True(CoreFactory.IsSupported(exe));
            Assert.Null(CoreCatalog.ConsoleForRom(exe));
            Assert.DoesNotContain("PlayStation", CoreCatalog.DiscoveredConsoles);
            var bundle = CoreFactory.Load(exe);
            var core = Assert.IsType<CoreEngine>(bundle.Core);
            Assert.Equal(("diamond", "psx", "pal"), (core.Info.Id, core.Machine.Info.System, core.Machine.Info.Region));
            core.RunFrame();
            Assert.Equal((320, 256), (core.ScreenWidth, core.ScreenHeight));
            Assert.InRange(core.FrameRateHz, 49.76, 49.77);
            Assert.Null(bundle.CheatAutoDetectCodec);
            Assert.Null(bundle.Notice);
            (core as IDisposable)?.Dispose();
            Assert.False(CoreFactory.IsSupported("game.cue") || CoreFactory.IsSupported("game.chd"), "a disc's formats wait for routing by content");
        }

        // The core carries the development mark of EmuSen_CoreAPI.md §27 until the tester offers the PlayStation.
        [Fact]
        public void A_player_is_not_offered_the_core_in_development()
        {
            Assert.True(CoreSidecar.Read(CoreSidecar.PathFor(LibraryPath))?.Development, "the build writes the sidecar marked for development");
            CoreDiscovery.UseDevelopment(false);
            Assert.DoesNotContain(CoreDiscovery.Found, c => c.Info.Id == "diamond");
            Assert.All(PsxSystem.Entry.Extensions, e => Assert.False(CoreFactory.IsSupported("program" + e)));
            Assert.DoesNotContain("PlayStation", CoreCatalog.DiscoveredConsoles);
            Assert.Empty(CoreCatalog.SettingsFor("PlayStation"));
        }

        // The gate of Diamond_Plan.md §7's stage 1: the kit's fifteen cases on a synthetic program and a synthetic disc, single-image form.
        [Fact]
        public void The_kit_passes_on_a_synthetic_program_and_a_synthetic_disc()
        {
            string runner = Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-conform.exe" : "emusen-core-conform");
            Assert.True(File.Exists(runner), $"no kit runner at {runner}");
            foreach (var (name, image) in new[] { ("program.psexe", SyntheticPsExe.Build()), ("disc (Track 1).bin", SyntheticPsxDisc.Build().DataTrack) })
            {
                var start = new ProcessStartInfo(runner) { RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (string a in new[] { "--core", LibraryPath, "--image", Write(name, image), "--frames", "120" }) start.ArgumentList.Add(a);
                using var process = Process.Start(start)!;
                var stderr = process.StandardError.ReadToEndAsync();
                string output = process.StandardOutput.ReadToEnd();
                process.WaitForExit();
                _output.WriteLine($"{name}:\n{output}{stderr.Result}");
                Assert.True(process.ExitCode == 0, output + stderr.Result);
                Assert.Equal(15, output.Split('\n').Count(l => l.StartsWith("pass C")));
            }
        }

        // A program that prints through the DEV8 DUART, as the open kernel's TTY will.
        private static byte[] Printing(string text)
        {
            var words = new List<uint> { 0x3C08_1F80, 0x3508_2023 };
            foreach (char c in text) words.AddRange(new[] { 0x2409_0000u | c, 0xA109_0000u });
            uint loop = SyntheticPsExe.Origin + 4 * (uint)words.Count;
            words.AddRange(new[] { 0x0800_0000u | ((loop >> 2) & 0x03FF_FFFF), 0u });
            return SyntheticPsExe.Build(SyntheticPsExe.Words(words.ToArray()));
        }

        // The TTY reader of Diamond_Plan.md §3.3: the text in the tty space, graded by a ps1-tests protocol as data.
        [Fact]
        public void The_tty_reader_grades_pass_and_fail_lines_a_program_prints()
        {
            var engine = new CoreAbiTestRomEngine(LibraryPath);
            var cop = Ps1Tests.Programs.Single(p => p.Path == "cpu/cop/cop.exe");
            var grader = new TestRomGrader(new[] { Ps1Tests.Protocol(cop, _ => null) });
            foreach (var (text, outcome) in new[] { ("boot\ncpu/cop\npass - a\npass - b\nDone.\n", RomOutcome.Passed), ("cpu/cop\npass - a\nfail - b:1\nDone.\n", RomOutcome.Failed), ("cpu/cop\npass - a\n", RomOutcome.Incomplete) })
            {
                string path = Write(Path.Combine("cpu", "cop", "cop.exe").Replace(Path.DirectorySeparatorChar, '_'), Printing(text));
                var run = engine.Run(path, new[] { 2 });
                Assert.Equal(text, TtyReader.Text(run.At(2)!));
                var verdict = Ps1Tests.Grade(cop, run, _ => null);
                Assert.Equal(outcome, verdict.Outcome);
                _output.WriteLine($"{outcome}: {verdict.Detail}");
            }
            Assert.Equal(RomOutcome.Visual, grader.Grade("other.exe", engine.Run(Write("other.exe", Printing("x")), new[] { 1 })).Outcome);
        }

        private static string? Corpus => Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_PS1TESTS") is { } c && Directory.Exists(c) ? c : null;

        // Each of ps1-tests' graded programs run to frame 900 on `files`, and those that did not pass.
        private List<string> RunPs1Tests(string corpus, IReadOnlyList<(uint, byte[])> files)
        {
            var engine = new CoreAbiTestRomEngine(LibraryPath, _ => files, rom => Ps1Tests.Programs.FirstOrDefault(p => rom.Replace('\\', '/').EndsWith(p.Path))?.Settings ?? "");
            string? Log(string program) => Path.Combine(corpus, Path.GetDirectoryName(program)!, "psx.log") is var l && File.Exists(l) ? File.ReadAllText(l) : null;
            var failed = new List<string>();
            foreach (var program in Ps1Tests.Programs)
            {
                var verdict = Ps1Tests.Grade(program, engine.Run(Path.Combine(corpus, program.Path), new[] { 900 }), Log);
                _output.WriteLine($"{program.Path}: {verdict.Outcome} {verdict.Detail}");
                if (verdict.Outcome != RomOutcome.Passed) failed.Add(program.Path);
            }
            return failed;
        }

        // The gate of Diamond_Plan.md §7's stage 3: ps1-tests by their TTY on the open kernel with no firmware folder; opt-in by the corpus - see Diamond_Native.md §11.14.
        [Fact]
        public void Ps1_tests_pass_by_their_tty_on_the_open_kernel()
        {
            if (Corpus is not { } corpus)
            {
                _output.WriteLine("skipped: EMUSEN_DIAMOND_PS1TESTS names ps1-tests' release");
                return;
            }
            // Waiting on the kernel: its printf prints access-time's "%2-d" as "-d", so the table's hundredths are lost.
            Assert.Equal(new[] { "cpu/access-time/access-time.exe" }, RunPs1Tests(corpus, Array.Empty<(uint, byte[])>()));
        }

        // The same on a player's BIOS, the development path of Diamond_Plan.md §5.9's option D; opt-in by the corpus and a firmware folder.
        [Fact]
        public void Ps1_tests_pass_by_their_tty_on_a_players_bios()
        {
            string? firmware = Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_FIRMWARE");
            if (Corpus is not { } corpus || firmware is null || !Directory.Exists(firmware))
            {
                _output.WriteLine("skipped: EMUSEN_DIAMOND_PS1TESTS and EMUSEN_DIAMOND_FIRMWARE name ps1-tests' release and a folder of the player's own BIOS");
                return;
            }
            var declared = CoreLibrary.Open(LibraryPath).Info.Systems.Single().Firmware;
            var bios = declared.Where(f => File.Exists(Path.Combine(firmware, f.Name))).Select(f => (f.Which, File.ReadAllBytes(Path.Combine(firmware, f.Name)))).ToArray();
            Assert.Empty(RunPs1Tests(corpus, bios));
        }

        [Fact]
        public void The_ps_exe_builder_writes_the_documented_header()
        {
            var code = SyntheticPsExe.Words(0x2408_0001, 0x0800_4000, 0);
            var exe = SyntheticPsExe.Build(code, "Japan", pc: 0x8001_0000, load: 0x8001_0000);
            Assert.Equal("PS-X EXE", Encoding.ASCII.GetString(exe, 0, 8));
            Assert.Equal((0x8001_0000u, 0x8001_0000u, 0x800u, 0x801F_FFF0u), (BitConverter.ToUInt32(exe, 0x10), BitConverter.ToUInt32(exe, 0x18), BitConverter.ToUInt32(exe, 0x1C), BitConverter.ToUInt32(exe, 0x30)));
            Assert.Equal(0x1000, exe.Length);
            Assert.StartsWith("Sony Computer Entertainment Inc. for Japan area", Encoding.ASCII.GetString(exe, 0x4C, 60));
            Assert.Equal(code, exe[0x800..0x80C]);
            Assert.Equal(0x0800_0000u | (0x8001_0000u >> 2 & 0x03FF_FFFF), BitConverter.ToUInt32(SyntheticPsExe.Build(), 0x800));
            Assert.All(SyntheticPsExe.Build(market: null)[0x4C..0x800], b => Assert.Equal(0, b));
        }

        // ECMA-130's EDC divides its sector, Annex A's checks hold on every P- and Q-vector, and one changed byte breaks them.
        [Fact]
        public void The_disc_builders_sectors_are_the_ones_ecma_130_and_psx_spx_describe()
        {
            var data = Enumerable.Range(0, CdSector.User).Select(i => (byte)(i * 7 + 3)).ToArray();
            var mode1 = CdSector.Mode1(16, data);
            Assert.Equal(new byte[] { 0x00, 0x02, 0x16, 1 }, mode1[12..16]);
            Assert.Equal(0u, CdSector.Edc(mode1.AsSpan(0, 2068)));
            Assert.Equal(0, CdSector.Syndromes(mode1));
            Assert.All(mode1[2068..2076], b => Assert.Equal(0, b));

            var form1 = CdSector.Mode2Form1(4500, data, 0x89);
            Assert.Equal(new byte[] { 0x01, 0x02, 0x00, 2 }, form1[12..16]);
            Assert.Equal(form1[16..20], form1[20..24]);
            Assert.Equal(0u, CdSector.Edc(form1.AsSpan(16, 2060)));
            var cleared = (byte[])form1.Clone();
            Array.Clear(cleared, 12, 4);
            Assert.Equal(0, CdSector.Syndromes(cleared));
            Assert.NotEqual(0, CdSector.Syndromes(form1));

            var form2 = CdSector.Mode2Form2(12, new byte[CdSector.Form2User]);
            Assert.Equal(0x20, form2[18] & 0x20);
            Assert.Equal(0u, CdSector.Edc(form2.AsSpan(16, 2336)));

            mode1[600] ^= 1;
            Assert.NotEqual(0u, CdSector.Edc(mode1.AsSpan(0, 2068)));
            Assert.NotEqual(0, CdSector.Syndromes(mode1));
        }

        [Fact]
        public void The_synthetic_disc_carries_its_system_area_its_volume_and_its_files()
        {
            var exe = SyntheticPsExe.Build();
            var disc = SyntheticPsxDisc.Build("Americas", 2, exe);
            int sectors = disc.Iso.Length / CdSector.User;
            Assert.Equal(sectors * CdSector.Raw, disc.DataTrack.Length);
            Assert.Equal("          Licensed  by          Sony Computer Entertainment Amer  ica ", Encoding.ASCII.GetString(disc.Iso, 4 * CdSector.User, 70));
            byte[] Sector(int block) => disc.Iso[(block * CdSector.User)..((block + 1) * CdSector.User)];
            var pvd = Sector(16);
            Assert.Equal((1, "CD001", "PLAYSTATION"), ((int)pvd[0], Encoding.ASCII.GetString(pvd, 1, 5), Encoding.ASCII.GetString(pvd, 8, 11)));
            Assert.Equal((uint)sectors, BitConverter.ToUInt32(pvd, 80));
            Assert.Equal((255, "CD001"), ((int)Sector(17)[0], Encoding.ASCII.GetString(Sector(17), 1, 5)));

            var root = Sector(BitConverter.ToInt32(pvd, 156 + 2));
            var files = new Dictionary<string, (int Extent, int Length)>();
            for (int at = 0; root[at] != 0; at += root[at])
                files[Encoding.ASCII.GetString(root, at + 33, root[at + 32])] = (BitConverter.ToInt32(root, at + 2), BitConverter.ToInt32(root, at + 10));
            var cnf = files["SYSTEM.CNF;1"];
            string config = Encoding.ASCII.GetString(disc.Iso, cnf.Extent * CdSector.User, cnf.Length);
            Assert.StartsWith($"BOOT = cdrom:\\{SyntheticPsxDisc.ExeName};1", config);
            var program = files[SyntheticPsxDisc.ExeName + ";1"];
            Assert.Equal(exe, disc.Iso[(program.Extent * CdSector.User)..(program.Extent * CdSector.User + program.Length)]);

            for (int block = 0; block < sectors; block++)
            {
                var raw = disc.DataTrack[(block * CdSector.Raw)..((block + 1) * CdSector.Raw)];
                Assert.Equal(CdSector.Msf(block), (raw[12], raw[13], raw[14]));
                if (block is >= 12 and < 16)
                {
                    Assert.Equal(0x20, raw[18] & 0x20);
                    continue;
                }
                Assert.Equal(Sector(block), raw[24..2072]);
                Array.Clear(raw, 12, 4);
                Assert.Equal(0, CdSector.Syndromes(raw));
            }
            var mode1 = SyntheticPsxDisc.Build(mode: 1);
            Assert.Equal(disc.Iso, mode1.Iso);
            Assert.Equal(1, mode1.DataTrack[15]);

            Assert.Equal((2 + 1) * 75 * CdSector.Raw, disc.AudioTrack.Length);
            Assert.All(disc.AudioTrack[..(150 * CdSector.Raw)], b => Assert.Equal(0, b));
            Assert.Equal((8000, -8000), ((int)BitConverter.ToInt16(disc.AudioTrack, 150 * CdSector.Raw), (int)BitConverter.ToInt16(disc.AudioTrack, 150 * CdSector.Raw + 2)));
            string cue = File.ReadAllText(disc.WriteSet(_root, "disc"));
            Assert.Contains("FILE \"disc (Track 1).bin\" BINARY\n  TRACK 01 MODE2/2352", cue);
            Assert.Contains("TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:02:00", cue);
            Assert.True(File.Exists(Path.Combine(_root, "disc (Track 2).bin")));
        }

        [Fact]
        public void The_pack_names_each_players_pad_from_the_port_settings()
        {
            Assert.Equal(new[] { PsxSystem.DualShock, PsxSystem.DualShock }, PsxSystem.PlayerControllers(_ => null));
            Assert.Equal(new[] { PsxSystem.DigitalPad }, PsxSystem.PlayerControllers(k => k == "pad1" ? PsxSystem.DigitalPad : "none"));
            Assert.False(PsxSystem.Entry.SelfDelimitingImages, "a disc does not record its own length, so C4 may not require a truncated one refused");
        }
    }
}
