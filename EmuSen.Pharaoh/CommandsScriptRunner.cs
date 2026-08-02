using EmuSen.Common.Imaging;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Pharaoh
{
    // --commands mode's script interpreter: an ordered list of verbs
    // executed one line at a time against a shared FrameRunner, so
    // frame-stepping, input, screenshots, and debug commands can interleave
    // freely in one script. Full verb syntax: EmuSen.DianaOS/DianaOS/Usr/Home/Documents/
    // EmuSen Manual/EmuSen_Debugging_Tools_Reference_v5.md §3.15. Anything not recognized
    // as one of those verbs falls through to the real DianaOSInterpreter.
    public sealed class CommandsScriptRunner
    {
        private readonly FrameRunner runner;
        private readonly SnesDebugTarget debugTarget;
        private readonly DianaOSInterpreter debugCmd;
        private readonly Action<string> emit;

        public CommandsScriptRunner(FrameRunner runner, SnesDebugTarget debugTarget, DianaOSInterpreter debugCmd, Action<string> emit)
        {
            this.runner = runner;
            this.debugTarget = debugTarget;
            this.debugCmd = debugCmd;
            this.emit = emit;
        }

        // Returns false if the commands file itself couldn't be found
        // (caller emits nothing further and exits 1) - true once the whole
        // script has run to completion. Deliberately doesn't touch
        // FlushVerboseTrace/"[RUN] Done"/savestate/--out - those are the
        // shared epilogue, identical for this mode and the classic loop,
        // handled once by the caller after this returns.
        public bool Run(string commandsPath)
        {
            if (!File.Exists(commandsPath))
            {
                emit($"[ERROR] Commands file not found: {commandsPath}");
                return false;
            }

            var core = runner.Core;

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
                else if ((verb == "tap" || verb == "tap2") && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    int controller = verb == "tap2" ? 2 : 1;
                    var button = Enum.Parse<SnesButton>(parts[1], ignoreCase: true);
                    long duration = parts.Length >= 3 ? long.Parse(parts[2]) : 4;
                    runner.Tap(button, controller, duration);
                }
                else if ((verb == "hold" || verb == "release") && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    var button = Enum.Parse<SnesButton>(parts[1], ignoreCase: true);
                    int controller = parts.Length >= 3 ? int.Parse(parts[2]) : 1;
                    if (verb == "hold") runner.Hold(button, controller);
                    else runner.Release(button, controller);
                }
                else if (verb == "screenshot" && parts.Length >= 2)
                {
                    BmpFile.Write(parts[1], core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight);
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
                    // <path> <count> [every=1] [cols=8] [scale=4] - see §3.15.
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
                        thumbs.Add(ContactSheet.Downsample(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight, scale, out thumbW, out thumbH));
                    }
                    ContactSheet.WriteContactSheet(path, thumbs, thumbW, thumbH, cols);
                    emit($"[CONTACTSHEET] {count} frame(s), every {every}, {thumbW}x{thumbH} each -> {path}");
                }
                else if ((verb == "waitchange" || verb == "waitvalue") && parts.Length >= 4)
                {
                    // waitstable's memory-side counterpart - see §3.15.
                    emit($"> {cmdLine}");
                    string spaceName = parts[1];
                    var space = debugTarget.GetMemorySpaces()
                        .FirstOrDefault(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
                    if (space == null)
                    {
                        emit($"[WARN] No memory space named '{spaceName}' - ignoring.");
                        continue;
                    }

                    int addr = Convert.ToInt32(parts[2], 16);
                    bool waitForValue = verb == "waitvalue";
                    byte[] target = waitForValue ? ParseHexBytes(parts[3]) : new byte[Convert.ToInt32(parts[3], 16)];
                    int cap = parts.Length >= 5 ? int.Parse(parts[4]) : 600;

                    byte[] Sample()
                    {
                        var b = new byte[target.Length];
                        for (int i = 0; i < b.Length; i++) b[i] = space.Read(addr + i);
                        return b;
                    }

                    byte[] start = Sample();
                    byte[] now = start;
                    int stepped = 0;
                    bool Done() => waitForValue ? now.SequenceEqual(target) : !now.SequenceEqual(start);

                    while (!Done() && stepped < cap && runner.CurrentFrame < runner.FrameCap)
                    {
                        runner.RunFrames(1);
                        stepped++;
                        now = Sample();
                    }

                    string from = string.Join(' ', start.Select(v => v.ToString("X2")));
                    string to = string.Join(' ', now.Select(v => v.ToString("X2")));
                    emit(Done()
                        ? $"[{verb.ToUpperInvariant()}] {space.Name} 0x{addr:X} satisfied after {stepped} frame(s): {from} -> {to} (frame {runner.CurrentFrame})."
                        : $"[{verb.ToUpperInvariant()}] {space.Name} 0x{addr:X} NOT satisfied - still {to} after {stepped} frame(s) (cap {cap}, frame {runner.CurrentFrame}).");
                }
                else if (verb == "fastforward" || verb == "ff")
                {
                    // Headless already runs unpaced, so this is frame skip only - see §3.15.
                    emit($"> {cmdLine}");
                    bool on = parts.Length < 2 || parts[1].Equals("on", StringComparison.OrdinalIgnoreCase);
                    core.SkipRendering = on;
                    emit($"[FASTFORWARD] Scanline rendering {(on ? "OFF - frames advance without drawing" : "ON")} (frame {runner.CurrentFrame}).");
                }
                else if (verb == "rewind")
                {
                    emit($"> {cmdLine}");
                    string mode = parts.Length >= 2 ? parts[1].ToLowerInvariant() : "info";
                    var rewind = runner.Rewind;

                    if (mode == "on")
                    {
                        if (parts.Length >= 3) rewind.IntervalFrames = int.Parse(parts[2]);
                        if (parts.Length >= 4) rewind.BudgetBytes = long.Parse(parts[3]) * 1024 * 1024;
                        rewind.Enabled = true;
                        rewind.CaptureNow(core);
                        emit($"[REWIND] On - snapshot every {rewind.IntervalFrames} frame(s), budget {rewind.BudgetBytes / (1024 * 1024)}MB, {rewind.SnapshotBytes / 1024}KB per raw state.");
                    }
                    else if (mode == "off")
                    {
                        rewind.Enabled = false;
                        rewind.Clear();
                        emit("[REWIND] Off - chain cleared.");
                    }
                    else if (mode == "back")
                    {
                        int steps = parts.Length >= 3 ? int.Parse(parts[2]) : 1;
                        int done = runner.StepBack(steps);
                        // No RunFrame() ran, so regs/sprites/pal would be stale - see §3.15.
                        debugTarget.RefreshProviders();
                        emit(done == steps
                            ? $"[REWIND] Stepped back {done} snapshot(s) - now at frame {runner.CurrentFrame}, {rewind.Depth} left."
                            : $"[REWIND] Only {done} of {steps} snapshot(s) available - now at frame {runner.CurrentFrame}, chain exhausted.");
                    }
                    else
                    {
                        emit($"[REWIND] {(rewind.Enabled ? "On" : "Off")} - depth {rewind.Depth} ({rewind.BufferedSeconds(core.FrameRateHz):F1}s), " +
                             $"{rewind.BufferedBytes / 1024}KB held, {rewind.SnapshotBytes / 1024}KB per raw state, every {rewind.IntervalFrames} frame(s).");
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
                else if (verb == "layers")
                {
                    // Isolates one layer at a time - see §3.19.
                    emit($"> {cmdLine}");
                    string spec = parts.Length >= 2 ? parts[1].ToLowerInvariant() : "all";
                    int mask = spec switch
                    {
                        "all" => 0x1F,
                        "none" => 0x00,
                        "bg1" => 0x01,
                        "bg2" => 0x02,
                        "bg3" => 0x04,
                        "bg4" => 0x08,
                        "obj" => 0x10,
                        _ => Convert.ToInt32(spec, 16)
                    };
                    EmuSen.Debug.DebugSettings.LayerEnableMask = mask;
                    emit($"[LAYERS] Mask 0x{mask:X2} - {(mask == 0x1F ? "all layers" : DescribeLayerMask(mask))}.");
                }
                else if (verb == "scanregs")
                {
                    // Per-scanline PPU register trace - see §3.19.
                    emit($"> {cmdLine}");
                    string spec = parts.Length >= 2 ? parts[1].ToLowerInvariant() : "off";
                    int line = spec switch
                    {
                        "off" => -1,
                        "all" => -2,
                        _ => int.Parse(spec)
                    };
                    EmuSen.Debug.DebugSettings.ScanlineRegisterDumpLine = line;
                    emit($"[SCANREGS] {(line == -1 ? "off" : line == -2 ? "every scanline" : $"scanline {line} only")}.");
                }
                else if (verb == "vramsheet" && parts.Length >= 2)
                {
                    // IDebugTarget.RenderTileSheet(), not Renderer directly - core-agnostic.
                    emit($"> {cmdLine}");
                    var (rgba, w, h) = debugTarget.RenderTileSheet();
                    BmpFile.Write(parts[1], rgba, w, h);
                    emit($"[VRAMSHEET] {w}x{h} -> {parts[1]}");
                }
                else if (verb == "paletteswatch" && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    var (rgba, w, h) = debugTarget.RenderPaletteSwatch();
                    BmpFile.Write(parts[1], rgba, w, h);
                    emit($"[PALETTESWATCH] {w}x{h} -> {parts[1]}");
                }
                else if (verb == "spriteoverlay" && parts.Length >= 2)
                {
                    // Outlines every IDebugTarget.Sprites entry - already core-agnostic.
                    emit($"> {cmdLine}");
                    byte[] rgba = core.GetFrameBufferRgba();
                    var sprites = debugTarget.Sprites.Current;
                    foreach (var s in sprites)
                    {
                        SpriteOverlay.DrawSpriteOutline(rgba, core.ScreenWidth, core.ScreenHeight, s);
                    }
                    BmpFile.Write(parts[1], rgba, core.ScreenWidth, core.ScreenHeight);
                    emit($"[SPRITEOVERLAY] {sprites.Count} sprite(s) outlined -> {parts[1]}");
                }
                else if (verb == "audiodump" && parts.Length >= 2)
                {
                    emit($"> {cmdLine}");
                    int maxSamples = parts.Length >= 3 ? int.Parse(parts[2]) : int.MaxValue;
                    var (samples, sampleRate) = debugTarget.GetAudioSamples();
                    if (samples.Length > maxSamples) samples = samples[..maxSamples];
                    EmuSen.Audio.WavFile.Write(parts[1], samples, sampleRate);
                    emit($"[AUDIODUMP] {samples.Length} sample(s) @ {sampleRate}Hz -> {parts[1]}");
                }
                else
                {
                    emit($"> {cmdLine}");
                    emit(debugCmd.Execute(cmdLine));
                }
            }

            return true;
        }

        // Wall clock per frame plus the core's own phase breakdown, so a
        // slow game can be attributed rather than just observed - see §3.20.
        private string RunPerf(long count, int worstCount)
        {
            var core = runner.Core;
            var wall = new double[count];
            var cpu = new double[count];
            var ppu = new double[count];
            var hdma = new double[count];
            var objEval = new double[count];
            var blend = new double[count];
            var mainComp = new double[count];
            var subComp = new double[count];

            double toMs = 1000.0 / System.Diagnostics.Stopwatch.Frequency;
            long sa1RunAtStart = core.Cart?.Sa1?.ExecutedMasterClocks ?? 0;
            long sa1OfferedAtStart = core.Cart?.Sa1?.OfferedMasterClocks ?? 0;
            long taken = 0;
            for (long i = 0; i < count; i++)
            {
                if (runner.CurrentFrame >= runner.FrameCap) break;
                long t0 = System.Diagnostics.Stopwatch.GetTimestamp();
                runner.RunFrames(1);
                wall[i] = (System.Diagnostics.Stopwatch.GetTimestamp() - t0) * toMs;
                cpu[i] = core.LastFrameCpuSpc700Ms;
                ppu[i] = core.LastFramePpuMs;
                hdma[i] = core.LastFrameHdmaMs;
                objEval[i] = core.LastFrameObjEvalMs;
                blend[i] = core.LastFrameBlendMs;
                mainComp[i] = core.LastFrameMainCompositeMs;
                subComp[i] = core.LastFrameSubCompositeMs;
                taken++;
            }
            if (taken == 0) return "[PERF] No frames ran - already at the safety cap.";

            // Clocks per frame is the figure that says whether a coprocessor is
            // being starved or double-clocked, and the offered/run gap is how
            // long the game parked it - see Venus_SA1.md §2.3.
            string coprocessorLine = string.Empty;
            if (core.Cart?.Sa1 is { } perfSa1)
            {
                double runPerFrame = (perfSa1.ExecutedMasterClocks - sa1RunAtStart) / (double)taken;
                double offeredPerFrame = (perfSa1.OfferedMasterClocks - sa1OfferedAtStart) / (double)taken;
                double expected = core.MasterClocksPerFrame;
                coprocessorLine = $"  sa-1: {runPerFrame:F0} clocks/frame run of {offeredPerFrame:F0} offered "
                                + $"({runPerFrame / expected * 100:F1}% of the {expected:F0} a full-rate frame allows)";
            }

            var sortedWall = wall.Take((int)taken).OrderBy(v => v).ToArray();
            double Pct(double p) => sortedWall[Math.Min(sortedWall.Length - 1, (int)(p * sortedWall.Length))];
            double Mean(double[] a) => a.Take((int)taken).Average();
            double budget = 1000.0 / core.FrameRateHz;

            var sb = new System.Text.StringBuilder();
            sb.AppendLine($"[PERF] {taken} frame(s) to frame {runner.CurrentFrame}, budget {budget:F2}ms/frame at {core.FrameRateHz:F2}Hz.");
            sb.AppendLine($"  wall  mean {Mean(wall):F2}ms ({1000.0 / Mean(wall):F1} fps)  p50 {Pct(0.50):F2}  p95 {Pct(0.95):F2}  max {sortedWall[^1]:F2}");
            sb.AppendLine($"  over budget: {sortedWall.Count(v => v > budget)}/{taken} frame(s)");
            sb.AppendLine($"  cpu+spc700 {Mean(cpu):F2}ms   ppu {Mean(ppu):F2}ms   hdma {Mean(hdma):F2}ms   unattributed {Mean(wall) - Mean(cpu) - Mean(ppu) - Mean(hdma):F2}ms");
            sb.AppendLine($"  ppu split: mainComposite {Mean(mainComp):F2}ms  subComposite {Mean(subComp):F2}ms  objEval {Mean(objEval):F2}ms  blend {Mean(blend):F2}ms");
            if (coprocessorLine.Length > 0) sb.AppendLine(coprocessorLine);

            var worst = Enumerable.Range(0, (int)taken).OrderByDescending(i => wall[i]).Take(worstCount);
            foreach (int i in worst)
            {
                sb.AppendLine($"  worst frame +{i}: {wall[i]:F2}ms (cpu {cpu[i]:F2} ppu {ppu[i]:F2} hdma {hdma[i]:F2})");
            }
            return sb.ToString().TrimEnd();
        }

        // Folds every frame's FrameHash into one digest, so a renderer change can be
        // shown pixel-identical across a whole window, not one sampled frame - see §3.21.
        private string RunFrameSum(long count)
        {
            const ulong FnvOffsetBasis = 14695981039346656037;
            const ulong FnvPrime = 1099511628211;

            var core = runner.Core;
            ulong digest = FnvOffsetBasis;
            long taken = 0;
            for (long i = 0; i < count; i++)
            {
                if (runner.CurrentFrame >= runner.FrameCap) break;
                runner.RunFrames(1);
                digest = (digest ^ FrameHash.Compute(core.GetFrameBufferRgba())) * FnvPrime;
                taken++;
            }
            return $"[FRAMESUM] {taken} frame(s) to frame {runner.CurrentFrame}: {digest:X16}";
        }

        // framesum's counterpart for audio - see §3.22.
        private string RunAudioSum(long count)
        {
            const ulong FnvOffsetBasis = 14695981039346656037;
            const ulong FnvPrime = 1099511628211;
            var core = runner.Core;
            ulong digest = FnvOffsetBasis;
            long taken = 0, samples = 0, nonzero = 0;
            double sumSquares = 0;
            int peak = 0;

            for (long i = 0; i < count; i++)
            {
                if (runner.CurrentFrame >= runner.FrameCap) break;
                runner.RunFrames(1);
                short[] chunk = core.DequeueAudioSamples(int.MaxValue);
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

        private static string DescribeLayerMask(int mask)
        {
            var names = new List<string>();
            if ((mask & 0x01) != 0) names.Add("BG1");
            if ((mask & 0x02) != 0) names.Add("BG2");
            if ((mask & 0x04) != 0) names.Add("BG3");
            if ((mask & 0x08) != 0) names.Add("BG4");
            if ((mask & 0x10) != 0) names.Add("OBJ");
            return names.Count == 0 ? "nothing" : string.Join('+', names);
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
