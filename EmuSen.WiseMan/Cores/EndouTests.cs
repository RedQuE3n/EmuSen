using System.Diagnostics;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Sms;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Fixtures.RomRunner;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Endou's stage 1 stub through the v1 adapter, discovery, the generic test-ROM runner and the conformance kit, on SyntheticSmsRom's images - see Endou_Native.md §2-§5.
    [Collection(TestCollections.ProcessGlobals)]
    public class EndouTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenEndou_" + Guid.NewGuid().ToString("N"));

        public EndouTests(ITestOutputHelper output)
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

        public static string LibraryPath => CoreAdapterTests.LibraryPath("endou");

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
            Assert.Equal(("endou", "Endou", 0UL), (library.Info.Id, library.Info.DisplayName, library.Capabilities));
            Assert.Equal(new[] { (SmsSystems.MasterSystemId, true), (SmsSystems.Sg1000Id, true) }, library.Info.Systems.Select(s => (s.Id, s.Development)));
            foreach (var system in library.Info.Systems)
            {
                var pack = SystemPacks.For(system.Id)!;
                Assert.Equal(pack.Entry.Extensions, system.Extensions);
                Assert.Empty(system.Firmware);
            }
            Assert.Same(SmsSystems.MasterSystem, SystemPacks.For("sms")!.Entry);
            Assert.Equal(new[] { ("sms.pad", 8), ("sg1000.pad", 7) }, library.Info.Systems.Select(s => (s.Controllers.Single().Id, s.Controllers.Single().Buttons.Count)));
            Assert.Equal(new[] { "region" }, library.Settings.Select(s => s.Key));
        }

        [Fact]
        public void A_machine_runs_as_the_console_and_market_its_image_names()
        {
            var library = CoreLibrary.Open(LibraryPath);
            foreach (var (image, settings, system, region, hz) in new (byte[], string, string, string, double)[]
            {
                (SyntheticSmsRom.Cartridge(0x4), "", "sms", "ntsc", 59.9227),
                (SyntheticSmsRom.Cartridge(0x3), "region=sms.eu", "sms", "pal", 49.7015),
                (SyntheticSmsRom.WithCopierHeader(SyntheticSmsRom.Cartridge(0x4, 0x20000)), "", "sms", "ntsc", 59.9227),
                (SyntheticSmsRom.Sg1000(), "", "sg1000", "ntsc", 59.9227),
            })
            {
                using var machine = new CoreMachine(library, image, settings, Array.Empty<(uint, byte[])>());
                var info = machine.Info;
                Assert.Equal((system, region, 256, 192), (info.System, info.Region, info.BaseWidth, info.BaseHeight));
                Assert.Equal(hz, info.FrameRateHz, 4);
                Assert.Equal(new[] { "Z80" }, info.Processors.Select(p => p.Name));
                Assert.Equal(48000, info.AudioRate);
            }
            var refused = Assert.Throws<CoreRefusedException>(() => new CoreMachine(library, new byte[(4 << 20) + 513], "", Array.Empty<(uint, byte[])>()));
            _output.WriteLine(refused.Message);
        }

        [Fact]
        public void The_header_and_its_checksum_are_sega_s()
        {
            byte[] rom = SyntheticSmsRom.Cartridge(0x4, 0x8000, product: 27026, version: 3);
            Assert.Equal("TMR SEGA", System.Text.Encoding.ASCII.GetString(rom, 0x7FF0, 8));
            Assert.Equal(new byte[] { 0x26, 0x70, 0x23, 0x4C }, rom[0x7FFC..0x8000]);
            ushort stated = (ushort)(rom[0x7FFA] | rom[0x7FFB] << 8);
            Assert.Equal(SyntheticSmsRom.Checksum(rom, 0x7FF0, 0xC), stated);
            rom[0x7FFA] ^= 0xFF;
            Assert.Equal(stated, SyntheticSmsRom.Checksum(rom, 0x7FF0, 0xC));
            Assert.NotEqual(stated, (ushort)(SyntheticSmsRom.Cartridge(0x4, rightChecksum: false) is var bad ? bad[0x7FFA] | bad[0x7FFB] << 8 : 0));
        }

        [Fact]
        public void The_stub_runs_each_console_through_the_generic_runner()
        {
            var engine = new CoreAbiTestRomEngine(LibraryPath);
            Assert.True(engine.Available, engine.Report);
            foreach (var (name, image, spaces) in new (string, byte[], string[])[]
            {
                ("export.sms", SyntheticSmsRom.Cartridge(), new[] { "cram", "ram", "rom", "vram" }),
                ("japan.sms", SyntheticSmsRom.Cartridge(0x3), new[] { "cram", "ram", "rom", "vram" }),
                ("cartridge.sg", SyntheticSmsRom.Sg1000(), new[] { "ram", "rom", "vram" }),
            })
            {
                var run = engine.Run(Write(name, image), new[] { 1, 60 }, new[] { new RomPress(10, PadButton.Start) });
                var last = run.At(60)!;
                Assert.Equal(spaces, last.Spaces.Keys.Order());
                Assert.Equal(image.Length, last.Spaces["rom"].Length);
                Assert.Equal((256, 192), (last.Picture!.Width, last.Picture.Height));
                Assert.All(last.Picture.Rgb, b => Assert.Equal(0, b));
                Assert.Equal(48000, run.AudioRate);
                Assert.True(TestRomDifferential.Steady(run, 1, 60));
                Assert.Equal(RomOutcome.Visual, new TestRomGrader(Array.Empty<RomProtocol>()).Grade(name, run).Outcome);
            }
        }

        // A developer asking for development cores opens the files; the system packs name the consoles.
        [Fact]
        public void A_cartridge_opens_on_the_discovered_engine_when_development_cores_are_asked_for()
        {
            Assert.Contains(CoreDiscovery.Found, c => c.Info.Id == "endou");
            foreach (var (name, image, system) in new[] { ("game.sms", SyntheticSmsRom.Cartridge(), "sms"), ("game.sg", SyntheticSmsRom.Sg1000(), "sg1000") })
            {
                string path = Write(name, image);
                Assert.True(CoreFactory.IsSupported(path));
                var bundle = CoreFactory.Load(path);
                var core = Assert.IsType<CoreEngine>(bundle.Core);
                Assert.Equal(("endou", system), (core.Info.Id, core.Machine.Info.System));
                core.RunFrame();
                Assert.Equal((256, 192), (core.ScreenWidth, core.ScreenHeight));
                Assert.Null(bundle.CheatAutoDetectCodec);
                (core as IDisposable)?.Dispose();
            }
        }

        // The stage 1 mark: the build writes the sidecar marked, and a player's discovery sees neither system - see EmuSen_CoreAPI.md §27.
        [Fact]
        public void A_player_is_not_offered_the_core_in_development()
        {
            var sidecar = CoreSidecar.Read(CoreSidecar.PathFor(LibraryPath))!;
            Assert.True(sidecar.Development);
            CoreDiscovery.UseDevelopment(false);
            Assert.DoesNotContain(CoreDiscovery.Found, c => c.Info.Id == "endou");
            foreach (string ext in new[] { ".sms", ".sg" })
            {
                Assert.False(CoreFactory.IsSupported("game" + ext), ext);
                Assert.False(CoreCatalog.IsRomExtension(ext), ext);
                Assert.DoesNotContain(ext, EmuSen.Mistress.Library.RomLibrary.Extensions);
            }
            Assert.DoesNotContain("Master System", CoreCatalog.DiscoveredConsoles);
            Assert.Throws<NotSupportedException>(() => CoreFactory.Create(Write("game.sms", SyntheticSmsRom.Cartridge())));
        }

        // The gate of Endou_Plan.md §7's stage 1: every case the kit runs passes on each synthetic image, C16 being MEDIA's, which Endou does not claim.
        [Fact]
        public void The_kit_passes_on_synthetic_cartridges()
        {
            string runner = Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-conform.exe" : "emusen-core-conform");
            Assert.True(File.Exists(runner), $"no kit runner at {runner}");
            var expected = Enumerable.Range(1, 23).Where(n => n != 16).Select(n => $"C{n}").ToArray();
            foreach (var (name, image) in new[]
            {
                ("export.sms", SyntheticSmsRom.Cartridge()),
                ("japan.sms", SyntheticSmsRom.Cartridge(0x3, 0x40000)),
                ("copier.sms", SyntheticSmsRom.WithCopierHeader(SyntheticSmsRom.Cartridge())),
                ("cartridge.sg", SyntheticSmsRom.Sg1000()),
            })
            {
                var start = new ProcessStartInfo(runner) { RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (string a in new[] { "--core", LibraryPath, "--image", Write(name, image), "--frames", "120" }) start.ArgumentList.Add(a);
                using var process = Process.Start(start)!;
                var stderr = process.StandardError.ReadToEndAsync();
                string output = process.StandardOutput.ReadToEnd();
                process.WaitForExit();
                _output.WriteLine($"{name}:\n{output}{stderr.Result}");
                Assert.True(process.ExitCode == 0, output + stderr.Result);
                var passed = output.Split('\n').Where(l => l.StartsWith("pass C")).Select(l => l.Split(' ', StringSplitOptions.RemoveEmptyEntries)[1]).ToArray();
                Assert.Equal(expected, passed);
            }
        }
    }
}
