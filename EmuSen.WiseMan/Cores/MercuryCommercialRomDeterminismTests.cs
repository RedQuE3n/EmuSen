using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    public class MercuryCommercialRomDeterminismTests
    {
        public static TheoryData<string> Cartridges => Roms;

        // Two fresh runs of the same image must agree exactly, or nothing else means anything.
        // Deliberately two independent boots - a shared cached run would assume what this proves.
        [Theory]
        [MemberData(nameof(Cartridges))]
        public void The_same_cartridge_run_twice_produces_the_same_frame(string path)
        {
            if (path.Length == 0) return;

            ulong first = RunTo(Load(path), BootFrames);
            ulong second = RunTo(Load(path), BootFrames);

            Assert.Equal(first, second);
        }
    }
}
