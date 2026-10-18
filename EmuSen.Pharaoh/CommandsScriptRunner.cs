using System.Buffers.Binary;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.Galaxia.Input;
using EmuSen.Pharaoh.Audio;
using EmuSen.Pharaoh.Imaging;

namespace EmuSen.Pharaoh
{
    // --commands mode's script interpreter: verbs run one line at a time against a shared FrameRunner, anything else handed to the shell - see EmuSen_Debugging_Tools_Reference_v5.md §3.15.
    public sealed class CommandsScriptRunner
    {
        private readonly FrameRunner runner;
        private readonly DianaShInterpreter debugCmd;
        private readonly Action<string> emit;
        private bool recording;

        public CommandsScriptRunner(FrameRunner runner, DianaShInterpreter debugCmd, Action<string> emit)
        {
            this.runner = runner;
            this.debugCmd = debugCmd;
            this.emit = emit;
        }

        private CrystalSession Session => runner.Session;

        // False when the commands file could not be found; the epilogue is the caller's - see §3.15.
        public bool Run(string commandsPath)
        {
            if (!File.Exists(commandsPath))
            {
                emit($"[ERROR] Commands file not found: {commandsPath}");
                return false;
            }

            emit($"[COMMANDS] Running {commandsPath} (safety cap {runner.FrameCap} frames)...");
            emit("");
            emit("=== Command output ===");

            foreach (string rawLine in File.ReadAllLines(commandsPath))
            {
                string cmdLine = rawLine.Trim();
                if (cmdLine.Length == 0 || cmdLine.StartsWith('#')) continue;

                string[] parts = cmdLine.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries);
                string verb = parts[0].ToLowerInvariant();

                if (verb == "frames" && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    runner.RunFrames(long.Parse(parts[1]));
                }
                else if (TapController(verb) is int tapped && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    var button = Enum.Parse<PadButton>(parts[1], ignoreCase: true);
                    long duration = parts.Length >= 3 ? long.Parse(parts[2]) : 4;
                    runner.Tap(button, tapped, duration);
                }
                else if ((verb == "hold" || verb == "release") && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    var button = Enum.Parse<PadButton>(parts[1], ignoreCase: true);
                    int controller = parts.Length >= 3 ? int.Parse(parts[2]) : 1;
                    if (verb == "hold") runner.Hold(button, controller);
                    else runner.Release(button, controller);
                }
                else if (verb == "screenshot" && parts.Length >= 2)
                {
                    var (rgba, width, height) = runner.Picture();
                    BmpFile.Write(parts[1], rgba, width, height);
                    emit($"[SCREENSHOT] Frame {runner.CurrentFrame} -> {parts[1]}");
                }
                else if (verb == "waitstable")
                {
                    long maxFrames = parts.Length >= 2 ? long.Parse(parts[1]) : 300;
                    long quietFrames = parts.Length >= 3 ? long.Parse(parts[2]) : 10;
                    emit($"> {cmdLine}");
                    long stepped = runner.WaitStable(maxFrames, quietFrames);
                    emit($"[WAITSTABLE] Advanced {stepped} frame(s) (cap {maxFrames}, quiet threshold {quietFrames}) - now at frame {runner.CurrentFrame}.");
                }
                else if (verb == "contactsheet" && parts.Length >= 3)
                {
                    emit($"> {cmdLine}");
                    string path = parts[1];
                    int count = int.Parse(parts[2]);
                    long every = parts.Length >= 4 ? long.Parse(parts[3]) : 1;
                    int cols = parts.Length >= 5 ? int.Parse(parts[4]) : 8;
                    int scale = parts.Length >= 6 ? int.Parse(parts[5]) : 4;

                    var thumbs = new List<byte[]>();
                    int thumbW = 0, thumbH = 0;
                    for (int i = 0; i < count; i++)
                    {
                        runner.RunFrames(every);
                        var (rgba, width, height) = runner.Picture();
                        thumbs.Add(ContactSheet.Downsample(rgba, width, height, scale, out thumbW, out thumbH));
                    }
                    ContactSheet.WriteContactSheet(path, thumbs, thumbW, thumbH, cols);
                    emit($"[CONTACTSHEET] {count} frame(s), every {every}, {thumbW}x{thumbH} each -> {path}");
                }
                else if (verb == "tapuntil" && parts.Length >= 5)
                {
                    // Scene navigation anchored on state, not frame counts - see §3.15d.
                    emit($"> {cmdLine}");
                    var button = Enum.Parse<PadButton>(parts[1], ignoreCase: true);
                    if (SpaceNamed(parts[2]) is not { } space)
                    {
                        emit($"[WARN] No memory space named '{parts[2]}' - ignoring.");
                        continue;
                    }

                    int addr = Convert.ToInt32(parts[3], 16);
                    byte[] target = ParseHexBytes(parts[4]);
                    int cap = parts.Length >= 6 ? int.Parse(parts[5]) : 3600;
                    int every = parts.Length >= 7 ? int.Parse(parts[6]) : 20;

                    byte[] now = Sample(space, addr, target.Length);
                    int stepped = 0, taps = 0;
                    while (!now.SequenceEqual(target) && stepped < cap && runner.CurrentFrame < runner.FrameCap)
                    {
                        // One frame at a time, held included: a four-frame tap would step over the value.
                        int phase = stepped % Math.Max(every, 5);
                        if (phase == 0) { runner.Hold(button, 1); taps++; }
                        else if (phase == 4) runner.Release(button, 1);

                        runner.RunFrames(1);
                        stepped++;
                        now = Sample(space, addr, target.Length);
                    }
                    runner.Release(button, 1);

                    string to = string.Join(' ', now.Select(v => v.ToString("X2")));
                    emit(now.SequenceEqual(target)
                        ? $"[TAPUNTIL] {space} 0x{addr:X} reached {to} after {taps} {button} tap(s), {stepped} frame(s) (frame {runner.CurrentFrame})."
                        : $"[TAPUNTIL] {space} 0x{addr:X} NOT reached - still {to} after {taps} {button} tap(s), {stepped} frame(s) (cap {cap}, frame {runner.CurrentFrame}).");
                }
                else if ((verb == "waitchange" || verb == "waitvalue") && parts.Length >= 4)
                {
                    // waitstable's memory-side counterpart - see §3.15.
                    emit($"> {cmdLine}");
                    if (SpaceNamed(parts[1]) is not { } space)
                    {
                        emit($"[WARN] No memory space named '{parts[1]}' - ignoring.");
                        continue;
                    }

                    int addr = Convert.ToInt32(parts[2], 16);
                    bool waitForValue = verb == "waitvalue";
                    byte[] target = waitForValue ? ParseHexBytes(parts[3]) : new byte[Convert.ToInt32(parts[3], 16)];
                    int cap = parts.Length >= 5 ? int.Parse(parts[4]) : 600;

                    byte[] start = Sample(space, addr, target.Length);
                    byte[] now = start;
                    int stepped = 0;
                    bool Done() => waitForValue ? now.SequenceEqual(target) : !now.SequenceEqual(start);

                    while (!Done() && stepped < cap && runner.CurrentFrame < runner.FrameCap)
                    {
                        runner.RunFrames(1);
                        stepped++;
                        now = Sample(space, addr, target.Length);
                    }

                    string from = string.Join(' ', start.Select(v => v.ToString("X2")));
                    string to = string.Join(' ', now.Select(v => v.ToString("X2")));
                    emit(Done()
                        ? $"[{verb.ToUpperInvariant()}] {space} 0x{addr:X} satisfied after {stepped} frame(s): {from} -> {to} (frame {runner.CurrentFrame})."
                        : $"[{verb.ToUpperInvariant()}] {space} 0x{addr:X} NOT satisfied - still {to} after {stepped} frame(s) (cap {cap}, frame {runner.CurrentFrame}).");
                }
                else if ((verb == "savestate" || verb == "loadstate") && parts.Length >= 2)
                {
                    // Pin an interesting moment once, then re-probe it in seconds - see §3.15c.
                    emit($"> {cmdLine}");
                    if (verb == "savestate")
                    {
                        Session.SaveState(parts[1]);
                        emit($"[STATE] Frame {runner.CurrentFrame} saved -> {parts[1]}");
                    }
                    else if (!File.Exists(parts[1]))
                    {
                        emit($"[STATE] Not found: {parts[1]}");
                    }
                    else
                    {
                        // The API's one sequence after a load clears rewind's history and refreshes the snapshots - see EmuSen_Backend.md §14.
                        Session.LoadState(parts[1]);
                        emit($"[STATE] Loaded {parts[1]} (script frame counter stays at {runner.CurrentFrame})");
                    }
                }
                else if (verb == "record")
                {
                    // A real video of a run, not a grid of thumbnails - see §3.15b.
                    emit($"> {cmdLine}");
                    string mode = parts.Length >= 2 ? parts[1].ToLowerInvariant() : "info";

                    if (mode == "on")
                    {
                        if (recording) { emit("[RECORD] Already recording - `record off` first."); continue; }
                        int stride = parts.Length >= 3 ? int.Parse(parts[2]) : 1;
                        string baseDir = parts.Length >= 4 ? parts[3] : Path.Combine(DianaShSandbox.LogsDirectory, CoreName, "Recordings");
                        string dir = Session.StartRecording(baseDir, (uint)Math.Max(1, stride));
                        RecordingDir = dir;
                        recording = true;
                        emit($"[RECORD] Started at frame {runner.CurrentFrame}, every {stride} frame(s) -> {dir}");
                    }
                    else if (mode == "off")
                    {
                        if (!recording) { emit("[RECORD] Not recording."); continue; }
                        emit(StopRecording());
                    }
                    else
                    {
                        emit(recording
                            ? $"[RECORD] Recording -> {RecordingDir}"
                            : "[RECORD] Not recording. `record on [<stride>] [<dir>]` starts one.");
                    }
                }
                else if (verb == "fastforward" || verb == "ff")
                {
                    // Headless already runs unpaced, so this is frame skip only - see §3.15.
                    emit($"> {cmdLine}");
                    bool on = parts.Length < 2 || parts[1].Equals("on", StringComparison.OrdinalIgnoreCase);
                    Session.Speed(100, drawNone: on);
                    emit($"[FASTFORWARD] Scanline rendering {(on ? "OFF - frames advance without drawing" : "ON")} (frame {runner.CurrentFrame}).");
                }
                else if (verb == "rewind")
                {
                    emit($"> {cmdLine}");
                    string mode = parts.Length >= 2 ? parts[1].ToLowerInvariant() : "info";

                    if (mode == "on")
                    {
                        uint interval = parts.Length >= 3 ? uint.Parse(parts[2]) : 0;
                        ulong budget = parts.Length >= 4 ? ulong.Parse(parts[3]) * 1024 * 1024 : 0;
                        Session.Rewind(true, interval, budget);
                        JsonElement reel = Reel();
                        runner.RewindInterval = reel.GetProperty("interval_frames").GetInt32();
                        emit($"[REWIND] On - snapshot every {runner.RewindInterval} frame(s), budget {reel.GetProperty("budget_bytes").GetInt64() / (1024 * 1024)}MB, {reel.GetProperty("snapshot_bytes").GetInt64() / 1024}KB per raw state.");
                    }
                    else if (mode == "off")
                    {
                        Session.Rewind(false);
                        emit("[REWIND] Off - chain cleared.");
                    }
                    else if (mode == "back")
                    {
                        int steps = parts.Length >= 3 ? int.Parse(parts[2]) : 1;
                        int done = runner.StepBack(steps);
                        int depth = Reel().GetProperty("depth").GetInt32();
                        emit(done == steps
                            ? $"[REWIND] Stepped back {done} snapshot(s) - now at frame {runner.CurrentFrame}, {depth} left."
                            : $"[REWIND] Only {done} of {steps} snapshot(s) available - now at frame {runner.CurrentFrame}, chain exhausted.");
                    }
                    else
                    {
                        JsonElement reel = Reel();
                        emit($"[REWIND] {(reel.GetProperty("enabled").GetBoolean() ? "On" : "Off")} - depth {reel.GetProperty("depth").GetInt32()} ({reel.GetProperty("seconds").GetDouble():F1}s), " +
                             $"{reel.GetProperty("buffered_bytes").GetInt64() / 1024}KB held, {reel.GetProperty("snapshot_bytes").GetInt64() / 1024}KB per raw state, every {reel.GetProperty("interval_frames").GetInt32()} frame(s).");
                    }
                }
                else if (verb == "perf")
                {
                    // Per-frame cost profile over a window - see §3.20.
                    emit($"> {cmdLine}");
                    long count = parts.Length >= 2 ? long.Parse(parts[1]) : 300;
                    int worst = parts.Length >= 3 ? int.Parse(parts[2]) : 5;
                    emit(RunPerf(count, worst));
                }
                else if (verb == "framesum")
                {
                    // Output-identity digest over a window - see §3.21.
                    emit($"> {cmdLine}");
                    emit(RunFrameSum(parts.Length >= 2 ? long.Parse(parts[1]) : 300));
                }
                else if (verb == "audiosum")
                {
                    // Output-identity digest over the mixed audio - see §3.22.
                    emit($"> {cmdLine}");
                    emit(RunAudioSum(parts.Length >= 2 ? long.Parse(parts[1]) : 300));
                }
                else if ((verb == "vramsheet" || verb == "paletteswatch") && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    byte[] sheet = verb == "vramsheet" ? Session.TileSheet() : Session.PaletteSwatch();
                    int w = (int)BinaryPrimitives.ReadUInt32LittleEndian(sheet), h = (int)BinaryPrimitives.ReadUInt32LittleEndian(sheet.AsSpan(4));
                    BmpFile.Write(parts[1], sheet[8..], w, h);
                    emit($"[{verb.ToUpperInvariant()}] {w}x{h} -> {parts[1]}");
                }
                else if (verb == "spriteoverlay" && parts.Length >= 2)
                {
                    // Outlines every sprite the session lists - already core-agnostic.
                    emit($"> {cmdLine}");
                    var (rgba, width, height) = runner.Picture();
                    JsonElement[] sprites = JsonDocument.Parse(Session.Sprites()).RootElement.GetProperty("sprites").EnumerateArray().ToArray();
                    foreach (JsonElement s in sprites)
                        SpriteOverlay.DrawSpriteOutline(rgba, width, height, s.GetProperty("x").GetInt32(), s.GetProperty("y").GetInt32(), s.GetProperty("width").GetInt32(), s.GetProperty("height").GetInt32());
                    BmpFile.Write(parts[1], rgba, width, height);
                    emit($"[SPRITEOVERLAY] {sprites.Length} sprite(s) outlined -> {parts[1]}");
                }
                else if (verb == "audiodump" && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    int maxSamples = parts.Length >= 3 ? int.Parse(parts[2]) : int.MaxValue;
                    byte[] peek = Session.AudioPeek();
                    int sampleRate = (int)BinaryPrimitives.ReadUInt32LittleEndian(peek);
                    var samples = new short[(peek.Length - 4) / 2];
                    for (int i = 0; i < samples.Length; i++) samples[i] = BinaryPrimitives.ReadInt16LittleEndian(peek.AsSpan(4 + i * 2));
                    if (samples.Length > maxSamples) samples = samples[..maxSamples];
                    WavFile.Write(parts[1], samples, sampleRate);
                    emit($"[AUDIODUMP] {samples.Length} sample(s) @ {sampleRate}Hz -> {parts[1]}");
                }
                else
                {
                    emit($"> {cmdLine}");
                    emit(debugCmd.Execute(cmdLine));
                }
            }

            // A script that never says `record off` still gets its video.
            if (recording) emit(StopRecording());

            return true;
        }

        // tap is controller 1's and tapN controller N's, tap2 as it always was - see EmuSen_Input.md §8.7.
        public static int? TapController(string verb) =>
            verb == "tap" ? 1 : verb.StartsWith("tap", StringComparison.Ordinal) && int.TryParse(verb.AsSpan(3), System.Globalization.NumberStyles.None, null, out int n) && n is >= 1 and <= 8 ? n : null;

        // The engine's name as the debugger gives it, which a recording's folder is named for.
        private string CoreName => JsonDocument.Parse(Session.ServiceInfo()).RootElement.GetProperty("engine").GetString() ?? "core";

        private string? RecordingDir { get; set; }

        private JsonElement Reel() => JsonDocument.Parse(Session.RewindReel()).RootElement;

        private string StopRecording()
        {
            string dir = Session.StopRecording();
            recording = false;
            return $"[RECORD] Stopped at frame {runner.CurrentFrame} -> {dir}";
        }

        // A memory space by its name in any case, as the session lists it; null when it has none of that name.
        private string? SpaceNamed(string name) =>
            JsonDocument.Parse(Session.Spaces()).RootElement.EnumerateArray().Select(s => s.GetProperty("name").GetString()).FirstOrDefault(n => string.Equals(n, name, StringComparison.OrdinalIgnoreCase));

        private byte[] Sample(string space, int addr, int length)
        {
            var bytes = new byte[length];
            Session.ReadSpace(space, unchecked((uint)addr), bytes);
            return bytes;
        }

        // Wall clock per frame plus the core's own phase breakdown, so a slow game can be attributed - see §3.20.
        private string RunPerf(long count, int worstCount)
        {
            string[] phaseNames = JsonDocument.Parse(Session.PhaseNames()).RootElement.EnumerateArray().Select(e => e.GetString() ?? "").ToArray();
            var current = new double[phaseNames.Length];

            var wall = new double[count];
            var phases = new double[phaseNames.Length][];
            for (int p = 0; p < phaseNames.Length; p++) phases[p] = new double[count];

            double toMs = 1000.0 / System.Diagnostics.Stopwatch.Frequency;
            long taken = 0;
            for (long i = 0; i < count; i++)
            {
                if (runner.CurrentFrame >= runner.FrameCap) break;
                long t0 = System.Diagnostics.Stopwatch.GetTimestamp();
                runner.RunFrames(1);
                wall[i] = (System.Diagnostics.Stopwatch.GetTimestamp() - t0) * toMs;
                int n = Session.Phases(current);
                for (int p = 0; p < phaseNames.Length && p < n; p++) phases[p][i] = current[p];
                taken++;
            }
            if (taken == 0) return "[PERF] No frames ran - already at the safety cap.";

            var sortedWall = wall.Take((int)taken).OrderBy(v => v).ToArray();
            double Pct(double p) => sortedWall[Math.Min(sortedWall.Length - 1, (int)(p * sortedWall.Length))];
            double Mean(double[] a) => a.Take((int)taken).Average();
            double hz = runner.Last.frame_rate_hz;
            double budget = 1000.0 / hz;

            var sb = new System.Text.StringBuilder();
            sb.AppendLine($"[PERF] {taken} frame(s) to frame {runner.CurrentFrame}, budget {budget:F2}ms/frame at {hz:F2}Hz.");
            sb.AppendLine($"  wall  mean {Mean(wall):F2}ms ({1000.0 / Mean(wall):F1} fps)  p50 {Pct(0.50):F2}  p95 {Pct(0.95):F2}  max {sortedWall[^1]:F2}");
            sb.AppendLine($"  over budget: {sortedWall.Count(v => v > budget)}/{taken} frame(s)");

            if (phaseNames.Length == 0)
            {
                sb.AppendLine($"  {CoreName} publishes no per-phase breakdown.");
            }
            else
            {
                // Only unnested phases sum to the frame; a "parent/child" one is already inside its parent.
                double attributed = 0;
                var top = new List<string>();
                var nested = new List<string>();
                for (int p = 0; p < phaseNames.Length; p++)
                {
                    double mean = Mean(phases[p]);
                    if (phaseNames[p].Contains('/'))
                    {
                        nested.Add($"{phaseNames[p]} {mean:F2}ms");
                        continue;
                    }
                    top.Add($"{phaseNames[p]} {mean:F2}ms");
                    attributed += mean;
                }
                sb.AppendLine("  " + string.Join("   ", top) + $"   unattributed {Mean(wall) - attributed:F2}ms");
                if (nested.Count > 0) sb.AppendLine("  split: " + string.Join("  ", nested));
            }

            var worst = Enumerable.Range(0, (int)taken).OrderByDescending(i => wall[i]).Take(worstCount);
            foreach (int i in worst)
            {
                string detail = phaseNames.Length == 0
                    ? ""
                    : " (" + string.Join(" ", phaseNames.Index().Where(n => !n.Item.Contains('/'))
                        .Select(n => $"{n.Item} {phases[n.Index][i]:F2}")) + ")";
                sb.AppendLine($"  worst frame +{i}: {wall[i]:F2}ms{detail}");
            }
            return sb.ToString().TrimEnd();
        }

        // Folds every frame's FrameHash into one digest, so a change can be shown pixel-identical across a window - see §3.21.
        private string RunFrameSum(long count)
        {
            const ulong FnvOffsetBasis = 14695981039346656037;
            const ulong FnvPrime = 1099511628211;

            ulong digest = FnvOffsetBasis;
            long taken = 0;
            for (long i = 0; i < count; i++)
            {
                if (runner.CurrentFrame >= runner.FrameCap) break;
                runner.RunFrames(1);
                digest = (digest ^ FrameHash.Compute(runner.Picture().Rgba)) * FnvPrime;
                taken++;
            }
            return $"[FRAMESUM] {taken} frame(s) to frame {runner.CurrentFrame}: {digest:X16}";
        }

        // framesum's counterpart for audio - see §3.22.
        private string RunAudioSum(long count)
        {
            const ulong FnvOffsetBasis = 14695981039346656037;
            const ulong FnvPrime = 1099511628211;
            ulong digest = FnvOffsetBasis;
            long taken = 0, samples = 0, nonzero = 0;
            double sumSquares = 0;
            int peak = 0;

            for (long i = 0; i < count; i++)
            {
                if (runner.CurrentFrame >= runner.FrameCap) break;
                runner.RunFrames(1);
                ReadOnlySpan<short> chunk = runner.DrainSound();
                foreach (short s in chunk)
                {
                    digest = (digest ^ (ulong)(ushort)s) * FnvPrime;
                    if (s != 0) nonzero++;
                    sumSquares += (double)s * s;
                    int mag = Math.Abs((int)s);
                    if (mag > peak) peak = mag;
                }
                samples += chunk.Length;
                taken++;
            }

            if (taken == 0) return "[AUDIOSUM] No frames ran - already at the safety cap.";
            double rms = samples == 0 ? 0 : Math.Sqrt(sumSquares / samples);
            return $"[AUDIOSUM] {taken} frame(s) to frame {runner.CurrentFrame}: {digest:X16} "
                 + $"samples={samples} nonzero={nonzero} ({(samples == 0 ? 0 : 100.0 * nonzero / samples):F1}%) peak={peak} rms={rms:F1}";
        }

        // "1E" or "1E,00" or "1E 00" - see §3.15's waitvalue entry.
        private static byte[] ParseHexBytes(string spec)
        {
            return spec.Split(new[] { ',', ':' }, StringSplitOptions.RemoveEmptyEntries)
                .Select(p => Convert.ToByte(p, 16))
                .ToArray();
        }
    }
}
