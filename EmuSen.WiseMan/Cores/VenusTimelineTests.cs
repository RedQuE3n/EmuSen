using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Venus's folded timeline: the master clock and the line overshoot it carries - see Venus_CPU.md §8.5a.
    [Collection(TestCollections.ProcessGlobals)]
    public class VenusTimelineTests
    {
        [Fact]
        public void A_frame_advances_the_master_clock_by_a_frame_of_scanlines()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());

            core.RunFrame();

            Assert.InRange(core.MasterClock, core.MasterClocksPerFrame, core.MasterClocksPerFrame + VenusCore.CyclesPerScanline);
        }

        // An instruction straddling a boundary is not truncated to it; the overshoot starts the next line.
        [Fact]
        public void The_master_clock_never_runs_backwards_across_frames()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());

            long previous = 0;
            for (int i = 0; i < 5; i++)
            {
                core.RunFrame();
                Assert.True(core.MasterClock > previous, $"frame {i} left the clock at {core.MasterClock}, not past {previous}");
                previous = core.MasterClock;
            }

            Assert.InRange(core.MasterClock - 5 * core.MasterClocksPerFrame, 0, 5 * VenusCore.CyclesPerScanline);
        }
    }
}
