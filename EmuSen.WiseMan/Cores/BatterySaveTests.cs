using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Every battery save in its console's folder in Saves, and each save an older build kept elsewhere copied there once - see EmuSen_Settings_Reference.md §4.85.6 and §4.85.11.
    [Collection(TestCollections.ProcessGlobals)]
    public class BatterySaveTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenBatterySave_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public BatterySaveTests()
        {
            _library = Path.Combine(_root, "Library");
            Directory.CreateDirectory(_library);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = false;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        public static TheoryData<string> Engines => new() { "Moon", "MoonRT", "Mercury", "MercuryRT" };

        private static bool Nes(string engine) => engine.StartsWith("Moon", StringComparison.Ordinal);

        private static ICore Make(string engine) => engine switch
        {
            "Moon" => new MoonCore(),
            "MoonRT" => new MoonRtCore(),
            "Mercury" => new MercuryCore(),
            _ => new MercuryRtCore(),
        };

        private static string RamSpace(string engine) => Nes(engine) ? MoonCore.SpacePrgRam : MercuryCore.SpaceCartRam;

        // A cartridge with battery-backed RAM whose program never touches it, so what a load put there stays.
        private string Rom(string engine, string stem, bool withRam = true)
        {
            byte[] image = Nes(engine)
                ? SyntheticNesRom.Build(battery: true, patches: (0, new byte[] { 0x4C, 0x00, 0x80 }))
                : SyntheticGbRom.Build(cartridgeType: withRam ? (byte)0x03 : (byte)0x0F, ramSizeCode: withRam ? (byte)0x02 : (byte)0x00, patches: (0, new byte[] { 0x18, 0xFE }));
            string path = Path.Combine(_library, stem + (Nes(engine) ? ".nes" : ".gb"));
            File.WriteAllBytes(path, image);
            return path;
        }

        private static byte[] Pattern(int seed)
        {
            var bytes = new byte[8192];
            new Random(seed).NextBytes(bytes);
            return bytes;
        }

        private static string SaveOf(string engine, string rom) => SaveLibrary.SramPathFor(rom, Nes(engine) ? BatterySave.Nes : BatterySave.GameBoyFolder(rom));

        private static string Beside(string rom) => Path.ChangeExtension(rom, SaveLibrary.SramExtension);

        private static (byte[] Bytes, DateTime Written) Snapshot(string path) => (File.ReadAllBytes(path), File.GetLastWriteTimeUtc(path));

        private static void AssertUnchanged((byte[] Bytes, DateTime Written) before, string path)
        {
            var after = Snapshot(path);
            Assert.Equal(before.Bytes, after.Bytes);
            Assert.Equal(before.Written, after.Written);
        }

        private static void Play(ICore core, int frames)
        {
            for (int i = 0; i < frames; i++) core.RunFrame();
            core.SaveSram();
        }

        private static void Done(ICore core) => (core as IDisposable)?.Dispose();

        [Theory]
        [MemberData(nameof(Engines))]
        public void A_save_beside_the_rom_is_copied_into_saves_and_the_original_is_left_alone(string engine)
        {
            string rom = Rom(engine, $"Beside {engine}");
            File.WriteAllBytes(Beside(rom), Pattern(1));
            File.SetLastWriteTimeUtc(Beside(rom), new DateTime(2020, 1, 2, 3, 4, 5, DateTimeKind.Utc));
            var before = Snapshot(Beside(rom));

            ICore core = Make(engine);
            core.LoadRom(rom);
            Assert.True(File.Exists(SaveOf(engine, rom)), "the save was not copied into Saves");
            Assert.Equal(Pattern(1), File.ReadAllBytes(SaveOf(engine, rom)));
            Assert.Equal(Pattern(1)[0x123], ReadRam(core, engine, 0x123));

            WriteRam(core, engine, 0x123, (byte)~Pattern(1)[0x123]);
            Play(core, BatterySave.FlushEveryNFrames + 1);
            Done(core);

            AssertUnchanged(before, Beside(rom));
            Assert.Equal((byte)~Pattern(1)[0x123], File.ReadAllBytes(SaveOf(engine, rom))[0x123]);
        }

        [Theory]
        [MemberData(nameof(Engines))]
        public void A_save_already_in_saves_wins_over_one_beside_the_rom(string engine)
        {
            string rom = Rom(engine, $"Both {engine}");
            File.WriteAllBytes(Beside(rom), Pattern(2));
            Directory.CreateDirectory(Path.GetDirectoryName(SaveOf(engine, rom))!);
            File.WriteAllBytes(SaveOf(engine, rom), Pattern(3));
            var before = Snapshot(Beside(rom));

            ICore core = Make(engine);
            core.LoadRom(rom);
            Assert.Equal(Pattern(3)[0x77], ReadRam(core, engine, 0x77));
            Assert.Equal(Pattern(3), File.ReadAllBytes(SaveOf(engine, rom)));
            Play(core, 10);
            Done(core);

            AssertUnchanged(before, Beside(rom));
            Assert.Equal(Pattern(3), File.ReadAllBytes(SaveOf(engine, rom)));
        }

        [Theory]
        [MemberData(nameof(Engines))]
        public void A_game_with_no_save_anywhere_gets_one_only_in_saves(string engine)
        {
            string rom = Rom(engine, $"Fresh {engine}");

            ICore core = Make(engine);
            core.LoadRom(rom);
            Assert.False(File.Exists(SaveOf(engine, rom)));
            Play(core, 10);
            Done(core);

            Assert.True(File.Exists(SaveOf(engine, rom)));
            Assert.Equal(new[] { Path.GetFileName(rom) }, Directory.GetFiles(_library).Select(Path.GetFileName).ToArray());
        }

        [Theory]
        [MemberData(nameof(Engines))]
        public void With_nobattery_nothing_is_read_copied_or_written_anywhere(string engine)
        {
            string rom = Rom(engine, $"Off {engine}");
            File.WriteAllBytes(Beside(rom), Pattern(4));
            var before = Snapshot(Beside(rom));

            CoreOptions.BatteryRamDisabled = true;
            ICore core = Make(engine);
            core.LoadRom(rom);
            Assert.NotEqual(Pattern(4)[0x55], ReadRam(core, engine, 0x55));
            Play(core, BatterySave.FlushEveryNFrames + 1);
            Done(core);

            Assert.False(Directory.Exists(DataStore.Saves) && Directory.EnumerateFiles(DataStore.Saves, "*", SearchOption.AllDirectories).Any(), "a save was written into Saves");
            AssertUnchanged(before, Beside(rom));
            Assert.Equal(new[] { Path.GetFileName(rom), Path.GetFileName(Beside(rom)) }.OrderBy(n => n), Directory.GetFiles(_library).Select(Path.GetFileName).OrderBy(n => n));
        }

        // A battery with no RAM behind it (MBC3 with a clock and nothing else) is neither read, copied nor written, on both Game Boy engines.
        [Theory]
        [InlineData("Mercury")]
        [InlineData("MercuryRT")]
        public void A_cartridge_without_ram_has_no_save_at_all(string engine)
        {
            string rom = Rom(engine, $"NoRam {engine}", withRam: false);
            File.WriteAllBytes(Beside(rom), Pattern(5));

            ICore core = Make(engine);
            core.LoadRom(rom);
            Play(core, BatterySave.FlushEveryNFrames + 1);
            Done(core);

            Assert.False(File.Exists(SaveOf(engine, rom)));
            Assert.Equal(Pattern(5), File.ReadAllBytes(Beside(rom)));
        }

        // The SNES and the N64 kept their saves in the flat Saves folder, so nothing beside their ROMs is taken up.
        [Fact]
        public void Only_the_nes_and_game_boy_copy_from_beside_the_rom()
        {
            string rom = Path.Combine(_library, "Snes.smc");
            File.WriteAllBytes(rom, SyntheticRom.Build((0x7FD8, new byte[] { 0x03 })));
            File.WriteAllBytes(Beside(rom), Pattern(6));

            var core = new VenusCore(headless: true);
            core.LoadRom(rom);
            Assert.False(File.Exists(SaveLibrary.SramPathFor(rom, BatterySave.Snes)));
            Assert.Equal(Pattern(6), File.ReadAllBytes(Beside(rom)));
        }

        private const uint SramAddress = 0x700000;

        private static byte[] SnesImage() => SyntheticRom.Build((0x7FD8, new byte[] { 0x03 }));

        private static string Flat(string rom) => SaveLibrary.FlatSramPathFor(rom);

        private static void PutFlat(string rom, byte[] bytes)
        {
            Directory.CreateDirectory(DataStore.Saves);
            File.WriteAllBytes(Flat(rom), bytes);
            File.SetLastWriteTimeUtc(Flat(rom), new DateTime(2021, 6, 7, 8, 9, 10, DateTimeKind.Utc));
        }

        // Every console in a folder of its own, named as the library names them; a .gbc image's save under GBC, a .gb image's under GB.
        [Fact]
        public void Each_consoles_save_lands_in_its_own_folder()
        {
            foreach (var (engine, stem) in new[] { ("Moon", "Same"), ("Mercury", "Same") })
            {
                ICore core = Make(engine);
                core.LoadRom(Rom(engine, stem));
                Play(core, 1);
                Done(core);
            }
            string gbc = Path.Combine(_library, "Same.gbc");
            File.Copy(Rom("Mercury", "Colour"), gbc);
            var colour = new MercuryCore();
            colour.LoadRom(gbc);
            colour.SaveSram();

            string snes = Path.Combine(_library, "Same.smc");
            File.WriteAllBytes(snes, SnesImage());
            new EmuSen.Cores.Nintendo.Venus.Memory.Cartridge(snes, batteryRamDisabled: false).SaveSram();

            string n64 = Path.Combine(_library, "Same.z64");
            File.WriteAllBytes(n64, SyntheticN64System.Build());
            var mars = new EmuSen.Cores.Nintendo.Mars.MarsCore(batteryRamDisabled: false);
            mars.LoadRom(n64);
            mars.Bus!.Si.Controllers[0].Pak!.Dirty = true;
            mars.SaveSram();

            foreach (string folder in new[] { "NES", "GB", "GBC", "SNES" })
                Assert.True(File.Exists(Path.Combine(DataStore.Saves, folder, "Same.srm")), $"no save in Saves/{folder}");
            Assert.True(File.Exists(Path.Combine(DataStore.Saves, "N64", "Same" + EmuSen.Cores.Nintendo.Mars.MarsCore.PakExtension)), "no pak in Saves/N64");
            Assert.False(File.Exists(Path.Combine(DataStore.Saves, "Same.srm")), "a save was written to the flat folder");
        }

        // The console a .gb image's save belongs to is the file's, so choosing the Color for it does not move the save.
        [Fact]
        public void A_gb_game_played_as_a_colour_console_keeps_its_save_in_gb()
        {
            string rom = Rom("Mercury", "Played as Colour");
            var core = new MercuryCore { Model = EmuSen.Cores.Nintendo.Mercury.GbModel.GameBoyColor };
            core.LoadRom(rom);
            core.SaveSram();
            Assert.True(File.Exists(SaveLibrary.SramPathFor(rom, BatterySave.GameBoy)));

            // A .gb image whose header asks for the Color, as seventeen in the library do: filed with the .gb images, as the library files it.
            string dual = Path.Combine(_library, "Dual.gb");
            File.WriteAllBytes(dual, SyntheticGbRom.Build(cartridgeType: 0x03, ramSizeCode: 0x02, cgbFlag: 0x80, patches: (0, new byte[] { 0x18, 0xFE })));
            foreach (ICore core2 in new ICore[] { new MercuryCore(), new MercuryRtCore() })
            {
                core2.LoadRom(dual);
                core2.SaveSram();
                Done(core2);
            }
            Assert.True(File.Exists(SaveLibrary.SramPathFor(dual, BatterySave.GameBoy)));
            Assert.False(Directory.Exists(Path.Combine(DataStore.Saves, BatterySave.GameBoyColor)));
        }

        // An SNES save in the flat folder, as every build before 2026-09-28 wrote it: copied into Saves/SNES and loaded, the original untouched.
        [Fact]
        public void A_flat_folder_snes_save_is_copied_and_still_found()
        {
            string rom = Path.Combine(_library, "Flat.smc");
            File.WriteAllBytes(rom, SnesImage());
            PutFlat(rom, Pattern(7));
            var before = Snapshot(Flat(rom));

            var cart = new EmuSen.Cores.Nintendo.Venus.Memory.Cartridge(rom, batteryRamDisabled: false);
            Assert.Equal(Pattern(7)[0x21], cart.Read8(SramAddress + 0x21));
            Assert.Equal(Pattern(7), File.ReadAllBytes(SaveLibrary.SramPathFor(rom, BatterySave.Snes)));
            cart.Write8(SramAddress + 0x21, (byte)~Pattern(7)[0x21]);
            cart.SaveSram();

            AssertUnchanged(before, Flat(rom));
            Assert.Equal((byte)~Pattern(7)[0x21], File.ReadAllBytes(SaveLibrary.SramPathFor(rom, BatterySave.Snes))[0x21]);
            Assert.Equal((byte)~Pattern(7)[0x21], new EmuSen.Cores.Nintendo.Venus.Memory.Cartridge(rom, batteryRamDisabled: false).Read8(SramAddress + 0x21));
        }

        // The N64's chip save and its Controller Pak both come across from the flat folder.
        [Fact]
        public void A_flat_folder_n64_save_and_pak_are_copied()
        {
            string rom = Path.Combine(_library, "Flat.z64");
            File.WriteAllBytes(rom, SyntheticN64System.Build());
            var eeprom = new byte[512];
            new Random(8).NextBytes(eeprom);
            PutFlat(rom, eeprom);
            string flatPak = Path.ChangeExtension(Flat(rom), EmuSen.Cores.Nintendo.Mars.MarsCore.PakExtension);
            var pak = new byte[32768];
            new Random(9).NextBytes(pak);
            File.WriteAllBytes(flatPak, pak);
            var before = (Snapshot(Flat(rom)), Snapshot(flatPak));

            var mars = new EmuSen.Cores.Nintendo.Mars.MarsCore(batteryRamDisabled: false);
            mars.LoadRom(rom);
            Assert.Equal(eeprom, mars.Bus!.Save.Contents![..eeprom.Length]);
            string save = SaveLibrary.SramPathFor(rom, BatterySave.N64);
            Assert.Equal(eeprom, File.ReadAllBytes(save));
            Assert.Equal(pak, File.ReadAllBytes(Path.ChangeExtension(save, EmuSen.Cores.Nintendo.Mars.MarsCore.PakExtension)));
            AssertUnchanged(before.Item1, Flat(rom));
            AssertUnchanged(before.Item2, flatPak);
        }

        // Gemfire, and four other pairs in the library: one stem on the NES and the SNES; the flat save is the SNES game's, since only it used that folder.
        [Theory]
        [MemberData(nameof(NesEngines))]
        public void An_nes_game_never_adopts_a_flat_folder_save_of_the_same_stem(string engine)
        {
            string nes = Rom(engine, "Gemfire (U)");
            string snes = Path.Combine(_library, "Gemfire (U).smc");
            File.WriteAllBytes(snes, SnesImage());
            PutFlat(snes, Pattern(10));
            var before = Snapshot(Flat(snes));

            ICore core = Make(engine);
            core.LoadRom(nes);
            Assert.False(File.Exists(SaveOf(engine, nes)), "the NES game copied the SNES game's save");
            Assert.NotEqual(Pattern(10)[0x40], ReadRam(core, engine, 0x40));
            Play(core, 10);
            Done(core);

            var cart = new EmuSen.Cores.Nintendo.Venus.Memory.Cartridge(snes, batteryRamDisabled: false);
            Assert.Equal(Pattern(10)[0x40], cart.Read8(SramAddress + 0x40));
            AssertUnchanged(before, Flat(snes));
            Assert.Equal(Pattern(10), File.ReadAllBytes(SaveLibrary.SramPathFor(snes, BatterySave.Snes)));
            Assert.NotEqual(Pattern(10), File.ReadAllBytes(SaveOf(engine, nes)));
        }

        public static TheoryData<string> NesEngines => new() { "Moon", "MoonRT" };

        // A save already in the console's folder wins over the flat one, which is then not read.
        [Fact]
        public void A_save_in_the_consoles_folder_wins_over_a_flat_one()
        {
            string rom = Path.Combine(_library, "Both.smc");
            File.WriteAllBytes(rom, SnesImage());
            PutFlat(rom, Pattern(11));
            Directory.CreateDirectory(Path.Combine(DataStore.Saves, BatterySave.Snes));
            File.WriteAllBytes(SaveLibrary.SramPathFor(rom, BatterySave.Snes), Pattern(12));
            var before = Snapshot(Flat(rom));

            var cart = new EmuSen.Cores.Nintendo.Venus.Memory.Cartridge(rom, batteryRamDisabled: false);
            Assert.Equal(Pattern(12)[3], cart.Read8(SramAddress + 3));
            cart.SaveSram();
            AssertUnchanged(before, Flat(rom));
        }

        // --nobattery with a flat save present: nothing read, copied or written, on the SNES and the N64.
        [Fact]
        public void With_nobattery_a_flat_save_is_neither_read_nor_copied()
        {
            string rom = Path.Combine(_library, "Off.smc");
            File.WriteAllBytes(rom, SnesImage());
            PutFlat(rom, Pattern(13));
            string n64 = Path.Combine(_library, "Off.z64");
            File.WriteAllBytes(n64, SyntheticN64System.Build());
            var before = Snapshot(Flat(rom));

            var cart = new EmuSen.Cores.Nintendo.Venus.Memory.Cartridge(rom, batteryRamDisabled: true);
            Assert.NotEqual(Pattern(13)[5], cart.Read8(SramAddress + 5));
            cart.SaveSram();
            var mars = new EmuSen.Cores.Nintendo.Mars.MarsCore(batteryRamDisabled: true);
            mars.LoadRom(n64);
            mars.SaveSram();

            Assert.Equal(new[] { Path.GetFileName(Flat(rom)) }, Directory.EnumerateFileSystemEntries(DataStore.Saves).Select(Path.GetFileName).ToArray());
            AssertUnchanged(before, Flat(rom));
        }

        // A Venus state carries the saving game's save path and --nobattery latch, which used to steer the loader's save.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_venus_state_from_another_game_leaves_the_save_where_this_game_opened_it(bool saverWithoutBattery)
        {
            string a = Path.Combine(_library, "A.smc"), b = Path.Combine(_library, "B.smc");
            File.WriteAllBytes(a, SyntheticRom.Build((0x7FD8, new byte[] { 0x03 })));
            File.WriteAllBytes(b, SyntheticRom.Build((0x7FD8, new byte[] { 0x03 })));

            CoreOptions.BatteryRamDisabled = saverWithoutBattery;
            var saver = new VenusCore(headless: true);
            saver.LoadRom(a);
            saver.RunFrame();
            using var state = new MemoryStream();
            saver.SaveState(state);

            CoreOptions.BatteryRamDisabled = false;
            var loader = new VenusCore(headless: true);
            loader.LoadRom(b);
            loader.LoadState(new MemoryStream(state.ToArray()));
            loader.SaveSram();

            Assert.True(File.Exists(SaveLibrary.SramPathFor(b, BatterySave.Snes)), "the loader did not write its own game's save");
            Assert.False(File.Exists(SaveLibrary.SramPathFor(a, BatterySave.Snes)), "the loader wrote the saving game's save");
        }

        private static byte ReadRam(ICore core, string engine, int address) => engine switch
        {
            "Moon" => ((MoonCore)core).ReadSpace(RamSpace(engine), address),
            "MoonRT" => ((MoonRtCore)core).ReadSpace(RamSpace(engine), address),
            "Mercury" => ((MercuryCore)core).ReadSpace(RamSpace(engine), address),
            _ => ((MercuryRtCore)core).ReadSpace(RamSpace(engine), address),
        };

        private static void WriteRam(ICore core, string engine, int address, byte value)
        {
            switch (engine)
            {
                case "Moon": ((MoonCore)core).WriteSpace(RamSpace(engine), address, value); break;
                case "MoonRT": ((MoonRtCore)core).WriteSpace(RamSpace(engine), address, value); break;
                case "Mercury": ((MercuryCore)core).WriteSpace(RamSpace(engine), address, value); break;
                default: ((MercuryRtCore)core).WriteSpace(RamSpace(engine), address, value); break;
            }
        }
    }
}
