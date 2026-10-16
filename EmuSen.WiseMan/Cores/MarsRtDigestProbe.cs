using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT through ICore, a digest of picture, sound and state each frame in every configuration, for comparing two builds - see Mars_Native.md §7.1.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsRtDigestProbe : IDisposable
    {
        public const string OutVariable = "EMUSEN_MARSRT_DIGEST_OUT";
        public const string GamesVariable = "EMUSEN_MARSRT_DIGEST_GAMES";
        public const string GpuVariable = "EMUSEN_MARSRT_DIGEST_GPU";

        private static readonly string Home = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "probe", "mars", "v1", "digest-home");

        public MarsRtDigestProbe() => DataStore.OverrideDirectory = Home;

        public void Dispose() => DataStore.OverrideDirectory = null;

        private static readonly (string Name, (string Key, string Value)[] Settings)[] Configurations =
        {
            ("default", new[] { ("RdpWorkers", "4") }),
            ("interpreter-one-thread", new[] { ("Recompiler", "false"), ("ThreadedRdp", "false"), ("DeferredPresentation", "false") }),
            ("threaded-now", new[] { ("RdpWorkers", "2"), ("DeferredPresentation", "false"), ("SkipRepeatedScans", "false") }),
            ("scale2-aa", new[] { ("RdpWorkers", "4"), ("RenderScale", "2"), ("Antialiasing", "2x") }),
        };

        [Fact]
        public void MarsRt_writes_its_digests()
        {
            string? output = Environment.GetEnvironmentVariable(OutVariable);
            string? games = Environment.GetEnvironmentVariable(GamesVariable);
            if (string.IsNullOrWhiteSpace(output) || string.IsNullOrWhiteSpace(games)) return;
            Assert.True(MarsRtCore.Available, EmuSen.Cores.Nintendo.Mars.Native.MarsNative.Report);
            CoreOptions.BatteryRamDisabled = true;
            Directory.CreateDirectory(output);
            string scratch = Path.Combine(output, "states");
            Directory.CreateDirectory(scratch);
            bool gpu = Environment.GetEnvironmentVariable(GpuVariable) == "1";

            foreach (string game in new[] { "sm64", "oot" })
            {
                string rom = Path.Combine(games, game + ".z64");
                foreach (var (name, settings) in gpu ? new[] { ("gpu-scale2", new[] { ("RdpWorkers", "4"), ("RenderScale", "2"), ("Gpu", "true") }) } : Configurations)
                {
                    using var run = new Run(Path.Combine(output, $"{game}-{name}-boot.txt"), settings);
                    run.Load(rom);
                    run.Play(0, gpu ? 300 : 600, held: 290, until: 310, saveAt: name == "default" ? 300 : -1, scratch, game);
                }
                if (gpu) continue;

                // A state and a snapshot saved while A was held load into a machine that presses nothing, as R3 of EmuSen_NativeCores.md §8.2 asks.
                foreach (string kind in new[] { "state", "snapshot" })
                {
                    using var run = new Run(Path.Combine(output, $"{game}-default-from-held-{kind}.txt"), Configurations[0].Settings);
                    run.Load(rom);
                    run.LoadState(Path.Combine(scratch, $"{game}-held.{kind}"));
                    run.Play(300, 120, held: -1, until: -1, saveAt: -1, scratch, game);
                }

                // The measurement state, an 8 MB machine's, in each configuration.
                foreach (var (name, settings) in Configurations)
                {
                    using var run = new Run(Path.Combine(output, $"{game}-{name}-from-state.txt"), settings);
                    run.Load(rom);
                    run.LoadState(Path.Combine(games, game + ".state"));
                    run.Play(0, 300, held: 100, until: 140, saveAt: -1, scratch, game);
                }

                // The other RDRAM size: a 4 MB machine's state into an 8 MB machine, and the 8 MB state into a stock machine.
                using (var small = new Run(Path.Combine(output, $"{game}-stock-boot.txt"), Configurations[0].Settings, expansionPak: false))
                {
                    small.Load(rom);
                    small.Play(0, 200, held: -1, until: -1, saveAt: 200, scratch, game + "-4mb");
                }
                using (var run = new Run(Path.Combine(output, $"{game}-default-from-4mb-state.txt"), Configurations[0].Settings))
                {
                    run.Load(rom);
                    run.LoadState(Path.Combine(scratch, $"{game}-4mb-held.state"));
                    run.Play(200, 120, held: -1, until: -1, saveAt: -1, scratch, game);
                }
                using (var run = new Run(Path.Combine(output, $"{game}-stock-from-8mb-state.txt"), Configurations[0].Settings, expansionPak: false))
                {
                    run.Load(rom);
                    run.LoadState(Path.Combine(games, game + ".state"));
                    run.Play(0, 120, held: -1, until: -1, saveAt: -1, scratch, game);
                }
            }
        }

        private sealed class Run : IDisposable
        {
            private readonly StreamWriter _out;
            private readonly MarsRtCore _core;
            private long _serial;

            public Run(string path, (string Key, string Value)[] settings, bool expansionPak = true)
            {
                _out = new StreamWriter(path);
                _core = new MarsRtCore(expansionPak, batteryRamDisabled: true);
                foreach (var (key, value) in settings) ((ICoreSettings)_core).Set(key, value);
            }

            public void Load(string rom)
            {
                _core.LoadRom(rom);
                Line("load", -1);
            }

            public void LoadState(string path)
            {
                _core.LoadState(path);
                Line("state", -1);
            }

            // Start pulsed every 150 frames, the stick swept, A held over [held, until), rendering skipped for frames 200 to 239.
            public void Play(int from, int frames, int held, int until, int saveAt, string scratch, string game)
            {
                for (int i = 0; i < frames; i++)
                {
                    int f = from + i;
                    _core.SetButton(0, PadButton.Start, f % 150 >= 145);
                    _core.SetButton(0, PadButton.A, f >= held && f < until);
                    _core.SetAxis(0, PadAxis.LeftX, Math.Sin(f / 20.0));
                    _core.SetAxis(0, PadAxis.LeftY, f % 90 < 30 ? -0.6 : 0);
                    _core.SkipRendering = i >= 200 && i < 240;
                    _core.RunFrame();
                    Line("frame", f);
                    if (i + 1 == saveAt)
                    {
                        using (var file = File.Create(Path.Combine(scratch, $"{game}-held.state"))) _core.SaveState(file);
                        using (var file = File.Create(Path.Combine(scratch, $"{game}-held.snapshot"))) _core.SaveSnapshot(file);
                    }
                }
                _core.SkipRendering = false;
                Line("end", from + frames, state: true);
                _out.WriteLine($"gpu {_core.GpuReport}");
            }

            private void Line(string what, int frame, bool state = false)
            {
                byte[] picture = _core.GetFrameBufferRgba();
                string pixels = Convert.ToHexString(SHA256.HashData(picture))[..16];
                _core.ReturnFrameBuffer(picture);
                short[] samples = _core.DequeueAudioSamples(int.MaxValue);
                string sound = Convert.ToHexString(SHA256.HashData(MemoryMarshal.AsBytes(samples.AsSpan())))[..16];
                long serial = _core.FrameSerial;
                bool moved = serial != _serial;
                _serial = serial;
                string text = $"{what} {frame} picture {pixels} {_core.ScreenWidth}x{_core.ScreenHeight} rows {_core.RowRepeat} lines {_core.DisplayLines} aspect {_core.DisplayAspect:F4} serial-moved {moved} sound {samples.Length} {sound} rate {_core.AudioSampleRate} hz {_core.FrameRateHz:F6} frames {_core.TotalFrames}";
                if (state || frame % 50 == 0)
                {
                    using var stream = new MemoryStream();
                    _core.SaveState(stream);
                    text += $" state {Convert.ToHexString(SHA256.HashData(stream.ToArray()))[..16]} {stream.Length}";
                }
                _out.WriteLine(text);
            }

            public void Dispose()
            {
                _core.Dispose();
                _out.Dispose();
            }
        }
    }
}
