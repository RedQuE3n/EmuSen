using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Pharaoh.Cli;

namespace EmuSen.WiseMan.HeadlessDebug
{
    // HeadlessDebugOptions.Parse is argv parsing pulled out into a plain
    // function of a string array - genuinely unit-testable for the first
    // time, instead of only exercisable by actually launching the process.
    public class HeadlessDebugOptionsTests
    {
        [Fact]
        public void Too_few_positional_args_returns_a_usage_error()
        {
            var (options, warnings, error) = HeadlessDebugOptions.Parse(new[] { "rom.sfc" });

            Assert.Null(options);
            Assert.NotNull(error);
            Assert.Contains("Usage:", error);
        }

        [Fact]
        public void Non_numeric_frame_count_returns_an_error()
        {
            var (options, warnings, error) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "notanumber" });

            Assert.Null(options);
            Assert.Contains("Invalid frame count", error);
        }

        [Fact]
        public void Zero_or_negative_frame_count_returns_an_error()
        {
            var (options, _, error) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "0" });
            Assert.NotNull(error);

            var (options2, _, error2) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "-5" });
            Assert.NotNull(error2);
        }

        [Fact]
        public void Valid_positional_args_parse_into_RomPath_and_FrameCount()
        {
            var (options, warnings, error) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600" });

            Assert.Null(error);
            Assert.NotNull(options);
            Assert.Equal("rom.sfc", options!.RomPath);
            Assert.Equal(600, options.FrameCount);
            Assert.Empty(warnings);
        }

        [Fact]
        public void Tap_without_duration_defaults_to_4_frames_on_controller_1()
        {
            var (options, _, _) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--tap", "10:Start" });

            var tap = Assert.Single(options!.Taps);
            Assert.Equal(10, tap.Start);
            Assert.Equal(14, tap.End);
            Assert.Equal(SnesButton.Start, tap.Button);
            Assert.Equal(1, tap.Controller);
        }

        [Fact]
        public void Tap_with_explicit_duration_is_respected()
        {
            var (options, _, _) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--tap", "10:A:20" });

            var tap = Assert.Single(options!.Taps);
            Assert.Equal(10, tap.Start);
            Assert.Equal(30, tap.End);
        }

        [Fact]
        public void Tap2_targets_controller_2()
        {
            var (options, _, _) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--tap2", "10:B" });

            var tap = Assert.Single(options!.Taps);
            Assert.Equal(2, tap.Controller);
        }

        [Fact]
        public void Watch_specs_are_captured_verbatim_including_malformed_ones()
        {
            // Validating the space:addr:len[:kind] shape happens later,
            // where the watch is actually registered against a loaded
            // core (needs Emit/--out to exist) - Parse just collects the
            // raw strings, same as before.
            var (options, _, _) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--watch", "WRAM:0x8000:0x100", "--watch", "malformed" });

            Assert.Equal(new[] { "WRAM:0x8000:0x100", "malformed" }, options!.ExtraWatches);
        }

        [Fact]
        public void Unknown_flag_produces_a_warning_instead_of_a_console_only_message()
        {
            var (options, warnings, error) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--flag", "ThisFlagDoesNotExist" });

            Assert.Null(error);
            Assert.Contains(options!.FlagsToEnable, f => f == "ThisFlagDoesNotExist");
            string warning = Assert.Single(warnings);
            Assert.Contains("ThisFlagDoesNotExist", warning);
            Assert.Contains("[WARN]", warning);
        }

        [Fact]
        public void Known_flag_produces_no_warning()
        {
            var (_, warnings, _) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--flag", "MasterLoggingEnabled" });

            Assert.Empty(warnings);
        }

        [Fact]
        public void Screenshot_spec_splits_frame_and_path_on_the_first_colon_only()
        {
            var (options, _, _) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--screenshot", "5:C:/some/path.bmp" });

            var shot = Assert.Single(options!.Screenshots);
            Assert.Equal(5, shot.Frame);
            Assert.Equal("C:/some/path.bmp", shot.Path);
        }

        [Fact]
        public void Verbose_flag_is_captured()
        {
            var (options, _, _) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--verbose" });
            Assert.True(options!.Verbose);
        }

        [Fact]
        public void CpuLog_range_is_parsed()
        {
            var (options, _, _) = HeadlessDebugOptions.Parse(new[] { "rom.sfc", "600", "--cpulog", "100:200" });
            Assert.Equal(100, options!.CpuLogStart);
            Assert.Equal(200, options.CpuLogEnd);
        }
    }
}
