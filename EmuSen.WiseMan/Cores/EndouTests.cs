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
    // Endou through the v1 adapter, discovery, the generic test-ROM runner and the conformance kit, on SyntheticSmsRom's images - see Endou_Native.md §2-§5, §9.
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
            CoreDiscovery.UseDevelopment(false);
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
            Assert.Equal(("endou", "Endou", CoreInterface.CapRomPatches | CoreInterface.CapDebug | CoreInterface.CapDebugStack | CoreInterface.CapDebugRegisters | CoreInterface.CapDebugDisassemble), (library.Info.Id, library.Info.DisplayName, library.Capabilities));
            Assert.Equal(new[] { (SmsSystems.MasterSystemId, false), (SmsSystems.Sg1000Id, false), ("gg", true) }, library.Info.Systems.Select(s => (s.Id, s.Development)));
            foreach (var system in library.Info.Systems.Where(s => !s.Development))
            {
                var pack = SystemPacks.For(system.Id)!;
                Assert.Equal(pack.Entry.Extensions, system.Extensions);
            }
            Assert.Equal(new[] { (1u, "bios_sms1.sms", 8192L, false, "accuracy"), (2u, "bios_sms2.sms", 8192L, false, "accuracy"), (3u, "bios_jsms.sms", 8192L, false, "accuracy") }, library.Info.Systems[0].Firmware.Select(f => (f.Which, f.Name, f.Size, f.Required, f.Replacement!.Effect)));
            Assert.Equal(new[] { 2, 0, 1 }, library.Info.Systems[0].Firmware.Select(f => f.Md5.Count));
            Assert.Empty(library.Info.Systems[1].Firmware);
            Assert.Equal(new[] { true, false, false }, library.Info.Systems.Select(s => s.StartsEmpty));
            Assert.Same(SmsSystems.MasterSystem, SystemPacks.For("sms")!.Entry);
            Assert.Equal(new[] { ("sms.pad", 8), ("sg1000.pad", 7), ("gg.pad", 7) }, library.Info.Systems.Select(s => (s.Controllers.Single().Id, s.Controllers.Single().Buttons.Count)));
            Assert.Equal(new[] { "model", "region", "bios_sms1", "bios_sms2", "bios_jsms" }, library.Settings.Select(s => s.Key));
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

        // The kept bytes a banking program leaves at $C000 a second after the open boot program hands it over, and the spaces its board shows - see Endou_Native.md §9.3, §13.
        private static (byte[] Kept, string[] Spaces) Run(CoreLibrary library, byte[] image, int count)
        {
            using var machine = new CoreMachine(library, image, "", Array.Empty<(uint, byte[])>());
            for (int frame = 0; frame < 60; frame++) machine.Advance();
            var kept = new byte[count];
            Assert.True(machine.ReadSpace(1, 0, kept));
            return (kept, machine.Info.Spaces.Select(s => s.Name).ToArray());
        }

        [Fact]
        public void Each_mapper_banks_its_slots_as_its_board_does()
        {
            var library = CoreLibrary.Open(LibraryPath);
            var sega = Run(library, SyntheticSmsRom.SegaMapper(), 10);
            Assert.Equal(new byte[] { 5, 6, 7, 0xF3, 1, 33, 0x77, 0x00, 1, 0 }, sega.Kept);
            Assert.Contains("CARTRAM", sega.Spaces);
            var small = Run(library, SyntheticSmsRom.SegaMapper(8), 6);
            Assert.Equal(new byte[] { 5, 6, 7, 0xF3, 1, 9 }, small.Kept);
            var codemasters = Run(library, SyntheticSmsRom.CodemastersMapper(), 7);
            Assert.Equal(new byte[] { 5, 3, 5, 0x5A, 0x00, 4, 0 }, codemasters.Kept);
            var plain = Run(library, SyntheticSmsRom.Plain(), 4);
            Assert.Equal(new byte[] { 2, 1, 0, 0 }, plain.Kept);
            Assert.DoesNotContain("CARTRAM", plain.Spaces);
            var mirrored = Run(library, SyntheticSmsRom.Plain(2), 3);
            Assert.Equal(new byte[] { 0, 1, 0 }, mirrored.Kept);
        }

        // The stub's pad reader: ports $DC and $DD copied to $C000 and $C001, active low, the Reset button only on the first Master System - see Endou_Native.md §9.5.
        [Fact]
        public void The_pads_and_the_reset_button_read_through_the_joystick_ports()
        {
            var library = CoreLibrary.Open(LibraryPath);
            foreach (var (settings, dd) in new[] { ("model=sms1", 0b1110_1101), ("model=sms2", 0b1111_1101), ("", 0b1111_1101) })
            {
                using var machine = new CoreMachine(library, SyntheticSmsRom.Cartridge(), settings, Array.Empty<(uint, byte[])>());
                machine.SetButtons(0, 0b1001_0001, 0xFF);
                machine.SetButtons(1, 0b0000_1010, 0xFF);
                for (int frame = 0; frame < 60; frame++) machine.Advance();
                var ports = new byte[2];
                Assert.True(machine.ReadSpace(1, 0, ports));
                Assert.Equal(new byte[] { 0b0110_1110, (byte)dd }, ports);
            }
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
            rom[0x7FFE] ^= 0x01;
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
                ("export.sms", SyntheticSmsRom.Cartridge(), new[] { "bios", "cram", "ram", "rom", "vram" }),
                ("japan.sms", SyntheticSmsRom.Cartridge(0x3), new[] { "bios", "cram", "ram", "rom", "vram" }),
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

        // The VDP through the v1 adapter: CRAM entries at the D-1 levels, the backdrop in the masked column - see Endou_Native.md §10.8.
        [Fact]
        public void The_vdp_draws_a_synthetic_program_s_picture()
        {
            var engine = new CoreAbiTestRomEngine(LibraryPath);
            Assert.True(engine.Available, engine.Report);
            var run = engine.Run(Write("picture.sms", SyntheticSmsRom.Picture()), new[] { 60 }, Array.Empty<RomPress>());
            var picture = run.At(60)!.Picture!;
            Assert.Equal((256, 192), (picture.Width, picture.Height));
            for (int y = 0; y < 192; y += 37)
            {
                Assert.Equal(((byte)0, (byte)255, (byte)0), picture.At(7, y));
                Assert.Equal(((byte)0, (byte)0, (byte)255), picture.At(8, y));
                Assert.Equal(((byte)0, (byte)0, (byte)255), picture.At(255, y));
            }
        }

        // J1: the Game Gear in development - a player is not offered .gg, a developer is; its own mode draws the LCD's 160 by 144 window in 12-bit colours, CRAM written through the latch - see Jadeite_Native.md §2, §4.
        [Fact]
        public void The_game_gear_is_a_system_in_development_that_draws_its_lcd_window()
        {
            Assert.False(CoreFactory.IsSupported("game.gg"));
            Assert.DoesNotContain("Game Gear", CoreCatalog.DiscoveredConsoles);
            CoreDiscovery.UseDevelopment(true);
            Assert.True(CoreFactory.IsSupported("game.gg"));
            var library = CoreLibrary.Open(LibraryPath);
            using var machine = new CoreMachine(library, SyntheticSmsRom.GgPicture(), "", Array.Empty<(uint, byte[])>(), "gg");
            Assert.Equal(("gg", "ntsc", 160, 144, 160, 146), (machine.Info.System, machine.Info.Region, machine.Info.BaseWidth, machine.Info.BaseHeight, machine.Info.MaxWidth, machine.Info.MaxHeight));
            for (int frame = 0; frame < 5; frame++) machine.Advance();
            var frameInfo = machine.FrameInfo;
            Assert.Equal((160, 144), (frameInfo.Width, frameInfo.Height));
            var pixels = new byte[160 * 144 * 4];
            Assert.Equal(pixels.Length, machine.CopyFrame(pixels));
            for (int i = 0; i < 160 * 144; i += 97)
                Assert.Equal(((byte)0, (byte)255, (byte)0), (pixels[4 * i], pixels[4 * i + 1], pixels[4 * i + 2]));
        }

        // The SG-1000's TMS9918A through the v1 adapter: Graphics I in TI's colours 6 and 12 - see Endou_Native.md §11.
        [Fact]
        public void The_tms9918a_draws_an_sg_1000_program_s_picture()
        {
            var engine = new CoreAbiTestRomEngine(LibraryPath);
            Assert.True(engine.Available, engine.Report);
            var run = engine.Run(Write("tms.sg", SyntheticSmsRom.TmsPicture()), new[] { 5 }, Array.Empty<RomPress>());
            var picture = run.At(5)!.Picture!;
            Assert.Equal((256, 192), (picture.Width, picture.Height));
            for (int y = 0; y < 192; y += 37)
                for (int x = 0; x < 256; x += 13)
                    Assert.Equal(x % 8 < 4 ? ((byte)0xD4, (byte)0x52, (byte)0x4D) : ((byte)0x21, (byte)0xB0, (byte)0x3B), picture.At(x, y));
        }

        // The PSG through the v1 adapter at 48 kHz: a tone's fundamental at 440.4 Hz, 4/pi of its 0-4,096 square's half swing, on both consoles, measured after the open boot program's half second - see Endou_Native.md §12, §13.
        [Fact]
        public void The_psg_sounds_a_synthetic_program_s_tone()
        {
            var library = CoreLibrary.Open(LibraryPath);
            foreach (bool sg1000 in new[] { false, true })
            {
                using var machine = new CoreMachine(library, SyntheticSmsRom.Tone(sg1000), "", Array.Empty<(uint, byte[])>());
                var left = new List<double>();
                for (int frame = 0; frame < 90; frame++)
                {
                    machine.Advance();
                    var (samples, rate) = machine.DrainAudio(4096);
                    Assert.Equal(48000, rate);
                    for (int i = 0; i < samples.Length; i += 2)
                    {
                        Assert.Equal(samples[i], samples[i + 1]);
                        left.Add(samples[i]);
                    }
                }
                Assert.InRange(left.Count, 72091, 72096);
                var second = left.Skip(33600).Take(28800).ToArray();
                double mean = second.Average(), re = 0, im = 0;
                for (int n = 0; n < second.Length; n++)
                {
                    double w = 0.5 - 0.5 * Math.Cos(2 * Math.PI * n / second.Length), a = 2 * Math.PI * 440.4 * n / 48000;
                    re += (second[n] - mean) * w * Math.Cos(a);
                    im += (second[n] - mean) * w * Math.Sin(a);
                }
                double amplitude = 4 * Math.Sqrt(re * re + im * im) / second.Length;
                Assert.InRange(amplitude, 2607.6 * 0.99, 2607.6 * 1.01);
                Assert.InRange(second.Max() - second.Min(), 4096, 4096 * 1.2);
            }
        }

        // Since stage 7 a player's build opens both systems on Endou, with the packs' two cheat formats - see Endou_Native.md §14.
        [Fact]
        public void A_cartridge_opens_on_endou_for_a_player_with_the_pack_s_codecs()
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
                Assert.Equal(("Pro Action Replay", "RAM", "Game Genie"), (bundle.CheatAutoDetectCodec!.Name, bundle.CheatAutoDetectCodec.SpaceName, bundle.CheatExplicitCodec!.Name));
                (core as IDisposable)?.Dispose();
            }
        }

        // Core ABI 1.15: the host names the system it routed the file to, so an image without Sega's header runs as its extension's system where the stage-1 rule guessed the other - see Endou_Native.md §2.3.
        [Theory]
        [InlineData("Music.sg", 0x10000, "sg1000", "sms")]
        [InlineData("Bios.sms", 0x2000, "sms", "sg1000")]
        [InlineData("Small.sg", 0x4000, "sg1000", "sg1000")]
        public void A_headerless_image_runs_as_the_system_it_was_routed_to(string name, int size, string routed, string guessed)
        {
            string path = Write(name, SyntheticSmsRom.Sg1000(size));
            var bundle = CoreFactory.Load(path);
            using var core = Assert.IsType<CoreEngine>(bundle.Core);
            Assert.Equal(("endou", routed), (core.Info.Id, core.Machine.Info.System));
            core.RunFrame();
            using var unnamed = new CoreMachine(core.Library, File.ReadAllBytes(path), "", Array.Empty<(uint, byte[])>());
            Assert.Equal(guessed, unnamed.Info.System);
        }

        // Stage 7: the build writes the sidecar unmarked, and a player's discovery puts the Master System and the SG-1000 on two shelves of their own (decided 2026-10-19) - see Endou_Native.md §14.
        [Fact]
        public void A_player_is_offered_the_master_system_and_the_sg_1000_on_two_shelves()
        {
            var sidecar = CoreSidecar.Read(CoreSidecar.PathFor(LibraryPath))!;
            Assert.False(sidecar.Development);
            foreach (string ext in new[] { ".sms", ".sg" })
            {
                Assert.True(CoreFactory.IsSupported("game" + ext), ext);
                Assert.True(CoreCatalog.IsRomExtension(ext), ext);
                Assert.Contains(ext, EmuSen.Mistress.Library.RomLibrary.Extensions);
            }
            Assert.Equal(new[] { "SG-1000", "Master System" }, CoreCatalog.DiscoveredConsoles.Where(c => c is "SG-1000" or "Master System"));
            var shelves = CoreCatalog.ShelvesInReleaseOrder.Where(s => s.Core.Console is "SG-1000" or "Master System").Select(s => (s.Name, s.Label, s.EsdeSystem, s.System));
            Assert.Equal(new[] { ("SG-1000 (Endou)", "SG-1000", "sg-1000", "sg1000"), ("Master System (Endou)", "Master System", "mastersystem", "sms") }, shelves);
            string sms = Write("Game.sms", SyntheticSmsRom.Cartridge()), sg = Write("Game.sg", SyntheticSmsRom.Sg1000());
            Assert.Equal(("Master System (Endou)", "SG-1000 (Endou)"), (CoreCatalog.ShelfFor(sms), CoreCatalog.ShelfFor(sg)));
            Assert.Equal(("Master System (Endou)", "SG-1000 (Endou)"), (EmuSen.Mistress.Library.RuntimeCatalog.ShelfFor(sms), EmuSen.Mistress.Library.RuntimeCatalog.ShelfFor(sg)));
            Assert.Equal("Master System (Endou)", CoreCatalog.Registry["endou"].DisplayName);
            Assert.Equal(new[] { "model", "region", "bios_sms1", "bios_sms2", "bios_jsms" }, CoreCatalog.SettingsFor("Master System").Select(s => s.Key));
            Assert.Equal(new[] { "model", "region", "bios_sms1", "bios_sms2", "bios_jsms" }, CoreCatalog.SettingsFor("SG-1000").Select(s => s.Key));
        }

        // Decided 2026-10-19: the open boot program by default, machine info naming the replacement; an export cartridge handed over with $AB at $C000; a Japanese one on the Japanese Master System, whose BIOS it replaces too - see Endou_Native.md §13, §16.
        [Fact]
        public void A_master_system_runs_the_open_boot_program_first_and_names_it()
        {
            var library = CoreLibrary.Open(LibraryPath);
            var image = SyntheticSmsRom.Banked(2, SyntheticSmsRom.Program(new[] { SyntheticSmsRom.Step.Keep(0xC000), SyntheticSmsRom.Step.Keep(0xC100) }));
            using var machine = new CoreMachine(library, image, "", Array.Empty<(uint, byte[])>());
            Assert.Equal(new[] { (2u, "replacement") }, machine.Info.Firmware.Select(f => (f.Which, f.Source)));
            Assert.Contains("BIOS", machine.Info.Spaces.Select(s => s.Name));
            for (int frame = 0; frame < 60; frame++) machine.Advance();
            var kept = new byte[2];
            Assert.True(machine.ReadSpace(1, 0, kept));
            Assert.Equal(new byte[] { 0xAB, 0x00 }, kept);
            using var japan = new CoreMachine(library, SyntheticSmsRom.Cartridge(0x3), "", Array.Empty<(uint, byte[])>());
            Assert.Equal(new[] { (3u, "replacement") }, japan.Info.Firmware.Select(f => (f.Which, f.Source)));
            using var mark3 = new CoreMachine(library, SyntheticSmsRom.Cartridge(0x3), "model=mark3", Array.Empty<(uint, byte[])>());
            Assert.Empty(mark3.Info.Firmware);
        }

        // The player's own BIOS for a console, opted into by its setting: a stand-in boot ROM written for the test runs and hands over; without the setting the open program runs - see Endou_OpenBoot.md §5.
        [Fact]
        public void The_player_s_own_bios_runs_when_its_setting_chooses_it()
        {
            var library = CoreLibrary.Open(LibraryPath);
            var image = SyntheticSmsRom.Banked(2, SyntheticSmsRom.Program(new[] { SyntheticSmsRom.Step.Keep(0xC000), SyntheticSmsRom.Step.Keep(0xC100) }));
            var files = new[] { (2u, SyntheticSmsRom.StandInBios()) };
            foreach (var (settings, source, mark) in new[] { ("bios_sms2=player", "file", (byte)0x5C), ("", "replacement", (byte)0x00) })
            {
                using var machine = new CoreMachine(library, image, settings, files);
                Assert.Equal(new[] { (2u, source) }, machine.Info.Firmware.Select(f => (f.Which, f.Source)));
                for (int frame = 0; frame < 60; frame++) machine.Advance();
                var kept = new byte[2];
                Assert.True(machine.ReadSpace(1, 0, kept));
                Assert.Equal(new byte[] { 0xAB, mark }, kept);
            }
        }

        // The export check refuses a Japanese cartridge on an export console in words, on its own screen - see Endou_OpenBoot.md §3.3.
        [Fact]
        public void An_export_console_refuses_a_japanese_cartridge_in_words()
        {
            var library = CoreLibrary.Open(LibraryPath);
            using var machine = new CoreMachine(library, SyntheticSmsRom.Cartridge(0x3), "model=sms2", Array.Empty<(uint, byte[])>());
            for (int frame = 0; frame < 60; frame++) machine.Advance();
            var names = new byte[64];
            Assert.True(machine.ReadSpace(2, 0x3800 + 15 * 64, names));
            Assert.Equal("IT IS FOR A JAPANESE CONSOLE.", new string(names.Where((_, i) => i % 2 == 0).Select(b => (char)(b + 32)).ToArray()).Trim());
        }

        // The gate of Endou_Plan.md §7's stage 1: every case the kit runs passes on each synthetic image, C16 being MEDIA's, which Endou does not claim.
        [Fact]
        public void The_kit_passes_on_synthetic_cartridges()
        {
            string runner = Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-conform.exe" : "emusen-core-conform");
            Assert.True(File.Exists(runner), $"no kit runner at {runner}");
            var expected = Enumerable.Range(1, 24).Where(n => n != 16).Select(n => $"C{n}").ToArray();
            foreach (var (name, image) in new[]
            {
                ("export.sms", SyntheticSmsRom.Cartridge()),
                ("japan.sms", SyntheticSmsRom.Cartridge(0x3, 0x40000)),
                ("copier.sms", SyntheticSmsRom.WithCopierHeader(SyntheticSmsRom.Cartridge())),
                ("cartridge.sg", SyntheticSmsRom.Sg1000()),
                ("tms.sg", SyntheticSmsRom.TmsPicture()),
                ("tone.sms", SyntheticSmsRom.Tone()),
                ("tone.sg", SyntheticSmsRom.Tone(sg1000: true)),
                ("picture.gg", SyntheticSmsRom.GgPicture()),
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
