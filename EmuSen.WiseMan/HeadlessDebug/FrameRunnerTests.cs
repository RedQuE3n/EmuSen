using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.HeadlessDebug;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.HeadlessDebug
{
    // FrameRunner is the one shared frame-stepping primitive the classic
    // loop and --commands mode both drive - see FrameRunner.cs's own
    // comment for why it replaced two separately-drifting implementations.
    public class FrameRunnerTests
    {
        private static (EmuSen.Cores.Nintendo.Venus.VenusCore Core, List<string> Log) NewCoreAndLog()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return (core, new List<string>());
        }

        [Fact]
        public void RunFrames_advances_CurrentFrame_by_the_requested_count()
        {
            var (core, log) = NewCoreAndLog();
            var runner = new FrameRunner(core, frameCap: 100, log.Add);

            runner.RunFrames(5);

            Assert.Equal(5, runner.CurrentFrame);
        }

        [Fact]
        public void RunFrames_stops_at_the_safety_cap_and_warns_with_generalized_wording()
        {
            var (core, log) = NewCoreAndLog();
            var runner = new FrameRunner(core, frameCap: 3, log.Add);

            runner.RunFrames(5);

            Assert.Equal(3, runner.CurrentFrame);
            string warning = Assert.Single(log, l => l.StartsWith("[WARN] Hit the"));
            Assert.Contains("3-frame safety cap", warning);
            // Generalized wording, not the old "'frames' request" text -
            // waitstable/contactsheet's own inner RunFrames(1) calls can
            // hit this cap too, not just the `frames` verb.
            Assert.Contains("ignoring the rest of this request", warning);
            Assert.DoesNotContain("'frames' request", warning);
        }

        [Fact]
        public void WaitStable_does_not_exceed_maxFrames()
        {
            var (core, log) = NewCoreAndLog();
            var runner = new FrameRunner(core, frameCap: 1000, log.Add);

            long stepped = runner.WaitStable(maxFrames: 20, quietFrames: 5);

            Assert.True(stepped <= 20);
            Assert.True(runner.CurrentFrame <= 20);
        }

        [Fact]
        public void WaitStable_requires_a_real_change_before_a_quiet_streak_counts_as_settled()
        {
            // A blank synthetic ROM's framebuffer never changes frame to
            // frame, so "sawChange" should never flip true - meaning this
            // must run the full maxFrames instead of exiting early on the
            // very first quiet streak it sees. This is the exact bug
            // WaitStable's own comment documents having caught once: a
            // naive "N identical frames in a row" check reports "stable"
            // instantly for a scene that was already static from frame
            // zero, indistinguishable from one that already finished
            // reacting to input.
            var (core, log) = NewCoreAndLog();
            var runner = new FrameRunner(core, frameCap: 1000, log.Add);

            long stepped = runner.WaitStable(maxFrames: 15, quietFrames: 3);

            Assert.Equal(15, stepped);
            Assert.Equal(15, runner.CurrentFrame);
        }

        [Fact]
        public void Hold_then_RunFrames_latches_the_button_pressed()
        {
            var (core, log) = NewCoreAndLog();
            var runner = new FrameRunner(core, frameCap: 100, log.Add);

            runner.Hold(SnesButton.Start);
            runner.RunFrames(1);

            ushort latched = (ushort)((core.Bus!.Input.ReadJoy1High() << 8) | core.Bus.Input.ReadJoy1Low());
            Assert.NotEqual(0, latched & 0x1000); // Start's bit
        }

        [Fact]
        public void Release_clears_a_previously_held_button()
        {
            var (core, log) = NewCoreAndLog();
            var runner = new FrameRunner(core, frameCap: 100, log.Add);

            runner.Hold(SnesButton.Start);
            runner.RunFrames(1);
            runner.Release(SnesButton.Start);
            runner.RunFrames(1);

            ushort latched = (ushort)((core.Bus!.Input.ReadJoy1High() << 8) | core.Bus.Input.ReadJoy1Low());
            Assert.Equal(0, latched & 0x1000);
        }

        [Fact]
        public void Tap_holds_for_the_given_duration_then_releases()
        {
            var (core, log) = NewCoreAndLog();
            var runner = new FrameRunner(core, frameCap: 100, log.Add);

            runner.Tap(SnesButton.A, controller: 1, duration: 4);
            Assert.Equal(4, runner.CurrentFrame);

            // Release() updates the live register immediately, but the
            // *latched* register (what ReadJoy1Low/High and a real game's
            // $4218/$4219 reads actually see) only updates on the next
            // auto-joypad-read - i.e. the next frame's vblank - matching
            // real hardware. One more frame with nothing held makes that
            // release observable.
            runner.RunFrames(1);
            ushort latched = (ushort)((core.Bus!.Input.ReadJoy1High() << 8) | core.Bus.Input.ReadJoy1Low());
            Assert.Equal(0, latched & 0x0080); // A's bit, released after the tap
        }

        [Fact]
        public void OnFrameAdvanced_callback_fires_once_per_completed_frame_with_post_increment_numbering()
        {
            var (core, _) = NewCoreAndLog();
            var seen = new List<long>();
            var runner = new FrameRunner(core, frameCap: 100, _ => { }, onFrameAdvanced: seen.Add);

            runner.RunFrames(3);

            Assert.Equal(new long[] { 1, 2, 3 }, seen);
        }
    }
}
