using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Pharaoh;
using EmuSen.Pharaoh.Cli;
using EmuSen.Pharaoh.Imaging;
using EmuSen.Pharaoh.Reference;
using EmuSen.DianaSh.DianaSh.Bin;

// Headless script-driven debugging harness - loads a ROM, runs the
// configured engine for a fixed number of frames with no window/audio/human
// involved, then feeds a script of the exact same debug commands the F4
// prompt accepts and writes the results to a plain log file. Full
// reference for every flag, both frame-loop modes (classic and
// --commands), --diffshot, and the design reasoning behind each:
// EmuSen.DianaSh/DianaSh/Usr/Home/Documents/EmuSen Manual/EmuSen_Debugging_Tools_Reference_v5.md §3.15.
//
// Usage:
//   dotnet run -- <rom> <frames> [--watch space:addr:len[:kind]]... [--script path] [--out path] [--tap frame:button[:duration]]... [--tap2 frame:button[:duration]]... [--loadstate path] [--savestate path] [--screenshot frame:path]...
//   dotnet run -- <rom> <maxframes> --commands path [other flags above except --tap/--tap2/--screenshot/--script]
//   dotnet run -- --diffshot <bmp1> <bmp2> <outpath>
//   dotnet run -- --testroms <dir|rom> [frame-budget]
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
        // Before every other mode, because it relaunches this same argv - see §3.42.
        if (!CpuThrottle.TryParsePercent(args, out int throttlePercent, out string? throttleError))
        {
            Console.WriteLine(throttleError);
            return 1;
        }
        if (throttlePercent > 0 && !CpuThrottle.AlreadyThrottled)
        {
            return CpuThrottle.ReExec(throttlePercent, args);
        }

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

        // Walks the ROM library once and catalogues it - see EmuSen_Galaxia.md §7.
        if (args.Length >= 1 && args[0] == "--index-library")
        {
            return LibraryIndexer.Run(args);
        }

        // --compare and --verify-dictionary are Python now - see EmuSen_Stack.md §3.
        if (args.Length >= 1 && (args[0] == "--compare" || args[0] == "--verify-dictionary"))
        {
            Console.WriteLine($"{args[0]} moved to EmuSen.WiseMan/Reference/analysis - it reads files,");
            Console.WriteLine("not machines, so it no longer needs the emulator to be built:");
            Console.WriteLine();
            Console.WriteLine("  cd EmuSen.WiseMan/Reference/analysis");
            Console.WriteLine("  ./compare.py <oursDir> <theirsDir> <frame> [phaseWindow] [--input F]...");
            Console.WriteLine("  ./verify_dictionary.py <oursDir> <theirsDir>");
            return 1;
        }

        // EmuSen writing the reference probe's own dump protocol - see §3.48.
        if (args.Length >= 1 && args[0] == "--probe")
        {
            if (args.Length < 5)
            {
                Console.WriteLine("Usage: dotnet run -- --probe <rom> <outDir> <startFrame> <endFrame> [stride] [--sig] [--tap F:BTN[:DUR]]...");
                return 1;
            }
            return ProbeRunner.Run(args);
        }

        // Also standalone: third-party NES test ROMs, which report through PRG RAM - see Moon_TestRoms.md.
        if (args.Length >= 1 && args[0] == "--testroms")
        {
            if (args.Length < 2)
            {
                Console.WriteLine("Usage: dotnet run -- --testroms <dir|rom> [frame-budget]");
                return 1;
            }
            if (args.Length >= 3 && !int.TryParse(args[2], out _))
            {
                Console.WriteLine($"[ERROR] frame-budget must be a number, got '{args[2]}'");
                return 1;
            }
            return TestRomRunner.Run(
                args[1],
                args.Length >= 3 ? int.Parse(args[2]) : NesTestRomRunner.DefaultFrameBudget);
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


        var log = new List<string>();
        void Emit(string line)
        {
            Console.WriteLine(line);
            log.Add(line);
        }

        // Parse validated every --flag, so its warnings are ready the moment Emit exists, and land in --out too.
        foreach (string w in warnings) Emit(w);

        using var runtime = new CrystalRuntime();
        using var runtimeLog = new CrystalRuntimeLog(runtime);

        // Off by default; on when asked, since it turns on every individually-enabled logging flag's console output.
        if (options.Verbose) runtime.LogSwitch = true;
        // Any known flag turns the master switch on, as setting one always did; the one known today is that switch.
        if (options.FlagsToEnable.Any(spec => HeadlessDebugOptions.Flags.ContainsKey(spec.Split('=', 2)[0]))) runtime.LogSwitch = true;

        // A create option, latched at open, which is what reads the .srm - see §3.15's --nobattery entry.
        if (options.NoBattery) Emit("[ROM] --nobattery: the cartridge save is neither read nor written.");

        // So a throttled measurement says so in its own log, not just on the parent's console - see §3.42.
        if (CpuThrottle.AlreadyThrottled) Emit("[THROTTLE] This run is inside a CPU quota; every timing below is a slow-machine timing.");

        Emit($"[ROM] Loading: {options.RomPath}");
        CrystalSession session;
        try
        {
            session = runtime.Open(new CrystalOpen { Path = options.RomPath, NoBattery = options.NoBattery });
            runtimeLog.File();
            if (JsonDocument.Parse(session.Info()).RootElement.GetProperty("notices").GetProperty("engine").GetString() is { } notice) Emit($"[core] {notice}");
        }
        catch (CrystalException ex)
        {
            // A cartridge this build can't map is a bad-input answer, not a crash.
            Emit($"[ERROR] {ex.Message}");
            return 1;
        }
        using (session)
        {
            string coreName = JsonDocument.Parse(session.ServiceInfo()).RootElement.GetProperty("engine").GetString() ?? "";
            Emit($"[ROM] Core: {coreName}");

            // Jumping straight to a saved moment sidesteps scripting menu navigation whose timing is not known.
            if (options.LoadStatePath != null)
            {
                if (!File.Exists(options.LoadStatePath))
                {
                    Emit($"[ERROR] State file not found: {options.LoadStatePath}");
                    return 1;
                }
                session.LoadState(options.LoadStatePath);
                Emit($"[STATE] Loaded: {options.LoadStatePath}");
            }

            // The shell over the session's own table, so its debug verbs and the frame call reach one session - see EmuSen_Backend.md §14.8.
            var debugCmd = DianaShInterpreter.CreateForService(session.ServiceTable());

            // The two ranges registered from power-on for the Yoshi/coin investigation, on a core with a WRAM space.
            if (JsonDocument.Parse(session.Spaces()).RootElement.EnumerateArray().Any(s => s.GetProperty("name").GetString() == "WRAM"))
            {
                session.WatchAdd("WRAM", 0x8000, 0x1800, 0);
                session.WatchAdd("WRAM", 0x0D80, 0x0080, 0);
            }

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

            // Shared between both frame-loop modes below, only one of which runs - see §3.15's --autoshot entry.
            ulong? lastAutoshotHash = null;
            if (options.AutoshotDir != null) Directory.CreateDirectory(options.AutoshotDir);
            FrameRunner? current = null;

            void CheckAutoshot(long frameNum)
            {
                if (options.AutoshotDir == null) return;
                var (frame, width, height) = current!.Picture();
                ulong hash = FrameHash.Compute(frame);
                if (lastAutoshotHash == hash) return;
                lastAutoshotHash = hash;
                string path = Path.Combine(options.AutoshotDir, $"frame_{frameNum}.bmp");
                BmpFile.Write(path, frame, width, height);
                Emit($"[AUTOSHOT] Frame {frameNum} changed -> {path}");
            }

            // --commands takes over the whole run - see §3.15.
            if (options.CommandsPath != null)
            {
                // Post-increment autoshot names in this mode; the frame call refreshes the debugger's snapshots itself - see §3.15.
                var runner = current = new FrameRunner(session, options.FrameCount, Emit, n => CheckAutoshot(n)) { Verbose = options.Verbose, RuntimeLog = runtimeLog };
                var scriptRunner = new CommandsScriptRunner(runner, debugCmd, Emit);
                if (!scriptRunner.Run(options.CommandsPath))
                {
                    return 1;
                }

                FinishRun(session, Emit);
                WriteSaveStateIfRequested(session, options.SaveStatePath, Emit);
                WriteOutLogIfRequested(options.OutPath, log);
                return 0;
            }

            Emit($"[RUN] Executing {options.FrameCount} frames...");

            // The classic loop's autoshot names use the frame about to run, pre-increment - see §3.15.
            var classicRunner = current = new FrameRunner(session, options.FrameCount, Emit, n => CheckAutoshot(n - 1)) { Verbose = options.Verbose, RuntimeLog = runtimeLog };
            while (classicRunner.CurrentFrame < options.FrameCount)
            {
                // Captured before RunFrames(1) advances CurrentFrame, so --tap/--screenshot stay pre-increment (§3.15).
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
                        var (rgba, width, height) = classicRunner.Picture();
                        BmpFile.Write(shot.Path, rgba, width, height);
                        Emit($"[SCREENSHOT] Frame {frame} -> {shot.Path}");
                    }
                }
            }

            FinishRun(session, Emit);

            // A moment found once, perhaps after much trial, is jumped back to with --loadstate on every later run.
            WriteSaveStateIfRequested(session, options.SaveStatePath, Emit);

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
                commands.AddRange(JsonDocument.Parse(session.WatchList()).RootElement.EnumerateArray().Select(w => $"watch log {w.GetProperty("id").GetInt64()} 500"));
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
    }

    private static void FinishRun(CrystalSession session, Action<string> emit)
    {
        emit($"[RUN] Done, {JsonDocument.Parse(session.Info()).RootElement.GetProperty("total_frames").GetInt64()} total frames executed.");
    }

    private static void WriteSaveStateIfRequested(CrystalSession session, string? saveStatePath, Action<string> emit)
    {
        if (saveStatePath == null) return;
        session.SaveState(saveStatePath);
        emit($"[STATE] Saved: {saveStatePath}");
    }

    private static void WriteOutLogIfRequested(string? outPath, List<string> log)
    {
        if (outPath == null) return;
        File.WriteAllLines(outPath, log);
        Console.WriteLine($"[OUT] Wrote log to {outPath}");
    }
}
