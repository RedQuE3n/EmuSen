using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    public class MercuryCommercialRomBatteryTests
    {
        public static TheoryData<string> Cartridges => Roms;

        // The switch every core honours, latched at LoadRom, so the claim is about this load and safe in parallel - see Mercury_RealCartridges.md §2.
        [Theory]
        [MemberData(nameof(Cartridges))]
        public void Disabling_the_battery_leaves_no_save_beside_the_rom_or_in_saves(string path)
        {
            if (path.Length == 0) return;

            string beside = Path.ChangeExtension(path, ".srm"), saves = EmuSen.Galaxia.Library.SaveLibrary.SramPathFor(path, EmuSen.Cores.BatterySave.GameBoyFolder(path));
            bool besideBefore = File.Exists(beside), savesBefore = File.Exists(saves);

            using var core = Load(path);
            for (int i = 0; i < 320; i++) core.RunFrame();
            core.SaveSram();

            Assert.Equal(besideBefore, File.Exists(beside));
            Assert.Equal(savesBefore, File.Exists(saves));
        }
    }
}
