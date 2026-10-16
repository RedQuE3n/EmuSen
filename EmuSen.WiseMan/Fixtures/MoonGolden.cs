using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia.Input;
using EmuSen.Pharaoh;

namespace EmuSen.WiseMan.Fixtures
{
    // One golden run of MoonRT: a ROM, an input script, resets, and frames or instructions; a corpus run follows blargg's protocol - see Moon_Native.md §9.2.
    public sealed record MoonGoldenRun(string Name, Func<byte[]> Rom, string Input = "none", bool SkipRendering = false, int Frames = 0, int Steps = 0, bool Corpus = false, int[]? Resets = null)
    {
        public override string ToString() => Name;
    }

    // Recording and grading MoonRT's golden runs - see Moon_Native.md §9.2.
    public static class MoonGolden
    {
        // Set to 1 to write traces instead of grading them; the reason variable must say why, and is written into each trace that changes.
        public const string RecordVariable = "EMUSEN_MOONRT_GOLDEN_RECORD";
        public const string ReasonVariable = "EMUSEN_MOONRT_GOLDEN_REASON";

        // Where the full states every IntervalFrames frames go, outside the tree; ~/.cache/emusen/moonrt-goldens by default.
        public const string StatesVariable = "EMUSEN_MOONRT_GOLDEN_STATES";

        public const int IntervalFrames = 300;
        public const int StepRow = 100;
        public const int ChainRow = 60;
        public const int CorpusFrames = 2400;

        public const string Heading = "A golden trace of MoonRT, recorded from MoonRT itself; re-recorded only deliberately, with the reason below - see Moon_Native.md §9.2.";

        public static string Directory([CallerFilePath] string here = "") =>
            Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(here)!)!, "Cores", "MoonGoldens");

        public static string StateCache => Environment.GetEnvironmentVariable(StatesVariable) is { Length: > 0 } dir ? dir
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "moonrt-goldens");

        public static bool Recording => Environment.GetEnvironmentVariable(RecordVariable) == "1";

        public static string Reason => Environment.GetEnvironmentVariable(ReasonVariable) is { Length: > 0 } r ? r
            : throw new InvalidOperationException($"{RecordVariable}=1 needs {ReasonVariable} to say why the goldens are re-recorded.");

        public static string TracePath(string name) => Path.Combine(Directory(), name + ".trace");

        // Bit n is the NES pad's button n: A, B, Select, Start, Up, Down, Left, Right.
        public static readonly PadButton[] MaskOrder = { PadButton.A, PadButton.B, PadButton.Select, PadButton.Start, PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right };

        // Both pads' masks on a frame numbered from 0: the bench's Start and A on pad 1, or both pads changed every seventh frame.
        public static (uint One, uint Two) ScriptMask(string input, int frame) => input switch
        {
            "none" => (0, 0),
            "bench" => ((frame % 90) switch { < 5 => 1u << 3, >= 45 and < 50 => 1u << 0, _ => 0u }, 0),
            "pads" => (((uint)(frame / 7 * 7) * 37) & 0xFF, ((uint)(frame / 7 * 7) * 91) & 0xFF),
            _ => throw new ArgumentException($"no input script named {input}"),
        };

        // The script's fingerprint over the frames a run takes, so a reader can check its copy.
        public static string ScriptName(string input, int frames)
        {
            if (input == "none") return input;
            var masks = new byte[frames * 2];
            for (int f = 0; f < frames; f++) (masks[2 * f], masks[2 * f + 1]) = ((byte)ScriptMask(input, f).One, (byte)ScriptMask(input, f).Two);
            return $"{input} {GoldenTrace.Hash(masks)}";
        }

        // setup runs once the ROM is loaded and frame after each frame is graded, for a test that looks at more than the trace.
        public static GoldenTrace Run(MoonGoldenRun run, byte[]? resumeFrom = null, int resumeAt = 0, Action<int, byte[]>? interval = null, Action<MoonRtCore>? setup = null, Action<int, MoonRtCore>? frame = null)
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            byte[] rom = run.Rom();
            var trace = new GoldenTrace
            {
                Heading = Heading,
                Name = run.Name,
                Rom = Convert.ToHexString(SHA256.HashData(rom)),
                Rendering = run.SkipRendering ? "skipped" : "on",
                RecordedBy = "MoonRT",
                Unit = run.Steps > 0 ? "step" : run.Corpus ? "chain" : "frame",
            };
            if (run.Steps > 0) Steps(rom, run, trace);
            else Frames(rom, run, trace, resumeFrom, resumeAt, interval, setup, frame);
            return trace;
        }

        private static string RomFile(byte[] rom)
        {
            string dir = Path.Combine(Path.GetTempPath(), "emusen-moonrt-goldens");
            System.IO.Directory.CreateDirectory(dir);
            string path = Path.Combine(dir, Convert.ToHexString(SHA256.HashData(rom))[..24] + ".nes");
            if (!File.Exists(path)) File.WriteAllBytes(path, rom);
            return path;
        }

        private static void Frames(byte[] rom, MoonGoldenRun run, GoldenTrace trace, byte[]? resumeFrom, int resumeAt, Action<int, byte[]>? interval, Action<MoonRtCore>? setup, Action<int, MoonRtCore>? frame)
        {
            trace.Input = ScriptName(run.Input, run.Frames);
            CoreOptions.BatteryRamDisabled = true;
            using var core = new MoonRtCore();
            try { core.LoadRom(RomFile(rom)); }
            catch (Exception e) when (e is NotSupportedException or InvalidDataException)
            {
                trace.Transcript["verdict"] = $"Unsupported {e.GetType().Name}";
                return;
            }
            core.SkipRendering = run.SkipRendering;
            setup?.Invoke(core);
            if (resumeFrom is not null) core.LoadState(new MemoryStream(resumeFrom, writable: false));
            var protocol = run.Corpus ? new NesTestRomProtocol(a => core.ReadSpace("PRGRAM", a), core.Reset) : null;
            var chain = new byte[8];
            var link = new byte[24];
            (uint, uint) held = (uint.MaxValue, uint.MaxValue);
            int done = resumeAt;
            for (int f = resumeAt; f < run.Frames; f++)
            {
                if (run.Resets?.Contains(f) == true) core.Reset();
                var mask = ScriptMask(run.Input, f);
                if (mask != held)
                {
                    for (int b = 0; b < MaskOrder.Length; b++)
                    {
                        core.SetButton(0, MaskOrder[b], (mask.Item1 & (1u << b)) != 0);
                        core.SetButton(1, MaskOrder[b], (mask.Item2 & (1u << b)) != 0);
                    }
                    held = mask;
                }
                try { core.RunFrame(); }
                catch (Exception e)
                {
                    trace.Transcript["verdict"] = $"Error {f + 1} {e.GetType().Name}";
                    break;
                }
                done = f + 1;
                short[] samples = core.DequeueAudioSamples(int.MaxValue);
                byte[] state = State(core);
                string s = GoldenTrace.Hash(state), p = GoldenTrace.Hash(core.GetFrameBufferRgba()), a = GoldenTrace.Hash(MemoryMarshal.AsBytes(samples.AsSpan()));
                if ((f + 1) % IntervalFrames == 0) interval?.Invoke(f + 1, state);
                frame?.Invoke(f + 1, core);
                if (!run.Corpus) trace.Rows.Add(new[] { (f + 1).ToString(CultureInfo.InvariantCulture), s, p, a });
                else
                {
                    Convert.FromHexString(s).CopyTo(link, 0);
                    Convert.FromHexString(p).CopyTo(link, 8);
                    Convert.FromHexString(a).CopyTo(link, 16);
                    GoldenTrace.Fold(chain, SHA256.HashData(link));
                    if ((f + 1) % ChainRow == 0) trace.Rows.Add(new[] { (f + 1).ToString(CultureInfo.InvariantCulture), Convert.ToHexString(chain) });
                    int status = protocol!.AfterFrame();
                    if (status == -2) { trace.Transcript["verdict"] = $"NoResult {f + 1}"; break; }
                    if (status >= 0)
                    {
                        trace.Transcript["verdict"] = $"{(status == 0 ? "Passed" : $"Failed({status})")} {f + 1}";
                        break;
                    }
                }
            }
            if (protocol is null) return;
            if (done > 0 && (trace.Rows.Count == 0 || trace.Rows[^1][0] != done.ToString(CultureInfo.InvariantCulture)))
                trace.Rows.Add(new[] { done.ToString(CultureInfo.InvariantCulture), Convert.ToHexString(chain) });
            trace.Transcript.TryAdd("verdict", $"NoResult {run.Frames}");
            trace.Transcript["resets"] = protocol.Resets.ToString(CultureInfo.InvariantCulture);
            if (protocol.SignatureSeen) trace.Transcript["text"] = protocol.Text().Replace("\\", "\\\\").Replace("\n", "\\n");
        }

        public static byte[] State(ICore core)
        {
            using var ms = new MemoryStream();
            core.SaveState(ms);
            return ms.ToArray();
        }

        // Each instruction's state hashed into a chain, written every StepRow instructions and at the end; a refusal ends the run.
        private static void Steps(byte[] rom, MoonGoldenRun run, GoldenTrace trace)
        {
            trace.Input = "none";
            using var machine = new MoonMachine(rom);
            machine.SetOptions(run.SkipRendering);
            var chain = new byte[8];
            int done = 0;
            for (int i = 1; i <= run.Steps; i++)
            {
                int cycles = machine.Step();
                if (cycles < 0)
                {
                    trace.Transcript["stop"] = $"{i} {MoonMachine.Describe(cycles)}";
                    break;
                }
                GoldenTrace.Fold(chain, SHA256.HashData(machine.Save()));
                done = i;
                if (i % StepRow == 0) trace.Rows.Add(new[] { i.ToString(CultureInfo.InvariantCulture), Convert.ToHexString(chain) });
            }
            if (done % StepRow != 0) trace.Rows.Add(new[] { done.ToString(CultureInfo.InvariantCulture), Convert.ToHexString(chain) });
        }

        // The interval state saved during recording at or after a frame, if this machine has it.
        public static (int Frame, byte[] State)? CachedState(string name, int atOrAfter)
        {
            string dir = Path.Combine(StateCache, name);
            if (!System.IO.Directory.Exists(dir)) return null;
            return System.IO.Directory.GetFiles(dir, "*.state")
                .Select(p => (Frame: int.Parse(Path.GetFileNameWithoutExtension(p), CultureInfo.InvariantCulture), Path: p))
                .Where(s => s.Frame >= atOrAfter).OrderBy(s => s.Frame)
                .Select(s => ((int, byte[])?)(s.Frame, File.ReadAllBytes(s.Path))).FirstOrDefault();
        }

        public static void SaveState(string name, int frame, byte[] state)
        {
            string dir = Path.Combine(StateCache, name);
            System.IO.Directory.CreateDirectory(dir);
            File.WriteAllBytes(Path.Combine(dir, frame.ToString(CultureInfo.InvariantCulture) + ".state"), state);
        }
    }
}
