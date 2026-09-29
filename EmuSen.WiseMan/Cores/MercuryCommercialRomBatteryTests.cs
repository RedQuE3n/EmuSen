using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    public class MercuryCommercialRomBatteryTests
    {
        public static TheoryData<string> Cartridges => Roms;

        // The switch is documented as honoured by every core, and Mercury did not honour it - see §2.
        // Mercury latches it in LoadSram during LoadRom, so the assertion is about this core's own
        // load rather than about the switch's value now - which is what makes this safe in parallel.
        [Theory]
        [MemberData(nameof(Cartridges))]
        public void Disabling_the_battery_leaves_no_save_beside_the_rom_or_in_saves(string path)
        {
            if (path.Length == 0) return;

            string beside = Path.ChangeExtension(path, ".srm"), saves = EmuSen.Galaxia.Library.SaveLibrary.SramPathFor(path, EmuSen.Cores.BatterySave.GameBoyFolder(path));
            bool besideBefore = File.Exists(beside), savesBefore = File.Exists(saves);

            var core = Load(path);
            for (int i = 0; i < 320; i++) core.RunFrame();
            core.SaveSram();

            Assert.Equal(besideBefore, File.Exists(beside));
            Assert.Equal(savesBefore, File.Exists(saves));
        }
    }
}
