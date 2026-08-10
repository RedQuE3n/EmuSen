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
        public void Disabling_the_battery_leaves_no_save_beside_the_rom(string path)
        {
            if (path.Length == 0) return;

            string savePath = Path.ChangeExtension(path, ".srm");
            File.Delete(savePath);

            var core = Load(path);
            for (int i = 0; i < 320; i++) core.RunFrame();
            core.SaveSram();

            Assert.False(File.Exists(savePath), $"{Path.GetFileName(path)} wrote {Path.GetFileName(savePath)} despite BatteryRamDisabled.");
        }
    }
}
