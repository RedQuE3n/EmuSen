using static EmuSen.WiseMan.Cores.MercuryCommercialRom;

namespace EmuSen.WiseMan.Cores
{
    public class MercuryCommercialRomSaveStateTests
    {
        public static TheoryData<string> Cartridges => Roms;

        // The strongest check available without a reference: state that is not saved shows up here.
        [Theory]
        [MemberData(nameof(Cartridges))]
        public void A_save_state_taken_mid_game_resumes_into_the_same_future(string path)
        {
            if (path.Length == 0) return;

            const int resumeFrames = 120;

            var core = Load(path);
            RunTo(core, BootFrames);

            using var stream = new MemoryStream();
            core.SaveState(stream);

            ulong straightThrough = RunTo(core, resumeFrames);

            stream.Position = 0;
            core.LoadState(stream);
            ulong afterReload = RunTo(core, resumeFrames);

            Assert.Equal(straightThrough, afterReload);
        }
    }
}
