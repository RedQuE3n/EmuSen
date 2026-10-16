using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures.Snes;

namespace EmuSen.WiseMan.Fixtures
{
    // One golden run of VenusRT: an image, an input script, resets, and frames; a corpus run ends at its suite's verdict - see VenusRT_Native.md §68.2.
    public sealed record VenusGoldenRun(string Name, Func<byte[]> Rom, string Input = "none", bool SkipRendering = false, int Frames = 0, bool Corpus = false, string? CorpusPath = null, int[]? Resets = null)
    {
        public override string ToString() => Name;
    }

    // Recording and grading VenusRT's golden runs through ICore, as a frontend holds the engine, with no firmware so the open replacements run - see VenusRT_Native.md §68.2.
    public static class VenusGolden
    {
        // Set to 1 to write traces instead of grading them; the reason variable must say why, and is written into each trace that changes.
        public const string RecordVariable = "EMUSEN_VENUSRT_GOLDEN_RECORD";
        public const string ReasonVariable = "EMUSEN_VENUSRT_GOLDEN_REASON";

        // Where the full states every IntervalFrames frames go, outside the tree; ~/.cache/emusen/venusrt-goldens by default.
        public const string StatesVariable = "EMUSEN_VENUSRT_GOLDEN_STATES";

        public const int IntervalFrames = 300;
        public const int ChainRow = 60;

        public const string Heading = "A golden trace of VenusRT, recorded from VenusRT itself; re-recorded only deliberately, with the reason below - see VenusRT_Native.md §68.2.";

        public static string Directory([CallerFilePath] string here = "") =>
            Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(here)!)!, "Cores", "VenusGoldens");

        public static string StateCache => Environment.GetEnvironmentVariable(StatesVariable) is { Length: > 0 } dir ? dir
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "venusrt-goldens");

        public static bool Recording => Environment.GetEnvironmentVariable(RecordVariable) == "1";

        public static string Reason => Environment.GetEnvironmentVariable(ReasonVariable) is { Length: > 0 } r ? r
            : throw new InvalidOperationException($"{RecordVariable}=1 needs {ReasonVariable} to say why the goldens are re-recorded.");

        public static string TracePath(string name) => Path.Combine(Directory(), name + ".trace");

        public static DiscoveredCore? Discovered => CoreDiscovery.Found.SingleOrDefault(c => c.Info.Id == "venusrt");

        // Bit n of a mask is PadButton n, the SNES pad's twelve in its own order.
        public static readonly PadButton[] MaskOrder = Enumerable.Range(0, 12).Select(i => (PadButton)i).ToArray();

        // Both pads' masks on a frame numbered from 0: the bench's Start and A on pad 1, or both pads changed every seventh frame.
        public static (uint One, uint Two) ScriptMask(string input, int frame) => input switch
        {
            "none" => (0, 0),
            "bench" => ((frame % 90) switch { < 5 => 1u << (int)PadButton.Start, >= 45 and < 50 => 1u << (int)PadButton.A, _ => 0u }, 0),
            "pads" => (((uint)(frame / 7 * 7) * 37) & 0xFFF, ((uint)(frame / 7 * 7) * 91) & 0xFFF),
            _ => throw new ArgumentException($"no input script named {input}"),
        };

        // The script's fingerprint over the frames a run takes, so a reader can check its copy.
        public static string ScriptName(string input, int frames)
        {
            if (input == "none") return input;
            var masks = new byte[frames * 4];
            for (int f = 0; f < frames; f++)
            {
                var (one, two) = ScriptMask(input, f);
                BitConverter.TryWriteBytes(masks.AsSpan(4 * f), (ushort)one);
                BitConverter.TryWriteBytes(masks.AsSpan(4 * f + 2), (ushort)two);
            }
            return $"{input} {GoldenTrace.Hash(masks)}";
        }

        // The firmware folder every run sees: empty, so a chip's open replacement runs, as for a player with no dumps.
        public static string NoFirmware => System.IO.Directory.CreateDirectory(Path.Combine(Path.GetTempPath(), $"emusen-venusrt-goldens-{Environment.ProcessId}", "no-firmware")).FullName;

        public static GoldenTrace Run(VenusGoldenRun run, byte[]? resumeFrom = null, int resumeAt = 0, Action<int, byte[]>? interval = null)
        {
            Assert.True(Discovered is not null, "VenusRT's library was not found beside the tests");
            byte[] rom = run.Rom();
            var trace = new GoldenTrace
            {
                Heading = Heading,
                Name = run.Name,
                Rom = Convert.ToHexString(SHA256.HashData(rom)),
                Input = ScriptName(run.Input, run.Frames),
                Rendering = run.SkipRendering ? "skipped" : "on",
                RecordedBy = "VenusRT",
                Unit = run.Corpus ? "chain" : "frame",
            };
            CoreOptions.BatteryRamDisabled = true;
            using var core = new CoreEngine(Discovered!.Open()!);
            core.LoadRom(RomFile(rom, run.CorpusPath));
            core.SkipRendering = run.SkipRendering;
            if (resumeFrom is not null) core.LoadState(new MemoryStream(resumeFrom, writable: false));
            int[] graded = run.Corpus ? SnesTestRomCorpus.Frames(run.CorpusPath!) : Array.Empty<int>();
            var shots = new List<SnesSnapshot>();
            var chain = new byte[8];
            var link = new byte[24];
            (uint, uint) held = (uint.MaxValue, uint.MaxValue);
            int done = resumeAt;
            for (int f = resumeAt; f < run.Frames; f++)
            {
                if (run.Resets?.Contains(f) == true) core.Machine.Reset();
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
                if (done % IntervalFrames == 0) interval?.Invoke(done, state);
                if (!run.Corpus)
                {
                    trace.Rows.Add(new[] { done.ToString(CultureInfo.InvariantCulture), s, p, a });
                    continue;
                }
                Convert.FromHexString(s).CopyTo(link, 0);
                Convert.FromHexString(p).CopyTo(link, 8);
                Convert.FromHexString(a).CopyTo(link, 16);
                GoldenTrace.Fold(chain, SHA256.HashData(link));
                if (done % ChainRow == 0) trace.Rows.Add(new[] { done.ToString(CultureInfo.InvariantCulture), Convert.ToHexString(chain) });
                if (graded.Contains(done)) shots.Add(Snapshot(core, done));
            }
            if (!run.Corpus) return trace;
            if (done > 0 && (trace.Rows.Count == 0 || trace.Rows[^1][0] != done.ToString(CultureInfo.InvariantCulture)))
                trace.Rows.Add(new[] { done.ToString(CultureInfo.InvariantCulture), Convert.ToHexString(chain) });
            if (!trace.Transcript.ContainsKey("verdict"))
            {
                var verdict = SnesTestRomGrader.Grade(run.CorpusPath!, new SnesRun("VenusRT", shots, core.AudioSampleRate, Array.Empty<short>()));
                trace.Transcript["verdict"] = $"{verdict.Outcome} {done}";
                if (verdict.Protocol.Length > 0) trace.Transcript["protocol"] = verdict.Protocol;
                if (verdict.Detail.Length > 0) trace.Transcript["detail"] = Printable(verdict.Detail);
            }
            return trace;
        }

        // The memories the grader reads, by the probe's lower-case names, and the picture.
        private static SnesSnapshot Snapshot(CoreEngine core, int frame)
        {
            var spaces = new Dictionary<string, byte[]>();
            foreach (string name in SnesEngine.Spaces)
            {
                if (core.Machine.Info.Spaces.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase)) is not { } space) continue;
                var bytes = new byte[core.Machine.SpaceSize(space.Id)];
                core.Machine.ReadSpace(space.Id, 0, bytes);
                spaces[name] = bytes;
            }
            return new SnesSnapshot(frame, spaces, SnesPicture.FromRgba(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight));
        }

        // A grader's detail on one line of printable ASCII, other bytes as \xNN.
        public static string Printable(string text)
        {
            var sb = new StringBuilder();
            foreach (char ch in text) sb.Append(ch is >= ' ' and < '\x7F' && ch != '\\' ? ch.ToString() : ch == '\\' ? "\\\\" : $"\\x{(int)ch & 0xFF:X2}");
            return sb.ToString();
        }

        // A corpus ROM runs from its own path, since the grader reads the sources beside it; any other image from a scratch copy named by its hash.
        private static string RomFile(byte[] rom, string? corpusPath)
        {
            if (corpusPath is not null) return corpusPath;
            string dir = Path.Combine(Path.GetTempPath(), "emusen-venusrt-goldens");
            System.IO.Directory.CreateDirectory(dir);
            string path = Path.Combine(dir, Convert.ToHexString(SHA256.HashData(rom))[..24] + ".sfc");
            if (!File.Exists(path)) File.WriteAllBytes(path, rom);
            return path;
        }

        public static byte[] State(ICore core)
        {
            using var ms = new MemoryStream();
            core.SaveState(ms);
            return ms.ToArray();
        }

        // The machine's state layout for an image, for naming the field a bisection finds.
        public static string Layout(byte[] rom)
        {
            using var core = new CoreEngine(Discovered!.Open()!);
            core.LoadRom(RomFile(rom, null));
            return core.Machine.Layout();
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
