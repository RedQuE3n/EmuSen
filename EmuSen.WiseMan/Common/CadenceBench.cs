using System;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Text;
using System.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Endymion;
using EmuSen.Mistress;
using EmuSen.Serenity;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Common
{
    // A real core paced as Mistress paces it, presented by a modelled vblank-locked presenter, written in PresentationTrace's format - see EmuSen_Settings_Reference.md §4.87.1.
    public static class CadenceBench
    {
        public sealed record Result(string StateSha, int MinQueuedFrames, double MinRatio, double MaxRatio, int Shed, double LastQueueMs, string Decision);

        public sealed class Options
        {
            public required string Rom;
            public string? Engine;
            public double DisplayHz = 60.0;
            public double Seconds = 30;
            public bool SyncToDisplay;
            public int Frames = int.MaxValue;
            public string? TracePath;
            public bool Audio = true;
            public double ContentHzOverride;
            public TextWriter? Log;
        }

        public static Result Run(Options o)
        {
            var session = new EmulatorSession { Engine = o.Engine };
            session.LoadRom(o.Rom);

            AudioPlayer? player = null;
            if (o.Audio)
            {
                SDL.SetHint(SDL.Hints.AudioDriver, "dummy");
                player = new AudioPlayer();
                if (!player.IsAvailable) { player.Dispose(); player = null; }
            }

            var display = new DisplayClock();
            long frequency = Stopwatch.Frequency;
            double period = frequency / o.DisplayHz;
            long t0 = Stopwatch.GetTimestamp() + frequency / 10;
            long newest = 0, lastShown = -1;
            bool stop = false;
            var draws = new StringBuilder();
            var frames = new StringBuilder();

            // The presenter: at each vblank the newest picture offered half a millisecond before it is the one shown.
            var presenter = new Thread(() =>
            {
                long latch = frequency / 2000;
                for (long n = 1; !Volatile.Read(ref stop); n++)
                {
                    long vblank = t0 + (long)Math.Round(n * period);
                    while (Stopwatch.GetTimestamp() < vblank - latch) Thread.SpinWait(50);
                    long seq = Volatile.Read(ref newest);
                    long previous = t0 + (long)Math.Round((n - 1) * period);
                    long ust = previous / (frequency / 1_000_000);
                    display.ObserveVblank(ust, n - 1, Stopwatch.GetTimestamp());
                    if (seq != 0) draws.Append(CultureInfo.InvariantCulture, $"D,{vblank - latch},{seq},{(seq != lastShown ? 1 : 0)},1,{ust},{n - 1},{n - 1}\n");
                    lastShown = seq;
                }
            }) { IsBackground = true, Name = "CadenceBench-Presenter", Priority = ThreadPriority.AboveNormal };
            presenter.Start();
            while (Stopwatch.GetTimestamp() < t0) Thread.SpinWait(50);

            var speed = new SpeedController();
            Stopwatch clock = Stopwatch.StartNew();
            var scheduler = new FrameScheduler(display.Current, Stopwatch.GetTimestamp()) { SyncToDisplay = o.SyncToDisplay };
            TimeSpan nextTick = clock.Elapsed;
            int minQueued = int.MaxValue, shedAtStart = 0;
            double minRatio = double.MaxValue, maxRatio = double.MinValue, lastQueueMs = 0;
            TimeSpan nextLog = TimeSpan.FromSeconds(1);
            int frame = 0;
            while (clock.Elapsed.TotalSeconds < o.Seconds && frame < o.Frames)
            {
                double contentHz = o.ContentHzOverride > 0 ? o.ContentHzOverride : session.FrameRateHz;
                nextTick = scheduler.Next(nextTick, contentHz, speed);
                long start = Stopwatch.GetTimestamp();
                session.RunFrame();
                frame++;
                short[] samples = session.DequeueAudioSamples(int.MaxValue);
                if (player is not null)
                {
                    player.RateControl.NominalRatio = scheduler.Decision.AudioRatio;
                    int queued = player.QueuedFrames;
                    if (clock.Elapsed.TotalSeconds > 3) { minQueued = Math.Min(minQueued, queued); minRatio = Math.Min(minRatio, player.RateControl.LastRatio); maxRatio = Math.Max(maxRatio, player.RateControl.LastRatio); }
                    else shedAtStart = player.RateControl.SheddingEvents;
                    player.Submit(samples, session.AudioSampleRate);
                    lastQueueMs = queued * 1000.0 / Math.Max(1, player.SampleRate);
                }
                session.GetFrameBufferRgba();
                Volatile.Write(ref newest, session.TotalFrames);
                long offered = Stopwatch.GetTimestamp();
                scheduler.Completed(TimeSpan.FromTicks((offered - start) * TimeSpan.TicksPerSecond / frequency));
                frames.Append(CultureInfo.InvariantCulture, $"F,{session.TotalFrames},{start},{offered},1\n");

                if (o.Log is not null && clock.Elapsed >= nextLog)
                {
                    nextLog += TimeSpan.FromSeconds(10);
                    o.Log.WriteLine(FormattableString.Invariant($"t={clock.Elapsed.TotalSeconds:F0}s {scheduler.Decision.Describe()} queue={lastQueueMs:F1}ms ratio={player?.RateControl.LastRatio:F5} shed={player?.RateControl.SheddingEvents}"));
                }

                if (nextTick - clock.Elapsed > TimeSpan.Zero) { while (clock.Elapsed < nextTick) Thread.SpinWait(100); }
                else nextTick = FramePacer.Settle(nextTick, clock.Elapsed, scheduler.Interval(contentHz, speed));
            }
            Volatile.Write(ref stop, true);
            presenter.Join();

            using var state = new MemoryStream();
            session.Core!.SaveState(state);
            string sha = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(state.ToArray()));
            int shed = (player?.RateControl.SheddingEvents ?? 0) - shedAtStart;
            player?.Dispose();

            if (o.TracePath is not null)
            {
                File.WriteAllText(o.TracePath, FormattableString.Invariant($"# freq {frequency} bench {Path.GetFileName(o.Rom)} {o.Engine} display {o.DisplayHz} Hz sync={o.SyncToDisplay} final: {scheduler.Decision.Describe()}\n") + frames + draws);
            }
            return new Result(sha, minQueued, minRatio, maxRatio, shed, lastQueueMs, scheduler.Decision.Describe());
        }
    }

    // Runs the bench when EMUSEN_CADENCE_ROM is set; otherwise does nothing.
    [Collection(TestCollections.ProcessGlobals)]
    public class CadenceBenchProbe
    {
        [Fact]
        public void Measure_cadence()
        {
            string? rom = Environment.GetEnvironmentVariable("EMUSEN_CADENCE_ROM");
            if (string.IsNullOrEmpty(rom)) return;
            string Env(string name, string fallback) => Environment.GetEnvironmentVariable(name) is { Length: > 0 } v ? v : fallback;
            CoreOptions.BatteryRamDisabled = true;
            var result = CadenceBench.Run(new CadenceBench.Options
            {
                Rom = rom,
                Engine = Environment.GetEnvironmentVariable("EMUSEN_CADENCE_ENGINE"),
                DisplayHz = double.Parse(Env("EMUSEN_CADENCE_HZ", "60"), CultureInfo.InvariantCulture),
                Seconds = double.Parse(Env("EMUSEN_CADENCE_SECONDS", "30"), CultureInfo.InvariantCulture),
                SyncToDisplay = Env("EMUSEN_CADENCE_MODE", "content") == "locked",
                TracePath = Env("EMUSEN_CADENCE_TRACE", Path.Combine(Path.GetTempPath(), "cadence.trace")),
                Log = Console.Out,
            });
            Console.WriteLine(FormattableString.Invariant($"[cadence] {result.Decision} minQueue={result.MinQueuedFrames} ratio=[{result.MinRatio:F5},{result.MaxRatio:F5}] shed={result.Shed} lastQueue={result.LastQueueMs:F1}ms sha={result.StateSha[..16]}"));
        }
    }
}
