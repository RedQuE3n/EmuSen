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
    // Every battery save in the Saves folder, and the NES and Game Boy saves a build before 2026-09-28 wrote beside the ROM copied there once - see EmuSen_Settings_Reference.md §4.85.6.
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
            Assert.True(File.Exists(SaveLibrary.SramPathFor(rom)), "the save was not copied into Saves");
            Assert.Equal(Pattern(1), File.ReadAllBytes(SaveLibrary.SramPathFor(rom)));
            Assert.Equal(Pattern(1)[0x123], ReadRam(core, engine, 0x123));

            WriteRam(core, engine, 0x123, (byte)~Pattern(1)[0x123]);
            Play(core, BatterySave.FlushEveryNFrames + 1);
            Done(core);

            AssertUnchanged(before, Beside(rom));
            Assert.Equal((byte)~Pattern(1)[0x123], File.ReadAllBytes(SaveLibrary.SramPathFor(rom))[0x123]);
        }

        [Theory]
        [MemberData(nameof(Engines))]
        public void A_save_already_in_saves_wins_over_one_beside_the_rom(string engine)
        {
            string rom = Rom(engine, $"Both {engine}");
            File.WriteAllBytes(Beside(rom), Pattern(2));
            Directory.CreateDirectory(DataStore.Saves);
            File.WriteAllBytes(SaveLibrary.SramPathFor(rom), Pattern(3));
            var before = Snapshot(Beside(rom));

            ICore core = Make(engine);
            core.LoadRom(rom);
            Assert.Equal(Pattern(3)[0x77], ReadRam(core, engine, 0x77));
            Assert.Equal(Pattern(3), File.ReadAllBytes(SaveLibrary.SramPathFor(rom)));
            Play(core, 10);
            Done(core);

            AssertUnchanged(before, Beside(rom));
            Assert.Equal(Pattern(3), File.ReadAllBytes(SaveLibrary.SramPathFor(rom)));
        }

        [Theory]
        [MemberData(nameof(Engines))]
        public void A_game_with_no_save_anywhere_gets_one_only_in_saves(string engine)
        {
            string rom = Rom(engine, $"Fresh {engine}");

            ICore core = Make(engine);
            core.LoadRom(rom);
            Assert.False(File.Exists(SaveLibrary.SramPathFor(rom)));
            Play(core, 10);
            Done(core);

            Assert.True(File.Exists(SaveLibrary.SramPathFor(rom)));
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

            Assert.False(Directory.Exists(DataStore.Saves) && Directory.EnumerateFileSystemEntries(DataStore.Saves).Any(), "a save was written into Saves");
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

            Assert.False(File.Exists(SaveLibrary.SramPathFor(rom)));
            Assert.Equal(Pattern(5), File.ReadAllBytes(Beside(rom)));
        }

        // The SNES and the N64 wrote to Saves already, so nothing beside their ROMs is taken up.
        [Fact]
        public void Only_the_nes_and_game_boy_copy_from_beside_the_rom()
        {
            string rom = Path.Combine(_library, "Snes.smc");
            File.WriteAllBytes(rom, SyntheticRom.Build((0x7FD8, new byte[] { 0x03 })));
            File.WriteAllBytes(Beside(rom), Pattern(6));

            var core = new VenusCore(headless: true);
            core.LoadRom(rom);
            Assert.False(File.Exists(SaveLibrary.SramPathFor(rom)));
            Assert.Equal(Pattern(6), File.ReadAllBytes(Beside(rom)));
        }

        // A Venus state carries the saving session's save path and --nobattery latch, which used to steer the loader's save.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_venus_state_from_another_game_leaves_the_save_where_this_session_opened_it(bool saverWithoutBattery)
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

            Assert.True(File.Exists(SaveLibrary.SramPathFor(b)), "the loader did not write its own game's save");
            Assert.False(File.Exists(SaveLibrary.SramPathFor(a)), "the loader wrote the saving game's save");
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
