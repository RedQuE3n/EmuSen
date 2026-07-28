using System.Diagnostics;
using EmuSen.Common;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.Mistress.Audio;

namespace EmuSen.WiseMan.Audio
{
    // Audio-behind-video regression coverage - see EmuSen_Settings_Reference.md §2.
    public class AudioLatencyDriftTests
    {
        static AudioLatencyDriftTests()
        {
            NativeEnvironment.Set("SDL_AUDIODRIVER", "dummy");
        }

        private const int FramesPerSecond = 60;
        private static readonly TimeSpan FrameInterval = TimeSpan.FromSeconds(1.0 / FramesPerSecond);

        // Real ROM, not SyntheticRom's blank one - see .gitignore.
        private static string RomPath => Path.Combine(DianaOSSandbox.UsrHomeDirectory, "Games", "SNES", "SMW.smc");

        [Fact]
        public void Audio_backlog_stays_bounded_after_a_single_stall()
        {
            if (!File.Exists(RomPath))
            {
                Console.WriteLine($"[SKIP] No ROM at {RomPath} - this measurement needs a real SNES ROM there.");
                return;
            }

            var session = new EmulatorSession();
            session.LoadRom(RomPath);

            using var player = new AudioPlayer();
            if (!player.IsAvailable)
            {
                Console.WriteLine("[SKIP] SDL dummy audio device did not open in this environment.");
                return;
            }

            var clock = Stopwatch.StartNew();

            RunPaced(session, player, seconds: 2.0, clock);
            double baselineBacklog = BacklogMs(session);

            // Simulates MainWindow.axaml.cs's EmulationLoop "fell behind -
            // resync to now" branch: 1 simulated second of RunFrame()+Pump()
            // with no sleeping between calls.
            for (int i = 0; i < FramesPerSecond; i++)
            {
                session.RunFrame();
                player.Pump(session);
            }
            double postStallBacklog = BacklogMs(session);

            RunPaced(session, player, seconds: 3.0, clock);
            double recoveredBacklog = BacklogMs(session);

            string report =
                $"baseline={baselineBacklog:F1}ms, post-stall={postStallBacklog:F1}ms, " +
                $"after 3s of normal play={recoveredBacklog:F1}ms";
            Console.WriteLine($"[AudioLatencyDriftTests] {report}");

            Assert.True(postStallBacklog < 400.0,
                $"expected the active resync to cap the stall's backlog well under the old ~1000ms ceiling; got {report}");
            // Doesn't return all the way to the pristine baseline - a
            // separate, expected ~256ms floor from AudioPlayer.Pump()'s own
            // SDL-queue throttle, not the ratchet this fix targets. What
            // matters here is staying well clear of the old unbounded,
            // ever-growing behavior (which reached 600-1000+ms).
            Assert.True(recoveredBacklog < 400.0,
                $"expected backlog to stay well under the old ~1000ms ceiling after 3s of normal play; got {report}");
        }

        private static double BacklogMs(EmulatorSession session)
        {
            int frames = session.Bus.Spc700.Dsp.AudioBuffer.Count / 2;
            return frames * 1000.0 / session.AudioSampleRate;
        }

        private static void RunPaced(EmulatorSession session, AudioPlayer player, double seconds, Stopwatch clock)
        {
            TimeSpan end = clock.Elapsed + TimeSpan.FromSeconds(seconds);
            TimeSpan nextTick = clock.Elapsed;
            while (clock.Elapsed < end)
            {
                nextTick += FrameInterval;
                session.RunFrame();
                player.Pump(session);
                TimeSpan remaining = nextTick - clock.Elapsed;
                if (remaining > TimeSpan.Zero) Thread.Sleep(remaining);
                else nextTick = clock.Elapsed; // fell behind - resync instead of bursting to catch up, matching EmulationLoop
            }
        }
    }
}
