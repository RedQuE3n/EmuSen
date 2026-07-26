using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // IDebugTarget.GetHardwareLoad/MaxSprites - backs the `coretop`
    // command's hardware-load bars and sprite-capacity gauge. Covers
    // SnesDebugTarget's own two behaviors: no frame-timings delegate
    // means "not modeled" (an empty list, not fake/zero numbers), and a
    // real delegate gets normalized against a 60fps frame's wall-clock
    // budget and clamped to 100 so a behind-schedule frame can't report
    // over 100% load.
    public class HardwareLoadTests
    {
        private static SnesDebugTarget BuildTarget(System.Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? frameTimings = null)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!, frameTimings);
        }

        [Fact]
        public void No_frame_timings_delegate_reports_not_modeled()
        {
            var target = BuildTarget();
            Assert.Empty(target.GetHardwareLoad());
        }

        [Fact]
        public void Frame_timings_are_normalized_against_the_60fps_budget()
        {
            // ~16.6667ms is a full 60fps frame - half that should read as
            // roughly 50% load.
            var target = BuildTarget(() => (8.33, 4.0, 0.5));

            var load = target.GetHardwareLoad();

            var cpu = Assert.Single(load, l => l.Name == "CPU+SPC700");
            Assert.InRange(cpu.Percent, 49.0, 51.0);

            var ppu = Assert.Single(load, l => l.Name == "PPU");
            Assert.InRange(ppu.Percent, 23.0, 25.0);

            var hdma = Assert.Single(load, l => l.Name == "HDMA");
            Assert.InRange(hdma.Percent, 2.0, 3.5);
        }

        [Fact]
        public void Load_over_a_full_frame_is_clamped_to_100_percent()
        {
            // A frame that ran 3x over budget (e.g. right after a debug
            // prompt ate real wall-clock time) must not report 300%.
            var target = BuildTarget(() => (50.0, 50.0, 50.0));

            var load = target.GetHardwareLoad();

            Assert.All(load, l => Assert.Equal(100.0, l.Percent));
        }

        [Fact]
        public void Max_sprites_is_the_real_snes_oam_capacity()
        {
            var target = BuildTarget();
            Assert.Equal(128, target.MaxSprites);
        }
    }
}
