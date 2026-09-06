using EmuSen.Common.Imaging;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;

namespace EmuSen.WiseMan.Cores
{
    // Shared by the seven MercuryCommercialRom*Tests classes - see Mercury_RealCartridges.md and §3.56.
    public static class MercuryCommercialRom
    {
        // Long enough for every ROM tried so far to clear its logo and reach a title screen.
        public const int BootFrames = 420;

        // Long enough for a title screen tapped with Start to reach something that plays music.
        public const int PlayFrames = 900;

        public static TheoryData<string> Roms => Fixtures.CommercialRomLibrary.AsTheoryData(".gb", ".gbc");

        public static MercuryCore Load(string path)
        {
            // The battery save would make every run depend on the last one - see Mercury_RealCartridges.md §2.
            // Only ever assigned true anywhere in this suite, which is what lets these classes run in parallel.
            CoreOptions.BatteryRamDisabled = true;

            var core = new MercuryCore();
            core.LoadRom(path);
            return core;
        }

        public static ulong RunTo(MercuryCore core, int frames)
        {
            for (int i = 0; i < frames; i++) core.RunFrame();
            return FrameHash.Compute(core.GetFrameBufferRgba());
        }
    }
}
