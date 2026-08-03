using EmuSen.Common.Imaging;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Pharaoh;
using EmuSen.Pharaoh.Cli;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

// Headless AI-agent-driven debugging harness - loads a ROM, runs the
// real VenusCore for a fixed number of frames with no window/audio/human
// involved, then feeds a script of the exact same debug commands the F4
// prompt accepts and writes the results to a plain log file. Full
// reference for every flag, both frame-loop modes (classic and
// --commands), --diffshot, and the design reasoning behind each:
// EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Debugging_Tools_Reference_v5.md §3.15.
//
// Usage:
//   dotnet run -- <rom> <frames> [--watch space:addr:len[:kind]]... [--script path] [--out path] [--tap frame:button[:duration]]... [--tap2 frame:button[:duration]]... [--loadstate path] [--savestate path] [--screenshot frame:path]...
//   dotnet run -- <rom> <maxframes> --commands path [other flags above except --tap/--tap2/--screenshot/--script]
//   dotnet run -- --diffshot <bmp1> <bmp2> <outpath>
//
// Internally this is now just dispatch: HeadlessDebugOptions.Parse turns
// argv into a plain options object, FrameRunner is the one shared
// frame-stepping primitive both modes below drive, CommandsScriptRunner
// owns the --commands verb interpreter, and DiffShotRunner owns
// --diffshot - see each class's own comment.
class Program
{
    static int Main(string[] args)
    {
        // Standalone utility mode - no ROM/core involved, so it's checked
        // before HeadlessDebugOptions.Parse ever looks at args[0].
        if (args.Length >= 1 && args[0] == "--diffshot")
        {
            if (args.Length < 4)
            {
                Console.WriteLine("Usage: dotnet run -- --diffshot <bmp1> <bmp2> <outpath>");
                return 1;
            }
            return DiffShotRunner.Run(args[1], args[2], args[3]);
        }

        var (options, warnings, error) = HeadlessDebugOptions.Parse(args);
        if (error != null)
        {
            Console.WriteLine(error);
            return 1;
        }

        if (!File.Exists(options!.RomPath))
        {
            Console.WriteLine($"[ERROR] ROM not found: {options.RomPath}");
            return 1;
        }

        // Off by default (matches DebugSettings.MasterLoggingEnabled's own
        // default) - only flip it on when explicitly asked, since it turns
        // on every individually-enabled *Logging flag's live console spam
        // (DmaSourceAddrLogging etc.), useful for a short, targeted run but
        // far too noisy over a long one.
        if (options.Verbose) EmuSen.Debug.DebugSettings.MasterLoggingEnabled = true;

        var log = new List<string>();
        void Emit(string line)
        {
            Console.WriteLine(line);
            log.Add(line);
        }

        // HeadlessDebugOptions.Parse already validated every --flag by the
        // same DebugSettings reflection lookup below, so any warnings are
        // ready the moment Emit exists - unlike the old Console.WriteLine
        // from inside argument parsing, these now actually land in --out.
        foreach (string w in warnings) Emit(w);

        foreach (string flagSpec in options.FlagsToEnable)
        {
            string[] parts = flagSpec.Split(new[] { '=' }, 2);
            string flagName = parts[0];
            string? rawValue = parts.Length >= 2 ? parts[1] : null;

            var settingsType = typeof(EmuSen.Debug.DebugSettings);
            var prop = settingsType.GetProperty(flagName);
            var field = prop == null ? settingsType.GetField(flagName) : null;
            Type? memberType = prop?.PropertyType ?? field?.FieldType;

            if (memberType == null) continue; // already warned above

            // Parse already proved this converts, by this same function.
            HeadlessDebugOptions.TryConvertFlagValue(rawValue, memberType, out object? value, out _);
            if (prop != null) prop.SetValue(null, value);
            else field!.SetValue(null, value);

            EmuSen.Debug.DebugSettings.MasterLoggingEnabled = true;
        }

        Emit($"[ROM] Loading: {options.RomPath}");
        var core = new VenusCore(headless: true);
        core.LoadRom(options.RomPath);

        // Jumping straight to a saved moment (e.g. Link already standing
        // in a room) sidesteps blindly scripting menu-navigation input
        // with --tap, which only works when the exact input timing is
        // already known.
        if (options.LoadStatePath != null)
        {
            if (!File.Exists(options.LoadStatePath))
            {
                Emit($"[ERROR] State file not found: {options.LoadStatePath}");
                return 1;
            }
            core.LoadState(options.LoadStatePath);
            Emit($"[STATE] Loaded: {options.LoadStatePath}");
        }

        // See SnesDebugTarget's own constructor comment - feeds
        // `coretop`'s hardware-load bars (not that this headless entry
        // point has any interactive use for them today, but there's no
        // reason for it to report less than the other two frontends do).
        var debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!,
            () => (core.LastFrameCpuSpc700Ms, core.LastFramePpuMs, core.LastFrameHdmaMs));
        var debugCmd = DianaOSInterpreter.CreateDefault(debugTarget, null,
            new EmuSen.Cores.Nintendo.Venus.Cheats.ActionReplayCheatCodec(),
            new EmuSen.Cores.Nintendo.Venus.Cheats.GameGenieCheatCodec(),
            new EmuSen.Cores.Nintendo.Venus.Debug.VenusCpuTraceSwitch());

        // Same two ranges registered from power-on in Hotaru's
        // Program.cs for the Yoshi/coin investigation - duplicated here
        // rather than shared, since this entry point has no window/input
        // loop to hang that wiring off of and needs them active before the
        // very first frame regardless.
        debugTarget.Watches.AddWatch("WRAM", 0x8000, 0x1800);
        debugTarget.Watches.AddWatch("WRAM", 0x0D80, 0x0080);

        foreach (string spec in options.ExtraWatches)
        {
            string[] parts = spec.Split(':');
            if (parts.Length < 3)
            {
                Emit($"[WARN] Ignoring malformed --watch '{spec}' (expected space:addr:len[:kind])");
                continue;
            }
            string cmd = $"watch add {parts[0]} {parts[1]} {parts[2]}" + (parts.Length >= 4 ? $" {parts[3]}" : "");
            Emit(debugCmd.Execute(cmd));
        }

        // Shared between both frame-loop modes below (only one of which
        // actually runs per invocation) - see §3.15's --autoshot entry.
        ulong? lastAutoshotHash = null;
        if (options.AutoshotDir != null) Directory.CreateDirectory(options.AutoshotDir);

        void CheckAutoshot(long frameNum)
        {
            if (options.AutoshotDir == null) return;
            byte[] frame = core.GetFrameBufferRgba();
            ulong hash = FrameHash.Compute(frame);
            if (lastAutoshotHash == hash) return;
            lastAutoshotHash = hash;
            string path = Path.Combine(options.AutoshotDir, $"frame_{frameNum}.bmp");
            BmpFile.Write(path, frame, core.ScreenWidth, core.ScreenHeight);
            Emit($"[AUTOSHOT] Frame {frameNum} changed -> {path}");
        }

        // --commands takes over the whole run - see §3.15.
        if (options.CommandsPath != null)
        {
            // CheckAutoshot(n) directly - post-increment, matching this
            // mode's own autoshot filename convention (see §3.15).
            // debugTarget.RefreshProviders() runs here too, once per frame,
            // so the --commands verbs this scriptRunner drives (`spriteoverlay`,
            // any future one reading a provider) see the frame that just ran,
            // not whatever was live at debugTarget's construction.
            var runner = new FrameRunner(core, options.FrameCount, Emit, n => { debugTarget.RefreshProviders(); CheckAutoshot(n); })
            {
                CpuLogStart = options.CpuLogStart,
                CpuLogEnd = options.CpuLogEnd,
                Verbose = options.Verbose,
                OnHalted = debugTarget.RefreshProviders,
            };
            var scriptRunner = new CommandsScriptRunner(runner, debugTarget, debugCmd, Emit);
            if (!scriptRunner.Run(options.CommandsPath))
            {
                return 1;
            }

            FinishRun(core, Emit);
            WriteSaveStateIfRequested(core, options.SaveStatePath, Emit);
            WriteOutLogIfRequested(options.OutPath, log);
            return 0;
        }

        Emit($"[RUN] Executing {options.FrameCount} frames...");

        // CheckAutoshot(n - 1) - the classic loop's autoshot filenames use
        // the frame *about to run* (0-indexed, pre-increment), not
        // FrameRunner's own post-increment CurrentFrame (see §3.15).
        // debugTarget.RefreshProviders() runs here too, once per frame -
        // see the --commands runner's own comment above.
        var classicRunner = new FrameRunner(core, options.FrameCount, Emit, n => { debugTarget.RefreshProviders(); CheckAutoshot(n - 1); })
        {
            CpuLogStart = options.CpuLogStart,
            CpuLogEnd = options.CpuLogEnd,
            Verbose = options.Verbose,
            OnHalted = debugTarget.RefreshProviders,
        };
        while (classicRunner.CurrentFrame < options.FrameCount)
        {
            // Captured before RunFrames(1) advances CurrentFrame, so
            // --tap/--screenshot's frame indexing stays pre-increment (§3.15).
            long frame = classicRunner.CurrentFrame;
            foreach (var tap in options.Taps)
            {
                bool pressed = frame >= tap.Start && frame < tap.End;
                if (pressed) classicRunner.Hold(tap.Button, tap.Controller);
                else classicRunner.Release(tap.Button, tap.Controller);
            }
            classicRunner.RunFrames(1);
            foreach (var shot in options.Screenshots)
            {
                if (shot.Frame == frame)
                {
                    BmpFile.Write(shot.Path, core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight);
                    Emit($"[SCREENSHOT] Frame {frame} -> {shot.Path}");
                }
            }
        }

        FinishRun(core, Emit);

        // Captures whatever scene --tap/frame-count navigation just
        // reached, so a promising moment (found once, maybe after a lot of
        // trial and error) can be jumped back to instantly with
        // --loadstate on every later run instead of re-deriving it.
        WriteSaveStateIfRequested(core, options.SaveStatePath, Emit);

        List<string> commands;
        if (options.ScriptPath != null)
        {
            if (!File.Exists(options.ScriptPath))
            {
                Emit($"[ERROR] Script not found: {options.ScriptPath}");
                return 1;
            }
            commands = File.ReadAllLines(options.ScriptPath)
                .Select(l => l.Trim())
                .Where(l => l.Length > 0 && !l.StartsWith('#'))
                .ToList();
        }
        else
        {
            commands = new List<string> { "watch list" };
            commands.AddRange(debugTarget.Watches.GetWatches().Select(w => $"watch log {w.Id} 500"));
        }

        Emit("");
        Emit("=== Command output ===");
        foreach (string cmd in commands)
        {
            Emit($"> {cmd}");
            Emit(debugCmd.Execute(cmd));
        }

        WriteOutLogIfRequested(options.OutPath, log);

        return 0;
    }

    private static void FinishRun(VenusCore core, Action<string> emit)
    {
        core.Cpu?.FlushVerboseTrace();
        core.Spc700?.FlushVerboseTrace();
        emit($"[RUN] Done, {core.TotalFrames} total frames executed.");
    }

    private static void WriteSaveStateIfRequested(VenusCore core, string? saveStatePath, Action<string> emit)
    {
        if (saveStatePath == null) return;
        core.SaveState(saveStatePath);
        emit($"[STATE] Saved: {saveStatePath}");
    }

    private static void WriteOutLogIfRequested(string? outPath, List<string> log)
    {
        if (outPath == null) return;
        File.WriteAllLines(outPath, log);
        Console.WriteLine($"[OUT] Wrote log to {outPath}");
    }
}
