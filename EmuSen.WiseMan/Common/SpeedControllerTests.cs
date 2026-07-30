using System;
using EmuSen.Common;

namespace EmuSen.WiseMan.Common
{
    // The policy fast-forward is built on - see EmuSen_Rewind_And_FastForward.md §2.
    public class SpeedControllerTests
    {
        private const double Ntsc = 60.0985;

        [Fact]
        public void Normal_speed_paces_at_the_cores_own_rate()
        {
            var s = new SpeedController();
            Assert.Equal(1.0 / Ntsc, s.FrameInterval(Ntsc).TotalSeconds, precision: 6);
        }

        [Fact]
        public void Three_hundred_percent_paces_at_a_third_of_a_frame()
        {
            var s = new SpeedController { SpeedPercent = 300 };
            Assert.Equal(1.0 / (Ntsc * 3), s.FrameInterval(Ntsc).TotalSeconds, precision: 6);
        }

        // Zero means "no pacing at all", not "infinitely slow" - §2.1.
        [Fact]
        public void Unthrottled_asks_for_no_wait_at_all()
        {
            var s = new SpeedController { SpeedPercent = SpeedController.UnthrottledPercent };
            Assert.Equal(TimeSpan.Zero, s.FrameInterval(Ntsc));
        }

        [Fact]
        public void Slow_motion_paces_slower_than_normal()
        {
            var s = new SpeedController { SpeedPercent = 25 };
            Assert.True(s.FrameInterval(Ntsc) > TimeSpan.FromSeconds(1.0 / Ntsc));
            Assert.True(s.IsSlowMotion);
            Assert.False(s.IsFastForwarding);
        }

        [Fact]
        public void Turbo_toggles_between_the_turbo_rate_and_normal()
        {
            var s = new SpeedController { TurboPercent = 400 };
            s.SetTurbo(true);
            Assert.Equal(400, s.SpeedPercent);
            Assert.True(s.IsFastForwarding);
            s.SetTurbo(false);
            Assert.Equal(SpeedController.NormalPercent, s.SpeedPercent);
            Assert.True(s.IsNormalSpeed);
        }

        [Fact]
        public void Normal_speed_and_below_draws_every_frame()
        {
            Assert.Equal(1, new SpeedController().RenderEveryNthFrame);
            Assert.Equal(1, new SpeedController { SpeedPercent = 25 }.RenderEveryNthFrame);
        }

        // Drawing 1 in N keeps the PRESENTED rate near the hardware rate - §2.2.
        [Fact]
        public void Fast_forward_draws_one_frame_in_every_speed_multiple()
        {
            var s = new SpeedController { SpeedPercent = 300 };
            Assert.Equal(3, s.RenderEveryNthFrame);
            Assert.True(s.ShouldRender(0));
            Assert.False(s.ShouldRender(1));
            Assert.False(s.ShouldRender(2));
            Assert.True(s.ShouldRender(3));
        }

        [Fact]
        public void Frame_skip_is_capped_however_high_the_speed_goes()
        {
            var s = new SpeedController { MaxFrameSkip = 9, SpeedPercent = 5000 };
            Assert.Equal(10, s.RenderEveryNthFrame);
        }

        [Fact]
        public void Unthrottled_skips_at_the_cap()
        {
            var s = new SpeedController { MaxFrameSkip = 9, SpeedPercent = SpeedController.UnthrottledPercent };
            Assert.Equal(10, s.RenderEveryNthFrame);
        }

        // Outside this band production stops matching a real device's rate - §2.3.
        [Fact]
        public void Audio_plays_near_normal_speed_and_is_dropped_outside_it()
        {
            Assert.True(new SpeedController().ShouldPlayAudio);
            Assert.True(new SpeedController { SpeedPercent = 200 }.ShouldPlayAudio);
            Assert.True(new SpeedController { SpeedPercent = 50 }.ShouldPlayAudio);
            Assert.False(new SpeedController { SpeedPercent = 201 }.ShouldPlayAudio);
            Assert.False(new SpeedController { SpeedPercent = 49 }.ShouldPlayAudio);
            Assert.False(new SpeedController { SpeedPercent = SpeedController.UnthrottledPercent }.ShouldPlayAudio);
        }

        [Fact]
        public void A_negative_speed_clamps_to_unthrottled_rather_than_inverting_pacing()
        {
            var s = new SpeedController { SpeedPercent = -50 };
            Assert.Equal(SpeedController.UnthrottledPercent, s.SpeedPercent);
        }
    }
}
