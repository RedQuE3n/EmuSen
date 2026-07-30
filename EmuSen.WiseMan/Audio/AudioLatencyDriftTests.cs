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
            int postStallQueue = player.QueuedFrames;

            RunPaced(session, player, seconds: 3.0, clock);
            double recoveredBacklog = BacklogMs(session);

            string report =
                $"baseline={baselineBacklog:F1}ms, post-stall={postStallBacklog:F1}ms, " +
                $"after 3s of normal play={recoveredBacklog:F1}ms, " +
                $"output queue={player.QueuedFrames} frames, " +
                $"ratio={player.RateControl.LastRatio:F5}, shed={player.RateControl.SheddingEvents}";
            Console.WriteLine($"[AudioLatencyDriftTests] {report}");

            // Pump() drains the core buffer completely every call now, so the
            // core side is no longer a latency reservoir at all - all buffering
            // lives in the output queue, where rate control can steer it.
            // See EmuSen_Audio_Sync.md §1.
            Assert.True(baselineBacklog < 50.0, $"core-side backlog should stay near zero; got {report}");
            Assert.True(postStallBacklog < 50.0, $"a stall should not leave a core-side backlog; got {report}");
            Assert.True(recoveredBacklog < 50.0, $"core-side backlog should stay near zero; got {report}");

            // A one-second burst is exactly the gross-backlog case shedding
            // exists for, so it may engage here - what matters is that it
            // lets go again and the queue is actually coming back down,
            // rather than parking at an end stop. See EmuSen_Audio_Sync.md §3.1.
            Assert.False(player.RateControl.IsShedding, $"still shedding after 3s of normal play; got {report}");
            Assert.True(player.QueuedFrames < postStallQueue,
                $"output queue should be draining back toward target; got {report}");
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
