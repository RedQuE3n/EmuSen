using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.DianaOS.DianaOS.Sys.Systems;
using EmuSen.DianaOS.DianaOS.Sys.Systems.Genesis;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Fixtures.RomRunner;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Nephrite's stage 0 through the generic test-ROM runner, the v1 adapter and discovery, and the runner's own comparisons - see Nephrite_Native.md §2-§3.
    [Collection(TestCollections.ProcessGlobals)]
    public class NephriteTests : IDisposable
    {
        public const string CorpusVariable = "EMUSEN_NEPHRITE_CORPUS";
        public static readonly TestRomCorpus Corpus = new(CorpusVariable);

        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenNephrite_" + Guid.NewGuid().ToString("N"));

        public NephriteTests(ITestOutputHelper output)
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

        public static string LibraryPath => CoreAdapterTests.LibraryPath("nephrite");

        private string Write(string name, byte[] image)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        [Fact]
        public void The_library_is_on_the_core_abi_and_its_systems_are_the_packs()
        {
            var library = CoreLibrary.Open(LibraryPath);
            Assert.True(library.Available, library.Report);
            Assert.Equal(("nephrite", "Nephrite", CoreInterface.CapRomPatches | CoreInterface.CapSettings | CoreInterface.CapDebug | CoreInterface.CapDebugRegisters | CoreInterface.CapDebugDisassemble), (library.Info.Id, library.Info.DisplayName, library.Capabilities));
            var packs = new[] { GenesisSystems.MegaDrive, GenesisSystems.MegaCd, GenesisSystems.S32x };
            Assert.Equal(packs.Select(p => p.Id), library.Info.Systems.Select(s => s.Id));
            foreach (var (pack, system) in packs.Zip(library.Info.Systems))
            {
                Assert.Equal(pack.Extensions, system.Extensions);
                Assert.Same(pack, SystemPacks.For(system.Id)!.Entry);
                Assert.All(system.Firmware, f => Assert.False(f.Required));
                Assert.Equal(new[] { 8, 12 }, system.Controllers.Select(c => c.Buttons.Count));
            }
            Assert.Empty(library.FirmwareFor(SyntheticMdRom.Cartridge()));
            Assert.Equal(new[] { "32X_G_BIOS.BIN", "32X_M_BIOS.BIN", "32X_S_BIOS.BIN" }, library.FirmwareFor(SyntheticMdRom.Cartridge("SEGA 32X")).Select(f => f.Name));
            Assert.Equal("bios_CD_J.bin", library.FirmwareFor(SyntheticMdRom.Disc("J")).Single().Name);
        }

        // The two buses are views, which the runner's snapshots leave out (EmuSen_CoreAPI.md §29).
        [Fact]
        public void The_stub_runs_each_system_through_the_generic_runner()
        {
            var engine = new CoreAbiTestRomEngine(LibraryPath);
            Assert.True(engine.Available, engine.Report);
            foreach (var (name, image, spaces) in new (string, byte[], string[])[]
            {
                ("game.md", SyntheticMdRom.Cartridge(saveBytes: 8192), new[] { "wram", "z80ram", "vram", "cram", "vsram", "sram", "rom" }),
                ("game.32x", SyntheticMdRom.Cartridge("SEGA 32X"), new[] { "wram", "z80ram", "vram", "cram", "vsram", "rom", "sdram", "framebuffer", "palette" }),
                ("game.iso", SyntheticMdRom.Disc(), new[] { "wram", "z80ram", "vram", "cram", "vsram", "prgram", "wordram", "pcmram", "bram" }),
            })
            {
                var run = engine.Run(Write(name, image), new[] { 1, 60 }, new[] { new RomPress(10, PadButton.Start) });
                var last = run.At(60)!;
                Assert.Equal(spaces.Order(), last.Spaces.Keys.Order());
                Assert.Equal((65536, 128, 80), (last.Spaces["wram"].Length, last.Spaces["cram"].Length, last.Spaces["vsram"].Length));
                Assert.Equal(name.EndsWith(".md") ? (256, 224) : (320, 224), (last.Picture!.Width, last.Picture.Height));
                Assert.All(last.Picture.Rgb, b => Assert.Equal(0, b));
                Assert.Equal(48000, run.AudioRate);
                Assert.True(TestRomDifferential.Steady(run, 1, 60));
                Assert.Equal(RomOutcome.Visual, new TestRomGrader(Array.Empty<RomProtocol>()).Grade(name, run).Outcome);
            }
            Assert.Equal(8192, engine.Run(Write("save.md", SyntheticMdRom.Cartridge(saveBytes: 8192)), new[] { 1 }).At(1)!.Spaces["sram"].Length);
        }

        [Fact]
        public void A_genesis_image_opens_on_the_discovered_engine_with_no_code_naming_it()
        {
            Assert.Contains(CoreDiscovery.Found, c => c.Info.Id == "nephrite");
            string rom = Write("game.gen", SyntheticMdRom.Cartridge(region: "E"));
            Assert.True(CoreFactory.IsSupported(rom));
            Assert.Equal("Genesis", CoreCatalog.ConsoleForRom(rom));
            var bundle = CoreFactory.Load(rom);
            var core = Assert.IsType<CoreEngine>(bundle.Core);
            Assert.Equal(("nephrite", "md", "pal"), (core.Info.Id, core.Machine.Info.System, core.Machine.Info.Region));
            core.RunFrame();
            Assert.Equal((256, 224), (core.ScreenWidth, core.ScreenHeight));
            Assert.InRange(core.FrameRateHz, 49.70, 49.71);
            Assert.Equal(("Action Replay", "Game Genie"), (bundle.CheatAutoDetectCodec!.Name, bundle.CheatExplicitCodec!.Name));
            Assert.Equal(((long?)0, (long?)0x3F_FFFF), (core.Machine.Info.PatchLow, core.Machine.Info.PatchHigh));
            Assert.Null(bundle.Notice);
            (core as IDisposable)?.Dispose();
        }

        // The Sega CD and the 32X are marked in development in the core's own info, so a shipped Nephrite would offer the Genesis alone; a developer asking for development cores can still open their files - see EmuSen_CoreAPI.md §27.4.
        [Fact]
        public void A_shipped_core_offers_only_the_systems_its_info_says_run()
        {
            string built = Path.Combine(_root, "shipped");
            Directory.CreateDirectory(built);
            string copy = Path.Combine(built, Path.GetFileName(LibraryPath));
            File.Copy(LibraryPath, copy);
            Assert.False(CoreSidecar.Write(copy).Development);
            CoreDiscovery.UseDirectories(new[] { built });
            try
            {
                CoreDiscovery.UseDevelopment(false);
                var info = CoreDiscovery.Found.Single().Info;
                Assert.Equal(new[] { false, true, true }, info.Systems.Select(s => s.Development));
                Assert.True(CoreFactory.IsSupported("game.md"));
                Assert.False(CoreFactory.IsSupported("game.iso") || CoreFactory.IsSupported("game.32x"));
                Assert.Equal(new[] { "Genesis" }, CoreCatalog.DiscoveredConsoles);
                Assert.Equal(new[] { "model", "region", "pad1", "pad2" }, CoreCatalog.SettingsFor("Genesis").Select(s => s.Key));
                Assert.Empty(CoreCatalog.SettingsFor("Sega CD"));

                CoreDiscovery.UseDevelopment(true);
                Assert.True(CoreFactory.IsSupported("game.iso") && CoreFactory.IsSupported("game.32x"));
                Assert.Equal(new[] { "Genesis" }, CoreCatalog.DiscoveredConsoles);
            }
            finally
            {
                CoreDiscovery.UseDirectories(null);
            }
        }

        // Decided 2026-10-07: the Genesis is offered to players; the Sega CD and the 32X stay marked in development in the core's info - see EmuSen_CoreAPI.md §27.4.
        [Fact]
        public void A_player_is_offered_the_genesis_and_not_the_sega_cd_or_the_32x()
        {
            var sidecar = CoreSidecar.Read(CoreSidecar.PathFor(LibraryPath))!;
            Assert.False(sidecar.Development);
            CoreDiscovery.UseDevelopment(false);
            Assert.Contains(CoreDiscovery.Found, c => c.Info.Id == "nephrite");
            foreach (string ext in new[] { ".md", ".gen", ".bin", ".smd" })
            {
                Assert.True(CoreFactory.IsSupported("game" + ext), ext);
                Assert.True(CoreCatalog.IsRomExtension(ext), ext);
                Assert.Contains(ext, EmuSen.Mistress.Library.RomLibrary.Extensions);
            }
            foreach (string ext in new[] { ".iso", ".32x" })
            {
                Assert.False(CoreFactory.IsSupported("game" + ext), ext);
                Assert.DoesNotContain(ext, EmuSen.Mistress.Library.RomLibrary.Extensions);
            }
            Assert.IsType<CoreEngine>(CoreFactory.Create(Write("game.gen", SyntheticMdRom.Cartridge())));
            Assert.Equal(new[] { "Genesis" }, CoreCatalog.DiscoveredConsoles);
            Assert.Contains(CoreCatalog.ShelvesInReleaseOrder, s => s.Name == "Genesis (Nephrite)" && s.EsdeSystem == "genesis");
            Assert.DoesNotContain(CoreCatalog.ShelvesInReleaseOrder, s => s.EsdeSystem is "segacd" or "sega32x");
            Assert.Equal(new[] { "model", "region", "pad1", "pad2" }, CoreCatalog.SettingsFor("Genesis").Select(s => s.Key));
            Assert.Equal(12, CoreCatalog.ButtonsFor("Genesis").Count);
            var firmware = FirmwareOverview.Build(new EmuSen.Galaxia.Models.GraphicsConfig());
            Assert.Contains(firmware, f => f.Name == "Sega Genesis / Mega Drive");
            Assert.DoesNotContain(firmware, f => f.Name is "Sega CD / Mega-CD" or "Sega 32X");
        }

        [Fact]
        public void Pictures_compare_exactly_and_up_to_a_colour_map()
        {
            var a = new RomPicture(4, 1, new byte[] { 0, 0, 0, 10, 10, 10, 10, 10, 10, 0, 0, 0 });
            var b = new RomPicture(4, 1, new byte[] { 1, 1, 1, 12, 12, 12, 12, 12, 12, 1, 1, 1 });
            Assert.Equal((4, 4), TestRomDifferential.Pictures(a, b));
            Assert.Equal(0, TestRomDifferential.OffColourMap(a, b));
            b.Rgb[9] = 12;
            Assert.Equal(1, TestRomDifferential.OffColourMap(a, b));
            var merged = new RomPicture(4, 1, new byte[] { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 });
            Assert.Equal(2, TestRomDifferential.OffColourMap(a, merged));
            Assert.Equal((0, 255, 0), RomPicture.FromProbe(new byte[] { 0xE0, 0x07 }, 1, 1, "Rgb565")!.At(0, 0) with { Item2 = (byte)(RomPicture.FromProbe(new byte[] { 0xE0, 0x07 }, 1, 1, "Rgb565")!.At(0, 0).G | 3) });
            Assert.Equal(((byte)3, (byte)2, (byte)1), RomPicture.FromProbe(new byte[] { 1, 2, 3, 0 }, 1, 1, "Xrgb8888")!.At(0, 0));
        }

        // Every ROM of the fetched corpus is taken by the stub and runs, or is refused with words; nothing is graded yet.
        [Fact]
        public void The_corpus_runs_on_the_stub()
        {
            var roms = Corpus.Unique();
            if (roms.Count == 0)
            {
                _output.WriteLine($"{CorpusVariable} is not set or holds no unique-roms.txt: not run");
                return;
            }
            var engine = new CoreAbiTestRomEngine(LibraryPath);
            int ran = 0, refused = 0;
            foreach (var (md5, path) in roms)
            {
                try
                {
                    var run = engine.Run(path, new[] { 60 });
                    Assert.NotNull(run.At(60)!.Picture);
                    ran++;
                }
                catch (CoreRefusedException e)
                {
                    refused++;
                    _output.WriteLine($"refused {md5} {Path.GetFileName(path)}: {e.Message}");
                }
            }
            _output.WriteLine($"{ran} ran, {refused} refused, of {roms.Count}");
            Assert.Equal(roms.Count, ran + refused);
        }

        public const string GamesVariable = "EMUSEN_NEPHRITE_GAMES";

        // The 68000's RAM and the picture against Genesis Plus GX's at frames 120 and 600 over every game in a folder, written to EMUSEN_NEPHRITE_REPORT - see Nephrite_Native.md §10, §14.
        [Fact]
        public void The_ram_against_genesis_plus_gx_at_anchors_over_the_games()
        {
            string? folder = Environment.GetEnvironmentVariable(GamesVariable);
            var gpgx = new LibretroProbeEngine("genesis_plus_gx", ReferenceCore("genesis_plus_gx"), Path.Combine(Path.GetDirectoryName(folder ?? "") ?? _root, "runs"), system: "megadrive", wordSwapped: new[] { "ram" });
            if (folder is null || !Directory.Exists(folder) || !gpgx.Available)
            {
                _output.WriteLine($"{GamesVariable} names no folder, or no probe or Genesis Plus GX core here: not run");
                return;
            }
            var nephrite = new CoreAbiTestRomEngine(LibraryPath);
            var frames = new[] { 120, 600 };
            var lines = new List<string> { "game\tequal_f120\tequal_f600\toffmap_f120\toffmap_f600" };
            var games = Directory.EnumerateFiles(folder).Where(p => Path.GetExtension(p).ToLowerInvariant() is ".bin" or ".md" or ".gen" or ".smd").Order().ToList();
            foreach (string game in games)
            {
                string row;
                try
                {
                    var diffs = TestRomDifferential.Compare(nephrite.Run(game, frames), gpgx.Run(game, frames), new Dictionary<string, string> { ["wram"] = "ram" });
                    row = string.Join("\t", frames.Select(f => diffs.FirstOrDefault(d => d.Frame == f) is { } d && d.SpaceBytes.TryGetValue("wram", out int n) ? (65536 - n).ToString() : "-"))
                        + "\t" + string.Join("\t", frames.Select(f => diffs.FirstOrDefault(d => d.Frame == f) is { PixelsCompared: > 0 } d ? d.PixelsOffMap.ToString() : "-"));
                }
                catch (Exception e) when (e is CoreRefusedException or InvalidOperationException)
                {
                    row = $"error\t{e.Message.Split('\n')[0]}";
                }
                lines.Add($"{Path.GetFileName(game)}\t{row}");
            }
            if (Environment.GetEnvironmentVariable("EMUSEN_NEPHRITE_REPORT") is { Length: > 0 } report) File.WriteAllLines(report, lines);
            _output.WriteLine($"{games.Count} games compared");
            Assert.Equal(games.Count + 1, lines.Count);
        }

        private static string ReferenceCore(string name) =>
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "probe", "libretro", "cores", $"{name}_libretro.so");

        // A Sega CD run on a core that needs a BIOS is reported skipped, with no --sysdir and with an empty one; nothing is ever fetched for it.
        [Fact]
        public void The_reference_probe_skips_a_sega_cd_run_without_the_players_firmware()
        {
            string disc = Write("disc.iso", SyntheticMdRom.Disc());
            string empty = Path.Combine(_root, "empty-sysdir");
            Directory.CreateDirectory(empty);
            var none = new LibretroProbeEngine("genesis_plus_gx", ReferenceCore("genesis_plus_gx"), Path.Combine(_root, "runs"), system: "segacd");
            var emptyDir = new LibretroProbeEngine("genesis_plus_gx", ReferenceCore("genesis_plus_gx"), Path.Combine(_root, "runs"), system: "segacd", systemDir: empty);
            if (!none.Available)
            {
                _output.WriteLine("no probe or Genesis Plus GX core here: not run");
                return;
            }
            var a = Assert.Throws<FirmwareSkippedException>(() => none.Run(disc, new[] { 10 }));
            var b = Assert.Throws<FirmwareSkippedException>(() => emptyDir.Run(disc, new[] { 10 }));
            Assert.Contains("no --sysdir", a.Message);
            Assert.Contains("holds none", b.Message);
            Assert.Empty(Directory.EnumerateFileSystemEntries(empty));
            _output.WriteLine(a.Message);
        }

        // A pinned option reaches the core and changes the run; an unpinned one is the default the core declares, which the probe states.
        [Fact]
        public void The_reference_probe_pins_a_core_option()
        {
            string rom = Write("cart.md", SyntheticMdRom.Cartridge());
            var runs = Path.Combine(_root, "runs");
            var plain = new LibretroProbeEngine("clownmdemu", ReferenceCore("clownmdemu"), runs);
            var pinned = new LibretroProbeEngine("clownmdemu", ReferenceCore("clownmdemu"), runs, new Dictionary<string, string> { ["clownmdemu_overseas_region"] = "japan" });
            if (!plain.Available)
            {
                _output.WriteLine("no probe or ClownMDEmu core here: not run");
                return;
            }
            plain.Run(rom, new[] { 5 });
            pinned.Run(rom, new[] { 5 });
            string[] logs = Directory.EnumerateFiles(Path.Combine(runs, "clownmdemu"), "probe.log", SearchOption.AllDirectories).Select(File.ReadAllText).ToArray();
            Assert.Equal(2, logs.Length);
            Assert.Contains(logs, l => l.Contains("option clownmdemu_overseas_region=elsewhere (default)"));
            Assert.Contains(logs, l => l.Contains("option clownmdemu_overseas_region=japan (pinned)"));
        }

        // The reference engines answer through the same runner: Genesis Plus GX and PicoDrive on the corpus's first cartridge.
        [Fact]
        public void The_reference_engines_run_a_cartridge_through_the_probe()
        {
            string? rom = Corpus.Unique().Select(r => r.Path).FirstOrDefault(p => Path.GetExtension(p) is ".md" or ".gen");
            string cores = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "probe", "libretro", "cores");
            var engines = new[] { "genesis_plus_gx", "picodrive" }.Select(n => new LibretroProbeEngine(n, Path.Combine(cores, $"{n}_libretro.so"), Path.Combine(Corpus.Root ?? _root, "runs"))).ToArray();
            if (rom is null || !engines.All(e => e.Available))
            {
                _output.WriteLine("no corpus cartridge, probe or reference core here: not run");
                return;
            }
            var runs = engines.Select(e => e.Run(rom, new[] { 600 })).ToArray();
            foreach (var run in runs)
            {
                var shot = run.At(600)!;
                Assert.Equal(65536, shot.Spaces["ram"].Length);
                Assert.Equal(320, shot.Picture!.Width);
            }
            var diff = TestRomDifferential.Compare(runs[0], runs[1]).Single();
            _output.WriteLine($"{Path.GetFileName(rom)} at 600: ram differs in {diff.SpaceBytes["ram"]} bytes, {diff.PixelsDiffering} of {diff.PixelsCompared} pixels, {diff.PixelsOffMap} off the colour map");
        }
    }
}
