using EmuSen.Pharaoh;

namespace EmuSen.WiseMan.HeadlessDebug
{
    // Argument parsing for --throttle; the re-exec itself needs systemd - see §3.42.
    public class CpuThrottleTests
    {
        private static string[] Args(params string[] extra)
        {
            var args = new List<string> { "rom.smc", "600" };
            args.AddRange(extra);
            return args.ToArray();
        }

        [Fact]
        public void No_throttle_flag_parses_as_zero_and_no_error()
        {
            Assert.True(CpuThrottle.TryParsePercent(Args("--nobattery"), out int percent, out string? error));
            Assert.Equal(0, percent);
            Assert.Null(error);
        }

        [Fact]
        public void A_plain_percentage_parses()
        {
            Assert.True(CpuThrottle.TryParsePercent(Args("--throttle", "33"), out int percent, out _));
            Assert.Equal(33, percent);
        }

        // Typing the sign the flag is named after should not be an error.
        [Fact]
        public void A_trailing_percent_sign_is_accepted()
        {
            Assert.True(CpuThrottle.TryParsePercent(Args("--throttle", "33%"), out int percent, out _));
            Assert.Equal(33, percent);
        }

        [Theory]
        [InlineData("0")]
        [InlineData("-5")]
        [InlineData("101")]
        [InlineData("abc")]
        public void An_out_of_range_or_unparseable_value_is_an_error(string value)
        {
            Assert.False(CpuThrottle.TryParsePercent(Args("--throttle", value), out _, out string? error));
            Assert.NotNull(error);
        }

        [Fact]
        public void A_missing_value_is_an_error()
        {
            Assert.False(CpuThrottle.TryParsePercent(Args("--throttle"), out _, out string? error));
            Assert.NotNull(error);
        }

        // 100 is one core's worth across every thread, not "unthrottled" - see §3.42.
        [Fact]
        public void A_hundred_percent_quota_is_accepted()
        {
            Assert.True(CpuThrottle.TryParsePercent(Args("--throttle", "100"), out int percent, out _));
            Assert.Equal(100, percent);
        }
    }
}
