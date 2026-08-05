using System;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Memory
{
    // The cartridge save, end to end through Galaxia - see EmuSen_Galaxia.md §5 and Venus_Memory.md §2.4.
    public class CartridgeSavePathTests : IDisposable
    {
        private readonly string _dir = Path.Combine(
            Path.GetTempPath(), "EmuSenCart_" + Guid.NewGuid().ToString("N"));

        private readonly string _romDir;

        // SRAM size $03 is 8KB, and bank $70 is where LoROM maps it - see Venus_Memory.md §2.1a.
        private const int SramSizeOffset = 0x7FD8;
        private const uint SramAddress = 0x700000;

        public CartridgeSavePathTests()
        {
            _romDir = Path.Combine(_dir, "Games");
            Directory.CreateDirectory(_romDir);
            DataStore.OverrideDirectory = Path.Combine(_dir, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreOptions.BatteryRamDisabled = false;
            if (Directory.Exists(_dir)) Directory.Delete(_dir, recursive: true);
        }

        private string WriteRom(string name)
        {
            string path = Path.Combine(_romDir, name + ".smc");
            File.WriteAllBytes(path, SyntheticRom.Build((SramSizeOffset, new byte[] { 0x03 })));
            return path;
        }

        [Fact]
        public void The_save_goes_to_the_Saves_directory_not_beside_the_rom()
        {
            var cart = new Cartridge(WriteRom("Synthetic"));

            Assert.Equal(Path.Combine(DataStore.Saves, "Synthetic.srm"), cart.SavePath);
            Assert.Equal(SaveLibrary.SramPathFor(WriteRom("Synthetic")), cart.SavePath);
        }

        [Fact]
        public void Sram_survives_a_save_and_a_fresh_load()
        {
            string rom = WriteRom("RoundTrip");

            var first = new Cartridge(rom);
            first.Write8(SramAddress, 0xA5);
            first.Write8(SramAddress + 1, 0x5A);
            first.SaveSram();

            Assert.Equal(0xA5, new Cartridge(rom).Read8(SramAddress));
            Assert.Equal(0x5A, new Cartridge(rom).Read8(SramAddress + 1));
        }

        // The durability fix: the write is a rename, so no partial file is left around.
        [Fact]
        public void Saving_leaves_no_temp_file_beside_the_save()
        {
            var cart = new Cartridge(WriteRom("NoTemp"));
            cart.Write8(SramAddress, 0x11);
            cart.SaveSram();

            Assert.True(File.Exists(cart.SavePath));
            Assert.False(File.Exists(cart.SavePath + AtomicFile.TempSuffix));
        }

        [Fact]
        public void Nothing_is_written_beside_the_rom()
        {
            var cart = new Cartridge(WriteRom("Clean"));
            cart.Write8(SramAddress, 0x22);
            cart.SaveSram();

            Assert.Equal(new[] { "Clean.smc" }, Directory.GetFiles(_romDir).Select(Path.GetFileName).ToArray());
        }

        // --nobattery must neither read nor write - see EmuSen_Multicore.md §6.
        [Fact]
        public void A_disabled_battery_writes_nothing_at_all()
        {
            string rom = WriteRom("NoBattery");
            CoreOptions.BatteryRamDisabled = true;

            var cart = new Cartridge(rom);
            cart.Write8(SramAddress, 0x33);
            cart.SaveSram();

            Assert.False(File.Exists(cart.SavePath));
        }
    }
}
