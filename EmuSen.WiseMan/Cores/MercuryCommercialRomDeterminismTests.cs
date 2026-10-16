using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    public class MercuryCommercialRomDeterminismTests
    {
        public static TheoryData<string> Cartridges => Roms;

        // Two independent fresh boots of one image agree exactly; a shared cached run would assume what this proves.
        [Theory]
        [MemberData(nameof(Cartridges))]
        public void The_same_cartridge_run_twice_produces_the_same_frame(string path)
        {
            if (path.Length == 0) return;

            using MercuryRig a = Load(path), b = Load(path);
            ulong first = RunTo(a, BootFrames);
            ulong second = RunTo(b, BootFrames);

            Assert.Equal(first, second);
        }
    }
}
