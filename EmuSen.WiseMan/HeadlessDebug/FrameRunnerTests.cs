using EmuSen.Crystal.Client;
using EmuSen.Pharaoh;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.HeadlessDebug
{
    // FrameRunner is the one frame-stepping primitive the classic loop and --commands drive, over a session of the frontend API - see EmuSen_Backend.md §14.8.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class FrameRunnerTests : IDisposable
    {
        private readonly CrystalRuntime _runtime = new();
        private readonly string _rom = Path.Combine(Path.GetTempPath(), $"wiseman_runner_{Guid.NewGuid():N}.sfc");

        public FrameRunnerTests() => File.WriteAllBytes(_rom, SyntheticRom.BuildPadReader());

        public void Dispose()
        {
            _runtime.Dispose();
            File.Delete(_rom);
        }

        private (CrystalSession Session, List<string> Log) NewSessionAndLog() => (_runtime.Open(new CrystalOpen { Path = _rom, NoBattery = true }), new List<string>());

        // A pad's latched word as the program last copied it to WRAM, after two frames so an auto-read has latched what is held.
        private static ushort Latched(FrameRunner runner, int pad = 0)
        {
            runner.RunFrames(2);
            Span<byte> word = stackalloc byte[2];
            runner.Session.ReadSpace("WRAM", (uint)(pad * 2), word);
            return (ushort)(word[0] | word[1] << 8);
        }

        [Fact]
        public void RunFrames_advances_CurrentFrame_by_the_requested_count()
        {
            var (session, log) = NewSessionAndLog();
            var runner = new FrameRunner(session, frameCap: 100, log.Add);

            runner.RunFrames(5);

            Assert.Equal(5, runner.CurrentFrame);
        }

        [Fact]
        public void RunFrames_stops_at_the_safety_cap_and_warns_with_generalized_wording()
        {
            var (session, log) = NewSessionAndLog();
            var runner = new FrameRunner(session, frameCap: 3, log.Add);

            runner.RunFrames(5);

            Assert.Equal(3, runner.CurrentFrame);
            string warning = Assert.Single(log, l => l.StartsWith("[WARN] Hit the"));
            Assert.Contains("3-frame safety cap", warning);
            // Generalized wording: waitstable's and contactsheet's own RunFrames(1) calls can hit the cap too.
            Assert.Contains("ignoring the rest of this request", warning);
            Assert.DoesNotContain("'frames' request", warning);
        }

        [Fact]
        public void WaitStable_does_not_exceed_maxFrames()
        {
            var (session, log) = NewSessionAndLog();
            var runner = new FrameRunner(session, frameCap: 1000, log.Add);

            long stepped = runner.WaitStable(maxFrames: 20, quietFrames: 5);

            Assert.True(stepped <= 20);
            Assert.True(runner.CurrentFrame <= 20);
        }

        // A blank synthetic ROM's picture never changes, so no quiet streak counts until a real change was seen: the whole budget runs.
        [Fact]
        public void WaitStable_requires_a_real_change_before_a_quiet_streak_counts_as_settled()
        {
            var (session, log) = NewSessionAndLog();
            var runner = new FrameRunner(session, frameCap: 1000, log.Add);

            long stepped = runner.WaitStable(maxFrames: 15, quietFrames: 3);

            Assert.Equal(15, stepped);
            Assert.Equal(15, runner.CurrentFrame);
        }

        [Fact]
        public void Hold_then_RunFrames_latches_the_button_pressed()
        {
            var (session, log) = NewSessionAndLog();
            var runner = new FrameRunner(session, frameCap: 100, log.Add);

            runner.Hold(PadButton.Start);

            Assert.NotEqual(0, Latched(runner) & 0x1000); // Start's bit
        }

        [Fact]
        public void Release_clears_a_previously_held_button()
        {
            var (session, log) = NewSessionAndLog();
            var runner = new FrameRunner(session, frameCap: 100, log.Add);

            runner.Hold(PadButton.Start);
            Assert.NotEqual(0, Latched(runner) & 0x1000);
            runner.Release(PadButton.Start);

            Assert.Equal(0, Latched(runner) & 0x1000);
        }

        [Fact]
        public void Tap_holds_for_the_given_duration_then_releases()
        {
            var (session, log) = NewSessionAndLog();
            var runner = new FrameRunner(session, frameCap: 100, log.Add);

            runner.Tap(PadButton.A, controller: 1, duration: 4);
            Assert.Equal(4, runner.CurrentFrame);

            // The latched word updates at the next auto-joypad read, so two more frames make the release observable.
            Assert.Equal(0, Latched(runner) & 0x0080); // A's bit, released after the tap
        }

        [Fact]
        public void OnFrameAdvanced_callback_fires_once_per_completed_frame_with_post_increment_numbering()
        {
            var (session, _) = NewSessionAndLog();
            var seen = new List<long>();
            var runner = new FrameRunner(session, frameCap: 100, _ => { }, onFrameAdvanced: seen.Add);

            runner.RunFrames(3);

            Assert.Equal(new long[] { 1, 2, 3 }, seen);
        }

        // tap names controller 1, tap2 controller 2 as before, and tap3 to tap8 the rest - see EmuSen_Input.md §8.7.
        [Theory]
        [InlineData("tap", 1)]
        [InlineData("tap2", 2)]
        [InlineData("tap4", 4)]
        [InlineData("tap8", 8)]
        [InlineData("tap9", null)]
        [InlineData("tap0", null)]
        [InlineData("tapx", null)]
        [InlineData("hold", null)]
        public void A_tap_verb_names_its_controller(string verb, int? controller) =>
            Assert.Equal(controller, CommandsScriptRunner.TapController(verb));

        // A held button on controller 2 reaches the SNES's second pad and not its first.
        [Fact]
        public void Hold_on_controller_2_reaches_the_second_pad()
        {
            var (session, log) = NewSessionAndLog();
            var runner = new FrameRunner(session, frameCap: 100, log.Add);
            runner.Hold(PadButton.Start, 2);
            Assert.Equal(0x1000, Latched(runner, pad: 1));
            Assert.Equal(0, Latched(runner));
        }
    }
}
