using System;
using System.Reflection;
using EmuSen.Endymion.Input;

namespace EmuSen.WiseMan.Input
{
    // The hot-plug rescan's timing: a pad dropped before any rescan must not overflow the clock, as Mistress's crash logs of 2026-09-26 showed - see EmuSen_Settings_Reference.md §4.4.
    public class GamepadRescanTests
    {
        private static bool RescanDue(TimeSpan now, TimeSpan last) =>
            (bool)typeof(GamepadManager).GetMethod("RescanDue", BindingFlags.Static | BindingFlags.NonPublic)!.Invoke(null, [now, last])!;

        private static TimeSpan FirstLast() =>
            (TimeSpan)typeof(GamepadManager).GetField("_lastRescan", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(new GamepadManager(new GamepadBindingMap(), start: false))!;

        [Theory]
        [InlineData(0)]
        [InlineData(5)]
        [InlineData(86_400_000)]
        public void A_pad_lost_before_any_rescan_is_looked_for_at_once_without_overflowing(long milliseconds)
        {
            Assert.True(RescanDue(TimeSpan.FromMilliseconds(milliseconds), FirstLast()));
        }

        [Fact]
        public void Rescans_are_a_second_apart()
        {
            Assert.False(RescanDue(TimeSpan.FromMilliseconds(1500), TimeSpan.FromMilliseconds(600)));
            Assert.True(RescanDue(TimeSpan.FromMilliseconds(1600), TimeSpan.FromMilliseconds(600)));
        }
    }
}
