using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The v1 adapter over the test cores: the loader's checks, the engine SPI with every capability and with none - see EmuSen_CoreAPI.md §19.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreAdapterTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoreAdapter_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public CoreAdapterTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = false;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        public static string LibraryPath(string name) => Path.Combine(AppContext.BaseDirectory,
            OperatingSystem.IsWindows() ? $"{name}.dll" : OperatingSystem.IsMacOS() ? $"lib{name}.dylib" : $"lib{name}.so");

        public static CoreLibrary Full => CoreLibrary.Open(LibraryPath("v1_test_core"));
        public static CoreLibrary Plain => CoreLibrary.Open(LibraryPath("v1_plain_core"));

        // The test core's image: its magic, a little-endian length, the payload.
        public static byte[] Image(params byte[] payload) => "V1TC"u8.ToArray().Concat(BitConverter.GetBytes((ushort)payload.Length)).Concat(payload).ToArray();

        private string Rom(byte[] image, string name = "game.tst")
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        [Fact]
        public void The_loader_accepts_both_test_cores_once_and_refuses_what_is_not_a_v1_core()
        {
            Assert.True(Full.Available, Full.Report);
            Assert.True(Plain.Available, Plain.Report);
            Assert.Same(Full, CoreLibrary.Open(LibraryPath("v1_test_core")));
            Assert.Equal(("v1-test-core", 0x3FFFFFUL, 0x10009u, 4), (Full.Info.Id, Full.Capabilities, Full.AbiVersion, Full.Settings.Count));
            Assert.Equal(("v1-plain-core", 0UL), (Plain.Info.Id, Plain.Capabilities));
            Assert.EndsWith("core ABI 1.9", Full.Report);
            Assert.Equal(new[] { ".tst" }, Full.Info.Systems.Single().Extensions);
            Assert.Equal("the image is empty", Full.Words(-9));

            var missing = CoreLibrary.Open(Path.Combine(_root, "no_such_core.so"));
            Assert.False(missing.Available);
            Assert.EndsWith("not found", missing.Report);
            string sqlite = Path.Combine(AppContext.BaseDirectory, "runtimes",
                (OperatingSystem.IsWindows() ? "win-" : OperatingSystem.IsMacOS() ? "osx-" : "linux-") + System.Runtime.InteropServices.RuntimeInformation.OSArchitecture.ToString().ToLowerInvariant(), "native",
                OperatingSystem.IsWindows() ? "e_sqlite3.dll" : OperatingSystem.IsMacOS() ? "libe_sqlite3.dylib" : "libe_sqlite3.so");
            var other = CoreLibrary.Open(sqlite);
            Assert.False(other.Available, sqlite);
            Assert.Contains("not on the core ABI", other.Report);
        }

        // Since 1.5 machine info's state may bound the frames whose pictures settle after a load; a core that writes nothing, every core before 1.5, reads as 0 - see EmuSen_CoreAPI.md §6.9.
        [Fact]
        public void A_bound_of_settling_pictures_is_read_from_the_state_and_absent_is_zero()
        {
            const string state = "\"state\":{\"format\":\"T\",\"version\":1,\"kinds\":[0]";
            Assert.Equal(0, CoreDescriptorReader.MachineInfo("{\"system\":\"test\"," + state + "}}", 32000).PictureSettles);
            Assert.Equal(240, CoreDescriptorReader.MachineInfo("{\"system\":\"test\"," + state + ",\"picture_settles\":240}}", 32000).PictureSettles);
            using var engine = new CoreEngine(Full);
            engine.LoadRom(Rom(Image(1, 2, 3)));
            Assert.Equal(0, engine.Machine.Info.PictureSettles);
        }

        // Since core ABI 1.6: a machine of a MEDIA core that lists no drive offers no disc change and refuses the lid and an insert, the core answering NO_SUCH_DRIVE; it runs and its states load - see EmuSen_CoreAPI.md §6.26.
        [Fact]
        public void A_media_core_whose_machine_lists_no_drive_offers_no_disc_change()
        {
            using var disc = new CoreEngine(Full);
            disc.LoadRom(Rom(Image(1, 2, 3)));
            Assert.Equal((1, true), (disc.Machine.Info.Drives.Count, disc.CanChangeDisc));
            using var engine = new CoreEngine(Full);
            engine.LoadRom(Rom(Image(0xCA, 2, 3), "cartridge.tst"));
            Assert.True(Full.Has(CoreInterface.CapMedia));
            Assert.Equal((0, false, false, 1), (engine.Machine.Info.Drives.Count, engine.CanChangeDisc, engine.LidOpen, engine.Discs.Count));
            Assert.Throws<InvalidOperationException>(() => engine.SetLid(true));
            Assert.Throws<InvalidOperationException>(() => engine.InsertDisc(0));
            Assert.Throws<InvalidOperationException>(() => engine.ChangeDisc(0));
            Assert.Equal(-267, Assert.Throws<CoreRefusedException>(() => engine.Machine.MediaLid(0, true)).Status);
            Assert.Equal(-267, Assert.Throws<CoreRefusedException>(() => engine.Machine.MediaInsert(0, Image(9), Array.Empty<(uint, byte[])>(), restore: false)).Status);
            for (int i = 0; i < 3; i++) engine.RunFrame();
            using var saved = new MemoryStream();
            engine.SaveState(saved);
            engine.RunFrame();
            engine.LoadState(new MemoryStream(saved.ToArray()));
            using var again = new MemoryStream();
            engine.SaveState(again);
            Assert.Equal(saved.ToArray(), again.ToArray());
            Assert.Equal(0, engine.Machine.Info.Drives.Count);
        }

        [Fact]
        public void A_refused_game_is_thrown_with_the_cores_own_words()
        {
            using var engine = new CoreEngine(Full);
            var e = Assert.Throws<CoreRefusedException>(() => engine.LoadRom(Rom("garbage!"u8.ToArray())));
            Assert.Equal((-264, "V1TestCore (test) refused the game: not a test core image."), (e.Status, e.Message));
            Assert.False(engine.IsRomLoaded);
        }

        [Fact]
        public void The_engine_runs_frames_draws_sounds_and_takes_input_from_the_descriptors()
        {
            using var engine = new CoreEngine(Full);
            engine.LoadRom(Rom(Image(1, 2, 3)));
            Assert.Equal((8, 4, 60.0, 32000), (engine.ScreenWidth, engine.ScreenHeight, engine.FrameRateHz, engine.AudioSampleRate));
            Assert.Equal(new[] { PadButton.A }, engine.SupportedButtons);
            Assert.Equal(new[] { PadAxis.LeftX }, engine.SupportedAxes);
            for (int i = 0; i < 3; i++) engine.RunFrame();
            Assert.Equal(3, engine.TotalFrames);
            byte[] picture = engine.GetFrameBufferRgba();
            Assert.Equal(8 * 4 * 4, picture.Length);
            Assert.Equal((byte)3, picture[0]);
            Assert.Equal(3, engine.FrameSerial);
            short[] sound = engine.DequeueAudioSamples(4096);
            Assert.Equal(3 * 2 * (32000 / 60), sound.Length);
            Assert.Equal(new[] { "count" }, engine.LastFramePhases.Select(p => p.Name));
            Assert.Equal(1, engine.StateVersion);

            engine.SetButton(0, PadButton.A, true);
            engine.RunFrame();
            Assert.Equal((byte)(3 + 1), engine.ReadSpace("RAM", 3));
            Assert.Contains(engine.RecentLog, l => l == "info\ttest\tcreated with 3 bytes");
        }

        [Fact]
        public void Settings_come_from_the_schema_and_a_run_time_change_resizes_the_picture()
        {
            using var engine = new CoreEngine(Full, new Dictionary<string, string> { ["Ram"] = "9", ["Unknown"] = "1" });
            Assert.Equal(new[] { "Wide", "Rate", "Ram", "Threads" }, engine.Settings.Select(s => s.Key));
            Assert.Equal(CoreSettingKind.Switch, engine.Settings[0].Kind);
            engine.LoadRom(Rom(Image()));
            Assert.Equal((byte)9, engine.ReadSpace("RAM", 0));
            engine.Set("Wide", "true");
            Assert.Equal(16, engine.ScreenWidth);
            Assert.Equal("The picture is drawn twice as wide.", engine.Settings[0].Note!(_ => ""));
            engine.Set("Rate", "48000");
            engine.RunFrame();
            engine.DequeueAudioSamples(1 << 16);
            Assert.Equal(48000, engine.AudioSampleRate);
            Assert.Throws<ArgumentException>(() => engine.Set("Wide", "maybe"));
            engine.Set("Ram", "4");
            Assert.Equal((byte)9, engine.ReadSpace("RAM", 0));
        }

        [Fact]
        public void States_round_trip_and_the_snapshot_is_the_cores_own_kind()
        {
            using var engine = new CoreEngine(Full);
            engine.LoadRom(Rom(Image(5)));
            engine.RunFrame();
            var saved = new MemoryStream();
            engine.SaveState(saved);
            var snap = new MemoryStream();
            engine.SaveSnapshot(snap);
            Assert.Equal("TSN1"u8.ToArray(), snap.ToArray()[..4]);
            engine.RunFrame();
            engine.LoadState(new MemoryStream(snap.ToArray()));
            var again = new MemoryStream();
            engine.SaveState(again);
            Assert.Equal(saved.ToArray(), again.ToArray());
            var e = Assert.Throws<InvalidDataException>(() => engine.LoadState(new MemoryStream("XXXX"u8.ToArray())));
            Assert.Equal("V1TestCore (test) refused the state: not this core's state.", e.Message);
        }

        [Fact]
        public void A_core_without_a_capability_gets_the_neutral_answer_and_a_run_time_setting_waits_for_the_next_load()
        {
            using var engine = new CoreEngine(Plain);
            engine.LoadRom(Rom(Image(5)));
            engine.RunFrame();
            engine.RunFrame();
            Assert.Empty(engine.SupportedAxes);
            engine.SetAxis(0, PadAxis.LeftX, 1);
            Assert.Equal(2, engine.FrameSerial);
            Assert.Equal(1, engine.RowRepeat);
            Assert.Equal(new[] { "frame" }, engine.LastFramePhases.Select(p => p.Name));
            engine.Set("Wide", "true");
            Assert.Equal(8, engine.ScreenWidth);
            var snap = new MemoryStream();
            engine.SaveSnapshot(snap);
            Assert.Equal("TST1"u8.ToArray(), snap.ToArray()[..4]);
            var target = engine.CreateDebugTarget();
            Assert.Empty(target.Disassemble("RAM", 0, 4));
            Assert.Empty(target.GetAudioSamples().Samples);
            Assert.All(target.CpuRegisters.Current, r => Assert.Equal(0UL, r.Value));
            engine.LoadRom(Rom(Image(5), "again.tst"));
            Assert.Equal(16, engine.ScreenWidth);
        }

        [Fact]
        public void Cheats_reach_the_core_as_pokes_it_gates_and_the_rest_are_applied_here()
        {
            using var engine = new CoreEngine(Full);
            engine.LoadRom(Rom(Image()));
            engine.Cheats.AddRamPoke("RAM", 5, 0x99, "set");
            engine.Cheats.AddCheat(CheatKind.RamPoke, new[] { new CheatWrite { Space = "RAM", Address = 6, Value = 2, Width = 1, Type = CheatWriteType.Increase } }, null, "add");
            engine.RunFrame();
            Assert.Equal(((byte)0x99, (byte)2), (engine.ReadSpace("RAM", 5), engine.ReadSpace("RAM", 6)));
            using var plain = new CoreEngine(Plain);
            plain.LoadRom(Rom(Image(), "plain.tst"));
            plain.Cheats.AddRamPoke("RAM", 5, 0x77, "set");
            plain.RunFrame();
            Assert.Equal((byte)0x77, plain.ReadSpace("RAM", 5));
        }

        // A registry handed over, as CoreFactory.Bundle hands a frontend's, applies at once, before any frame - see EmuSen_CoreAPI.md §26.
        [Fact]
        public void A_registry_handed_over_applies_before_the_next_frame()
        {
            using var plain = new CoreEngine(Plain);
            plain.LoadRom(Rom(Image(), "handed.tst"));
            var handed = new CheatRegistry();
            handed.AddRamPoke("RAM", 5, 0x55, "set");
            ((ICheatRegistryHost)plain).Cheats = handed;
            plain.ApplyCheats();
            Assert.Equal((byte)0x55, plain.ReadSpace("RAM", 5));
        }

        // Since 1.8 core info's slots carry a medium and a label, a console file its kind, and a kind this host does not know reads as the game's - see EmuSen_CoreAPI.md §6.28.
        [Fact]
        public void Core_infos_battery_slots_are_read_with_kind_medium_and_label()
        {
            var listed = Full.Info.Systems.Single().Battery!;
            Assert.Equal(new (uint, CoreBatteryKind, string?, string?, int?)[] { (0, CoreBatteryKind.Game, null, null, null), (1, CoreBatteryKind.Removable, "test.card", "Card slot", null), (2, CoreBatteryKind.Console, "test.memory", "Inside the console", null) },
                listed.Select(b => (b.Which, b.Kind, b.Medium, b.Label, b.Port)));
            var read = CoreDescriptorReader.Info(Full.Info.Text.Replace("\"kind\":\"console\"", "\"kind\":\"later\"").Replace("\"label\":\"Card slot\"", "\"label\":\"Card slot\",\"port\":1"));
            Assert.Equal((CoreBatteryKind.Game, 1), (read.Systems.Single().Battery![2].Kind, read.Systems.Single().Battery![1].Port));
        }

        [Fact]
        public void A_battery_file_is_written_when_the_core_says_it_changed_and_read_at_the_next_load()
        {
            string rom = Rom(Image());
            using (var engine = new CoreEngine(Full))
            {
                engine.LoadRom(rom);
                engine.SetButton(0, PadButton.A, true);
                engine.RunFrame();
            }
            string save = Path.Combine(DataStore.Saves, "TEST", "game.srm");
            Assert.True(File.Exists(save), save);
            Assert.Equal(1, File.ReadAllBytes(save)[0]);
            using var next = new CoreEngine(Full);
            next.LoadRom(rom);
            Assert.Equal(1, next.Machine.Battery(0).Data[0]);
        }

        [Fact]
        public void The_debug_target_reads_spaces_registers_and_disassembly_from_the_descriptors()
        {
            using var engine = new CoreEngine(Full);
            engine.LoadRom(Rom(Image(0x20, 0, 0x41)));
            engine.RunFrame();
            var target = engine.CreateDebugTarget();
            var spaces = target.GetMemorySpaces();
            Assert.Equal(new[] { ("RAM", 256, true), ("ROM", 0, false) }, spaces.Select(s => (s.Name, s.Size, s.IsWritable)));
            Assert.Equal((byte)0x20, spaces[0].Read(1));
            target.RefreshProviders();
            Assert.Equal(("F", 1UL, 64), (target.CpuRegisters.Current[0].Name, target.CpuRegisters.Current[0].Value, target.CpuRegisters.Current[0].BitWidth));
            Assert.Equal("counter", target.DebugCpus.Single().Name);
            var code = target.Disassemble("RAM", 1, 3);
            Assert.Equal(new[] { "$20", "$01", "$41" }, code.Select(c => c.OperandText));
            Assert.Equal((StaticReferenceKind.Call, 0x20), target.ClassifyStaticReference(code[0]));
            Assert.Null(target.ClassifyStaticReference(code[2]));
        }

        [Fact]
        public void Firmware_is_asked_for_from_the_image_alone()
        {
            using var engine = new CoreEngine(Full);
            var wanted = engine.GetFirmwareRequirements(Rom(Image(0xB0)));
            Assert.Equal(("boot.rom", 64, "optional"), (wanted.Single().FileName, wanted.Single().Size, wanted.Single().Purpose));
            Assert.Empty(engine.GetFirmwareRequirements(Rom(Image(1), "other.tst")));
        }
    }
}
