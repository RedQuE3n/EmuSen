using EmuSen.Common.Imaging;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.HeadlessDebug;
using EmuSen.HeadlessDebug.Cli;
using EmuSen.Shell;

// Headless AI-agent-driven debugging harness - the "not yet built" item
// from Man pages/EmuSen_Core_Gameplan.md §1's backlog. Every investigation
// in this project so far (including the Yoshi/coin WRAM-staging one) has
// required a human to run F4 prompt commands during a live play session
// and manually relay the console output back. This loads a ROM, runs the
// real VenusCore for a fixed number of frames with no window/audio/human
// involved, then feeds a script of the exact same debug commands the F4
// prompt accepts (ShellInterpreter.Execute doesn't care where a
// command line comes from - see that class's own comment) and writes the
// results to a plain log file.
//
// Usage:
//   dotnet run -- <rom> <frames> [--watch space:addr:len[:kind]]... [--script path] [--out path] [--tap frame:button[:duration]]... [--tap2 frame:button[:duration]]... [--loadstate path] [--savestate path] [--screenshot frame:path]...
//   dotnet run -- <rom> <maxframes> --commands path [other flags above except --tap/--tap2/--screenshot/--script]
//
// --watch registers an extra watch before the run starts (kind is
// write/read/both, default write) - space/addr/len match `watch add`'s own
// arguments. --script is a text file of newline-separated debug commands
// (anything ShellInterpreter understands - `watch log`, `disasm`,
// `writers`, etc.) run once after the frame loop finishes; if omitted, a
// default script just dumps every registered watch's full event log,
// which is exactly what the Yoshi/coin investigation needs. --out mirrors
// all output to a file in addition to stdout. --tap holds a button down
// for <duration> frames (default 4) starting at frame <frame> - a
// headless run otherwise sends no input at all, which is fine for a game
// with its own idle-timeout demo (SMW) but leaves others (LttP's
// file-select screen) stuck at a "press Start" prompt forever. --loadstate
// loads a VenusCore.SaveState() file (same format the Hotaru's
// F5/F9 hotkeys and its F4 prompt's own `state save|load` command use)
// before frame 0, for starting directly from an already-reached scene
// instead of re-deriving it via --tap every run. --savestate writes one
// out after the frame loop finishes, to capture a moment for reuse later.
// --screenshot dumps an uncompressed BMP of GetFrameBufferRgba() right
// after the given frame runs - the only way to actually see what a
// headless run reached, short of guessing from RAM addresses and sprite
// dumps alone.
//
// --autoshot <dir> works alongside every mode above (classic frame loop
// and --commands both): instead of guessing a frame number and hoping it
// landed on something interesting - the exact "screenshot every N frames,
// eyeball it, adjust" cycle the SMAS Select Game investigation spent many
// rounds on before --commands existed - it hashes GetFrameBufferRgba()
// every frame and only writes a BMP when the hash actually changes from
// the last saved one, to <dir>/frame_<n>.bmp. Cheap enough for a debugging
// tool (one FNV-1a pass over a ~230KB buffer/frame) even though it means
// paying GetFrameBufferRgba()'s existing per-frame allocation cost on
// every frame rather than only the ones a caller explicitly asked for.
//
// --commands is a fundamentally different mode from all of the above:
// instead of pre-declaring every tap/screenshot by frame number before the
// run starts (fine when the exact timing is already known, unworkable for
// open-ended exploration), it reads an ordered script and executes each
// line as it's reached, so frame-stepping, input, screenshots, and debug
// commands can interleave freely in one process. Investigating Super Mario
// All-Stars' Select Game menu needed a separate full relaunch (reboot +
// replay the whole boot sequence) for every single button guess - this
// collapses an entire investigation into one script, one process, one
// `--out` log. Lines:
//   frames <n>              - advance <n> frames, applying whatever's currently held
//   tap[2] <button> [dur]   - press (tap2 = controller 2) for <dur> frames (default 4), release, same as --tap/--tap2 but inline
//   hold/release <button> [controller]  - set a button's held state without advancing any frames (controller defaults to 1)
//   screenshot <path>       - capture the current frame right now, not tied to a frame number
//   waitstable [maxframes=300] [quietframes=10] - advance one frame at a time until the
//                             framebuffer hash stops changing for <quietframes> in a row (or
//                             <maxframes> is hit) - removes the remaining "run N frames and
//                             hope it settled" guesswork plain `frames` still needs.
//   contactsheet <path> <count> [every=1] [cols=8] [scale=4] - capture <count> frames spaced
//                             <every> apart, downsample each by <scale>, tile into one grid
//                             image - for "is this actually moving" questions across a span of
//                             frames without reviewing N separate screenshots one at a time.
//   vramsheet <path>        - export the current tile/character memory as a BMP (via
//                             IDebugTarget.RenderTileSheet() - core-agnostic; a core with
//                             nothing analogous returns a 0x0 empty image).
//   paletteswatch <path>    - export the current color palette memory as a BMP grid (via
//                             IDebugTarget.RenderPaletteSwatch(), same core-agnostic contract).
//   spriteoverlay <path>    - capture the current frame and draw a green bounding-box outline
//                             for every entry IDebugTarget.GetSprites() reports (already
//                             core-agnostic, no interface change needed for this verb).
//   audiodump <path> [maxsamples] - write whatever's currently buffered in
//                             IDebugTarget.GetAudioSamples() (non-destructive - it never
//                             dequeues) out as a standard 16-bit PCM .wav file.
//   anything else           - passed straight to ShellInterpreter.Execute, same as --script
// <frames> is still required and still means what it always did in every
// other mode - here it becomes a hard safety cap (a script's `frames`
// requests refuse to advance past it) so a typo can't hang the process
// indefinitely. --tap/--tap2/--screenshot/--script are ignored when
// --commands is given; everything else (--watch, --flag, --loadstate,
// --savestate, --verbose, --cpulog, --out) still applies normally.
//
// A separate standalone mode, unrelated to running a ROM at all:
//   dotnet run -- --diffshot <bmp1> <bmp2> <outpath>
// Reads back two of this harness's own BMPs (screenshot/autoshot/contact
// sheet output - not arbitrary external images), highlights every
// differing pixel in magenta over a dimmed copy of the second frame, and
// prints the changed-pixel count/percentage and bounding box - answering
// "what specifically changed" between two frames without eyeballing them
// side by side or improvising an image-diff script per investigation.
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

            object value = rawValue == null ? true : Convert.ChangeType(rawValue, memberType);
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

        var debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        var debugCmd = ShellInterpreter.CreateDefault(debugTarget);

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
        // actually runs per invocation) - see this file's own header
        // comment on --autoshot for why this exists.
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

        // --commands takes over the whole run - see this file's own header
        // comment for the script syntax and why this exists (collapsing an
        // entire multi-guess investigation into one process/one log
        // instead of a full relaunch per experiment).
        if (options.CommandsPath != null)
        {
            // CheckAutoshot(n) directly - FrameRunner's CurrentFrame is
            // already post-increment ("frames completed so far"), the same
            // convention this mode's autoshot filenames have always used.
            var runner = new FrameRunner(core, options.FrameCount, Emit, n => CheckAutoshot(n))
            {
                CpuLogStart = options.CpuLogStart,
                CpuLogEnd = options.CpuLogEnd,
                Verbose = options.Verbose,
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

        // CheckAutoshot(n - 1) - the classic loop's autoshot filenames have
        // always used the frame *about to run* (0-indexed, pre-increment),
        // not FrameRunner's own post-increment CurrentFrame convention;
        // this preserves that exactly instead of silently shifting every
        // autoshot filename by one.
        var classicRunner = new FrameRunner(core, options.FrameCount, Emit, n => CheckAutoshot(n - 1))
        {
            CpuLogStart = options.CpuLogStart,
            CpuLogEnd = options.CpuLogEnd,
            Verbose = options.Verbose,
        };
        while (classicRunner.CurrentFrame < options.FrameCount)
        {
            // Captured before RunFrames(1) advances CurrentFrame - matches
            // the old for-loop's pre-increment `frame` variable exactly, so
            // --tap/--screenshot's frame-indexed timing is bit-for-bit
            // unchanged.
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
