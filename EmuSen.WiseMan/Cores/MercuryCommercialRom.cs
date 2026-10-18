using EmuSen.Pharaoh.Imaging;
using EmuSen.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Shared by the seven MercuryCommercialRom*Tests classes, each cartridge on MercuryRT - see Mercury_RealCartridges.md, §3.56 and Mercury_Native.md §12.
    public static class MercuryCommercialRom
    {
        // Long enough for every ROM tried so far to clear its logo and reach a title screen.
        public const int BootFrames = 420;

        // Long enough for a title screen tapped with Start to reach something that plays music.
        public const int PlayFrames = 900;

        // Every cartridge; with none present, one empty case.
        public static TheoryData<string> Roms
        {
            get
            {
                var data = new TheoryData<string>();
                foreach (string path in Fixtures.CommercialRomLibrary.Find(".gb", ".gbc").DefaultIfEmpty(string.Empty)) data.Add(path);
                return data;
            }
        }

        public static MercuryRig Load(string path)
        {
            // No battery save, so no run depends on the last; only ever set true, so the classes run in parallel - see Mercury_RealCartridges.md §2.

            return MercuryRig.Open(path);
        }

        public static ulong RunTo(MercuryRig core, int frames)
        {
            for (int i = 0; i < frames; i++) core.RunFrame();
            return FrameHash.Compute(core.GetFrameBufferRgba());
        }
    }
}
