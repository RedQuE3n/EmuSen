using System;
using EmuSen.Serenity;

namespace EmuSen.WiseMan.Serenity
{
    // The render timer held at the game's rate, found by name since Avalonia keeps it private - see EmuSen_Settings_Reference.md §4.87.10.
    public class RenderTimerRateTests
    {
        private static object SleepingTimer(int fps) =>
            Activator.CreateInstance(typeof(Avalonia.AvaloniaObject).Assembly.GetType("Avalonia.Rendering.SleepLoopRenderTimer", throwOnError: true)!, fps)!;

        private static int RateOf(object timer) => (int)timer.GetType().GetProperty("DesiredFps")!.GetValue(timer)!;

        [Fact]
        public void The_sleeping_timer_is_found_by_name_and_held_at_the_rate_asked_for()
        {
            object timer = SleepingTimer(120);
            Assert.True(RenderTimerRate.Hold(timer, GameFrameControl.DefaultRenderFps));
            Assert.Equal(1000, RateOf(timer));
            Assert.True(RenderTimerRate.Hold(timer, 120));
            Assert.Equal(120, RateOf(timer));
        }

        [Fact]
        public void Any_other_timer_and_a_rate_below_one_are_left_alone()
        {
            Assert.False(RenderTimerRate.Hold(new object(), 1000));
            object timer = SleepingTimer(120);
            Assert.False(RenderTimerRate.Hold(timer, 0));
            Assert.Equal(120, RateOf(timer));
        }

        [Fact]
        public void The_default_is_a_millisecond_a_tick()
        {
            Assert.Equal(1000, GameFrameControl.DefaultRenderFps);
        }
    }
}
