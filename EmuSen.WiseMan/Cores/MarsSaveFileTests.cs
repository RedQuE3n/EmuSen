using System;
using System.IO;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The save chip and the Controller Pak through MarsCore and onto disk - see Mars_Save.md §7.
    // Passes the battery switch per core, since other tests arm --nobattery for the whole process - see §7.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsSaveFileTests : IDisposable
    {
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenMars_" + Guid.NewGuid().ToString("N"));

        public MarsSaveFileTests()
        {
            Directory.CreateDirectory(_dir);
            DataStore.OverrideDirectory = Path.Combine(_dir, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            if (Directory.Exists(_dir)) Directory.Delete(_dir, recursive: true);
        }

        [Fact]
        public void An_eeprom_the_game_wrote_is_saved_and_named_by_its_length_next_time()
        {
            string rom = WriteRom("Eeprom");

            var first = new MarsCore(batteryRamDisabled: false);
            first.LoadRom(rom);
            WriteEepromBlock(first.Bus!, 2, 0x5A);
            first.SaveSram();

            Assert.Equal(Eeprom.Size, new FileInfo(SaveLibrary.SramPathFor(rom)).Length);

            var second = new MarsCore(batteryRamDisabled: false);
            second.LoadRom(rom);

            Assert.Equal(N64SaveType.Eeprom4k, second.Bus!.Save.Type);
            Assert.Equal(0x5A, second.Bus.Save.Eeprom!.Data[16]);
        }

        [Fact]
        public void Nothing_is_written_when_the_game_changed_nothing()
        {
            string rom = WriteRom("Untouched");

            var core = new MarsCore(batteryRamDisabled: false);
            core.LoadRom(rom);
            core.SaveSram();

            Assert.False(File.Exists(SaveLibrary.SramPathFor(rom)));
            Assert.False(File.Exists(Path.ChangeExtension(SaveLibrary.SramPathFor(rom), MarsCore.PakExtension)));
        }

        [Fact]
        public void The_controller_pak_is_in_the_first_port_and_keeps_its_own_file()
        {
            string rom = WriteRom("Pak");

            var first = new MarsCore(batteryRamDisabled: false);
            first.LoadRom(rom);
            ControllerPak pak = first.Bus!.Si.Controllers[0].Pak!;
            pak.Write(0x0400, new byte[] { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32 });
            first.SaveSram();

            var second = new MarsCore(batteryRamDisabled: false);
            second.LoadRom(rom);

            Assert.Equal(ControllerPak.Size, new FileInfo(Path.ChangeExtension(SaveLibrary.SramPathFor(rom), MarsCore.PakExtension)).Length);
            Assert.Equal(32, second.Bus!.Si.Controllers[0].Pak!.Data[0x041F]);
            Assert.Null(second.Bus.Si.Controllers[1].Pak);
        }

        [Fact]
        public void A_saved_chip_is_written_again_only_after_it_changes_again()
        {
            string rom = WriteRom("Twice");
            string save = SaveLibrary.SramPathFor(rom);

            var core = new MarsCore(batteryRamDisabled: false);
            core.LoadRom(rom);
            WriteEepromBlock(core.Bus!, 0, 0x11);
            core.SaveSram();
            File.Delete(save);

            core.SaveSram();
            Assert.False(File.Exists(save));

            WriteEepromBlock(core.Bus!, 0, 0x22);
            core.SaveSram();
            Assert.True(File.Exists(save));
        }

        // Fields of a couple of thousand cycles, so three hundred frames cost next to nothing - see Mars_Save.md §7.
        [Fact]
        public void A_changed_chip_is_written_on_the_three_hundredth_frame_without_being_asked()
        {
            string rom = WriteRom("Autosave");
            string save = SaveLibrary.SramPathFor(rom);

            var core = new MarsCore(batteryRamDisabled: false);
            core.LoadRom(rom);
            core.Bus!.Write32(MemoryMap.ViBase + EmuSen.Cores.Nintendo.Mars.Vi.Vi.VerticalSync, 0x20);
            core.Bus.Write32(MemoryMap.ViBase + EmuSen.Cores.Nintendo.Mars.Vi.Vi.HorizontalSync, 0x40);
            WriteEepromBlock(core.Bus, 1, 0x33);

            for (int frame = 1; frame < 300; frame++) core.RunFrame();
            Assert.False(File.Exists(save));

            core.RunFrame();
            Assert.Equal(300, core.TotalFrames);
            Assert.True(File.Exists(save));
        }

        private static readonly byte[] SpinForever = { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 };

        private string WriteRom(string name)
        {
            string path = Path.Combine(_dir, name + ".z64");
            File.WriteAllBytes(path, SyntheticN64Rom.Build(patches: (0, SpinForever)));
            return path;
        }

        // An eight-byte write on the fifth channel, as a game's EEPROM driver sends it.
        private static void WriteEepromBlock(MemoryBus bus, byte block, byte value)
        {
            var ram = new byte[64];
            byte[] command = { 0, 0, 0, 0, 10, 1, 0x05, block, value, value, value, value, value, value, value, value, 0, 0xFE };
            command.CopyTo(ram, 0);
            Joybus.Run(ram, bus.Si.Controllers, bus.Save);
        }
    }
}
