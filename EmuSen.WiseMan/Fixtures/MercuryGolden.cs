using EmuSen.Cores.Native;
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Fixtures
{
    // One golden run: a ROM, a console, an input script, and frames or instructions - see Mercury_Native.md §14.4.
    public sealed record MercuryGoldenRun(string Name, Func<byte[]> Rom, GbModel Model = GbModel.Auto, string Input = "none", bool SkipRendering = false, int Frames = 0, int Steps = 0, bool Corpus = false)
    {
        public override string ToString() => Name;
    }

    // A golden trace: per frame eight bytes of SHA-256 of the state, the picture and the sound, or a chain over each instruction's state - see Mercury_Native.md §14.4.
    public sealed class MercuryGoldenTrace
    {
        public const int Version = 1;

        public string Name = "";
        public string Rom = "";
        public string Model = "";
        public string Input = "";
        public string Rendering = "";
        public string RecordedBy = "";

        // Frame rows (index, state, picture, sound), or step rows (index, chain); a corpus run ends at its verdict.
        public readonly List<string[]> Rows = new();
        public readonly SortedDictionary<string, string> Transcript = new(StringComparer.Ordinal);

        public string Text
        {
            get
            {
                var text = new StringBuilder();
                text.Append("# A golden trace of MercuryRT, recorded from the engine named below; re-recorded only deliberately - see Mercury_Native.md §14.4.\n");
                text.Append($"golden {Version}\nname {Name}\nrom {Rom}\nmodel {Model}\ninput {Input}\nrendering {Rendering}\nrecorded {RecordedBy}\n");
                string[]? last = null;
                foreach (string[] row in Rows)
                {
                    text.Append(row[0]);
                    for (int i = 1; i < row.Length; i++) text.Append(' ').Append(last is not null && last.Length == row.Length && last[i] == row[i] ? "=" : row[i]);
                    text.Append('\n');
                    last = row;
                }
                foreach (var (key, value) in Transcript) text.Append($"{key} {value}\n");
                return text.ToString();
            }
        }

        public static MercuryGoldenTrace Parse(string text)
        {
            var trace = new MercuryGoldenTrace();
            string[]? last = null;
            foreach (string line in text.Split('\n'))
            {
                if (line.Length == 0 || line[0] == '#') continue;
                string[] p = line.Split(' ');
                if (char.IsDigit(line[0]))
                {
                    string[] row = p.Select((v, i) => v == "=" && last is not null ? last[i] : v).ToArray();
                    trace.Rows.Add(row);
                    last = row;
                    continue;
                }
                string value = line[(p[0].Length + 1)..];
                switch (p[0])
                {
                    case "golden": Assert.Equal(Version, int.Parse(value, CultureInfo.InvariantCulture)); break;
                    case "name": trace.Name = value; break;
                    case "rom": trace.Rom = value; break;
                    case "model": trace.Model = value; break;
                    case "input": trace.Input = value; break;
                    case "rendering": trace.Rendering = value; break;
                    case "recorded": trace.RecordedBy = value; break;
                    default: trace.Transcript[p[0]] = value; break;
                }
            }
            return trace;
        }

        // The first difference from another trace of the same run, or null: the header, then row by row, then the transcript.
        public string? FirstDifference(MercuryGoldenTrace now)
        {
            foreach (var (what, a, b) in new[] { ("rom", Rom, now.Rom), ("model", Model, now.Model), ("input", Input, now.Input), ("rendering", Rendering, now.Rendering) })
                if (a != b) return $"{what}: recorded {a}, now {b}";
            string[] parts = Rows.Count > 0 && Rows[0].Length == 4 ? new[] { "", "state", "picture", "sound" } : new[] { "", "chain" };
            for (int i = 0; i < Math.Min(Rows.Count, now.Rows.Count); i++)
            {
                if (Rows[i].SequenceEqual(now.Rows[i])) continue;
                int at = Enumerable.Range(0, Rows[i].Length).First(k => k >= now.Rows[i].Length || Rows[i][k] != now.Rows[i][k]);
                string unit = parts.Length == 4 ? "frame" : "step";
                return $"{unit} {Rows[i][0]}: the {parts[Math.Min(at, parts.Length - 1)]} differs (recorded {Rows[i][at]}, now {(at < now.Rows[i].Length ? now.Rows[i][at] : "nothing")})";
            }
            if (Rows.Count != now.Rows.Count) return $"recorded {Rows.Count} rows, now {now.Rows.Count}";
            foreach (string key in Transcript.Keys.Union(now.Transcript.Keys).Order(StringComparer.Ordinal))
            {
                string? a = Transcript.GetValueOrDefault(key), b = now.Transcript.GetValueOrDefault(key);
                if (a != b) return $"{key}: recorded {a ?? "nothing"}, now {b ?? "nothing"}";
            }
            return null;
        }
    }

    // Recording and replaying golden runs on MercuryRT through the rig - see Mercury_Native.md §14.4, §15.
    public static class MercuryGolden
    {
        // Set to 1 to write traces from MercuryRT instead of grading them: a deliberate re-baseline, whose commit states its reason (§3.7's second rule).
        public const string RecordVariable = "EMUSEN_MERCURY_GOLDEN_RECORD";

        // Where the full states every IntervalFrames frames go, outside the tree; ~/.cache/emusen/mercury-goldens by default.
        public const string StatesVariable = "EMUSEN_MERCURY_GOLDEN_STATES";

        public const int IntervalFrames = 300;

        public const int StepRow = 100;

        public static string Directory([CallerFilePath] string here = "") =>
            Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(here)!)!, "Cores", "MercuryGoldens");

        public static string StateCache => Environment.GetEnvironmentVariable(StatesVariable) is { Length: > 0 } dir ? dir
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "mercury-goldens");

        public static bool Recording => Environment.GetEnvironmentVariable(RecordVariable) is { Length: > 0 };

        public static string TracePath(string name) => Path.Combine(Directory(), name + ".trace");

        public static string Hash(ReadOnlySpan<byte> bytes) => Convert.ToHexString(SHA256.HashData(bytes))[..16];

        // The bench's script, frame numbered from 0: Start for five frames of every ninety, A for five frames half way.
        public static uint ScriptMask(string input, int frame) => input switch
        {
            "none" => 0,
            "bench" => (frame % 90) switch { < 5 => 1u << 7, >= 45 and < 50 => 1u << 4, _ => 0u },
            _ => throw new ArgumentException($"no input script named {input}"),
        };

        // The script's own fingerprint over the frames a run takes, written beside its name so a reader can check its copy.
        public static string ScriptName(string input, int frames)
        {
            if (input == "none") return input;
            var masks = new byte[frames];
            for (int f = 0; f < frames; f++) masks[f] = (byte)ScriptMask(input, f);
            return $"{input} {Hash(masks)}";
        }

        // The order of a mask's bits: the pad's buttons in the C ABI's order.
        public static readonly PadButton[] MaskOrder = { PadButton.Right, PadButton.Left, PadButton.Up, PadButton.Down, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        public static MercuryGoldenTrace Run(MercuryGoldenRun run, byte[]? resumeFrom = null, int resumeAt = 0, Action<int, byte[]>? interval = null)
        {
            byte[] rom = run.Rom();
            using MercuryRig rig = MercuryRig.Create(rom, run.Model);
            rig.SkipRendering = run.SkipRendering;
            var trace = new MercuryGoldenTrace
            {
                Name = run.Name,
                Rom = Convert.ToHexString(SHA256.HashData(rom)),
                Model = run.Model.ToString(),
                Rendering = run.SkipRendering ? "skipped" : "on",
                RecordedBy = "MercuryRt",
            };
            if (run.Steps > 0) Steps(rig, run, trace);
            else Frames(rig, run, trace, resumeFrom, resumeAt, interval);
            return trace;
        }

        private static void Frames(MercuryRig rig, MercuryGoldenRun run, MercuryGoldenTrace trace, byte[]? resumeFrom, int resumeAt, Action<int, byte[]>? interval)
        {
            trace.Input = ScriptName(run.Input, run.Frames);
            if (resumeFrom is not null) rig.LoadState(new MemoryStream(resumeFrom, writable: false));
            uint held = uint.MaxValue;
            for (int f = resumeAt; f < run.Frames; f++)
            {
                uint mask = ScriptMask(run.Input, f);
                if (mask != held)
                {
                    for (int b = 0; b < MaskOrder.Length; b++) rig.SetButton(0, MaskOrder[b], (mask & (1u << b)) != 0);
                    held = mask;
                }
                try { rig.RunFrame(); }
                catch (NotSupportedException)
                {
                    trace.Transcript["verdict"] = $"IllegalOpcode {f + 1}";
                    break;
                }
                short[] samples = rig.DequeueAudioSamples(int.MaxValue);
                byte[] picture = rig.GetFrameBufferRgba();
                byte[] state = rig.State();
                trace.Rows.Add(new[] { (f + 1).ToString(CultureInfo.InvariantCulture), Hash(state), Hash(picture), Hash(MemoryMarshal.AsBytes(samples.AsSpan())) });
                if ((f + 1) % IntervalFrames == 0) interval?.Invoke(f + 1, state);
                if (run.Corpus && Verdict(rig) is { } verdict)
                {
                    trace.Transcript["verdict"] = $"{verdict} {f + 1}";
                    break;
                }
            }
            if (!run.Corpus) return;
            trace.Transcript.TryAdd("verdict", $"NoVerdict {run.Frames}");
            trace.Transcript["serial"] = Convert.ToHexString(rig.SerialLog.ToArray());
            if (HardwareTestRomLibrary.ReadCartRamStatus(i => rig.ReadSpace(MercuryEngines.SpaceCartRam, i)) is not null)
            {
                var text = new List<byte>();
                for (int i = 4; i < 0x2000; i++)
                {
                    byte b = rig.ReadSpace(MercuryEngines.SpaceCartRam, i);
                    if (b == 0) break;
                    text.Add(b);
                }
                trace.Transcript["cartram"] = Convert.ToHexString(text.ToArray());
            }
        }

        // A corpus ROM's verdict in MercuryHardwareTestRomTests' terms, or null while it runs.
        private static string? Verdict(MercuryRig rig)
        {
            TestRomVerdict serial = HardwareTestRomLibrary.ReadVerdict(rig.SerialLog);
            if (serial != TestRomVerdict.NoVerdict) return serial.ToString();
            return HardwareTestRomLibrary.ReadCartRamStatus(i => rig.ReadSpace(MercuryEngines.SpaceCartRam, i)) is int status ? (status == 0 ? "Passed" : $"Failed({status})") : null;
        }

        // Each instruction's state hashed into a chain, written every StepRow instructions and at the end; an illegal opcode ends the run.
        private static void Steps(MercuryRig rig, MercuryGoldenRun run, MercuryGoldenTrace trace)
        {
            trace.Input = "none";
            byte[] chain = new byte[8];
            var link = new byte[16];
            int done = 0;
            for (int i = 1; i <= run.Steps; i++)
            {
                try { rig.Step(); }
                catch (NotSupportedException)
                {
                    trace.Transcript["stop"] = i.ToString(CultureInfo.InvariantCulture);
                    break;
                }
                chain.CopyTo(link, 0);
                SHA256.HashData(rig.State()).AsSpan(0, 8).CopyTo(link.AsSpan(8));
                SHA256.HashData(link).AsSpan(0, 8).CopyTo(chain);
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
