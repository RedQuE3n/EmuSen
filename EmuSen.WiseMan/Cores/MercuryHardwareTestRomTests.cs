using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // blargg and mooneye, judged by their own serial output rather than by another emulator - see Mercury_HardwareTests.md.
    public class MercuryHardwareTestRomTests
    {
        // blargg's combined cpu_instrs is the longest of these at roughly half a minute of emulated time.
        private const int FrameCap = 3600;

        public static TheoryData<string> Roms => HardwareTestRomLibrary.AsTheoryData();

        [Theory]
        [MemberData(nameof(Roms))]
        public void A_hardware_test_rom_reports_a_pass(string path)
        {
            if (path.Length == 0) return;

            CoreOptions.BatteryRamDisabled = true;

            var core = new MercuryCore();
            core.LoadRom(path);

            var verdict = TestRomVerdict.NoVerdict;
            for (int frame = 0; frame < FrameCap && verdict == TestRomVerdict.NoVerdict; frame++)
            {
                core.RunFrame();
                verdict = HardwareTestRomLibrary.ReadVerdict(core.Bus!.SerialLog);
            }

            string name = Path.GetRelativePath(HardwareTestRomLibrary.Root, path);
            string transcript = HardwareTestRomLibrary.AsText(core.Bus!.SerialLog);

            Assert.True(verdict == TestRomVerdict.Passed,
                verdict == TestRomVerdict.NoVerdict
                    ? $"{name} printed no verdict in {FrameCap} frames. Serial said: \"{transcript}\""
                    : $"{name} reported a failure. Serial said: \"{transcript}\"");
        }
    }
}
